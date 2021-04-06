// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! Darwinia service. Specialized wrapper over substrate service.

// --- darwinia ---
pub mod chain_spec;
pub mod client;

pub use codec::Codec;

pub use sc_executor::NativeExecutionDispatch;
pub use sc_service::{
	ChainSpec, Configuration, TFullBackend, TFullClient, TLightBackend, TLightClient,
};

pub use chain_spec::{CrabChainSpec, DarwiniaChainSpec};
pub use client::DarwiniaClient;
pub use crab_runtime;
pub use darwinia_primitives::OpaqueBlock as Block;
pub use darwinia_runtime;

// --- std ---
use std::{sync::Arc, time::Duration};
// --- crates ---
use futures::stream::StreamExt;
// --- substrate ---
use sc_basic_authorship::ProposerFactory;
use sc_client_api::{ExecutorProvider, RemoteBackend, StateBackendFor};
use sc_consensus::LongestChain;
use sc_consensus_babe::{BabeBlockImport, BabeLink, BabeParams, Config as BabeConfig};
use sc_executor::native_executor_instance;
use sc_finality_grandpa::{
	Config as GrandpaConfig, FinalityProofProvider as GrandpaFinalityProofProvider, GrandpaParams,
	LinkHalf, SharedVoterState as GrandpaSharedVoterState,
	VotingRulesBuilder as GrandpaVotingRulesBuilder,
};
use sc_keystore::LocalKeystore;
use sc_network::Event;
use sc_service::{
	config::{KeystoreConfig, PrometheusConfig},
	BuildNetworkParams, Error as ServiceError, NoopRpcExtensionBuilder, PartialComponents,
	RpcHandlers, SpawnTasksParams, TaskManager,
};
use sc_telemetry::{TelemetryConnectionNotifier, TelemetrySpan};
use sc_transaction_pool::{BasicPool, FullPool};
use sp_api::ConstructRuntimeApi;
use sp_consensus::{
	import_queue::BasicQueue, CanAuthorWithNativeVersion, DefaultImportQueue, NeverCanAuthor,
};
use sp_inherents::InherentDataProviders;
use sp_runtime::traits::BlakeTwo256;
use sp_trie::PrefixedMemoryDB;
use substrate_prometheus_endpoint::Registry;
// --- darwinia ---
use darwinia_primitives::{AccountId, Balance, Hash, Nonce, Power};
use darwinia_rpc::{
	BabeDeps, DenyUnsafe, FullDeps, GrandpaDeps, LightDeps, RpcExtension, SubscriptionTaskExecutor,
};

type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FullClient<RuntimeApi, Executor> = sc_service::TFullClient<Block, RuntimeApi, Executor>;
type FullGrandpaBlockImport<RuntimeApi, Executor> = sc_finality_grandpa::GrandpaBlockImport<
	FullBackend,
	Block,
	FullClient<RuntimeApi, Executor>,
	FullSelectChain,
>;
type LightBackend = sc_service::TLightBackendWithHash<Block, BlakeTwo256>;
type LightClient<RuntimeApi, Executor> =
	sc_service::TLightClientWithBackend<Block, RuntimeApi, Executor, LightBackend>;

native_executor_instance!(
	pub CrabExecutor,
	crab_runtime::api::dispatch,
	crab_runtime::native_version,
);

native_executor_instance!(
	pub DarwiniaExecutor,
	darwinia_runtime::api::dispatch,
	darwinia_runtime::native_version,
);

/// A set of APIs that darwinia-like runtimes must implement.
pub trait RuntimeApiCollection:
	sp_api::ApiExt<Block, Error = sp_blockchain::Error>
	+ sp_api::Metadata<Block>
	+ sp_authority_discovery::AuthorityDiscoveryApi<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ sp_consensus_babe::BabeApi<Block>
	+ sp_finality_grandpa::GrandpaApi<Block>
	+ sp_offchain::OffchainWorkerApi<Block>
	+ sp_session::SessionKeys<Block>
	+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
	+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
	+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
	+ darwinia_balances_rpc_runtime_api::BalancesApi<Block, AccountId, Balance>
	+ darwinia_header_mmr_rpc_runtime_api::HeaderMMRApi<Block, Hash>
	+ darwinia_staking_rpc_runtime_api::StakingApi<Block, AccountId, Power>
where
	<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}
impl<Api> RuntimeApiCollection for Api
where
	Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::ApiExt<Block, Error = sp_blockchain::Error>
		+ sp_api::Metadata<Block>
		+ sp_authority_discovery::AuthorityDiscoveryApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_consensus_babe::BabeApi<Block>
		+ sp_finality_grandpa::GrandpaApi<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
		+ darwinia_balances_rpc_runtime_api::BalancesApi<Block, AccountId, Balance>
		+ darwinia_header_mmr_rpc_runtime_api::HeaderMMRApi<Block, Hash>
		+ darwinia_staking_rpc_runtime_api::StakingApi<Block, AccountId, Power>,
	<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

pub trait RuntimeExtrinsic: codec::Codec + Send + Sync + 'static {}
impl<E> RuntimeExtrinsic for E where E: codec::Codec + Send + Sync + 'static {}

/// Can be called for a `Configuration` to check if it is a configuration for the `Crab` network.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Crab` network.
	fn is_crab(&self) -> bool;

	/// Returns if this is a configuration for the `Darwinia` network.
	fn is_darwinia(&self) -> bool;
}
impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_crab(&self) -> bool {
		self.id().starts_with("crab")
	}

	fn is_darwinia(&self) -> bool {
		self.id().starts_with("darwinia")
	}
}

// If we're using prometheus, use a registry with a prefix of `darwinia`.
fn set_prometheus_registry(config: &mut Configuration) -> Result<(), ServiceError> {
	if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
		*registry = Registry::new_custom(Some("darwinia".into()), None)?;
	}

	Ok(())
}

#[cfg(feature = "full-node")]
fn new_partial<RuntimeApi, Executor>(
	config: &mut Configuration,
) -> Result<
	PartialComponents<
		FullClient<RuntimeApi, Executor>,
		FullBackend,
		FullSelectChain,
		DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
		FullPool<Block, FullClient<RuntimeApi, Executor>>,
		(
			impl Fn(DenyUnsafe, SubscriptionTaskExecutor) -> RpcExtension,
			(
				BabeBlockImport<
					Block,
					FullClient<RuntimeApi, Executor>,
					FullGrandpaBlockImport<RuntimeApi, Executor>,
				>,
				LinkHalf<Block, FullClient<RuntimeApi, Executor>, FullSelectChain>,
				BabeLink<Block>,
			),
			GrandpaSharedVoterState,
			Option<TelemetrySpan>,
		),
	>,
	ServiceError,
>
where
	Executor: 'static + NativeExecutionDispatch,
	RuntimeApi:
		'static + Send + Sync + ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
{
	if config.keystore_remote.is_some() {
		return Err(ServiceError::Other(format!(
			"Remote Keystores are not supported."
		)));
	}

	set_prometheus_registry(config)?;

	let inherent_data_providers = InherentDataProviders::new();
	let (client, backend, keystore_container, task_manager, telemetry_span) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);
	let select_chain = LongestChain::new(backend.clone());
	let transaction_pool = BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);
	let grandpa_hard_forks = vec![];
	let (grandpa_block_import, grandpa_link) =
		sc_finality_grandpa::block_import_with_authority_set_hard_forks(
			client.clone(),
			&(client.clone() as Arc<_>),
			select_chain.clone(),
			grandpa_hard_forks,
		)?;
	let justification_import = grandpa_block_import.clone();
	let (babe_import, babe_link) = sc_consensus_babe::block_import(
		BabeConfig::get_or_compute(&*client)?,
		grandpa_block_import,
		client.clone(),
	)?;
	let import_queue = sc_consensus_babe::import_queue(
		babe_link.clone(),
		babe_import.clone(),
		Some(Box::new(justification_import)),
		client.clone(),
		select_chain.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
		CanAuthorWithNativeVersion::new(client.executor().clone()),
	)?;
	let justification_stream = grandpa_link.justification_stream();
	let shared_authority_set = grandpa_link.shared_authority_set().clone();
	let shared_voter_state = GrandpaSharedVoterState::empty();
	let finality_proof_provider = GrandpaFinalityProofProvider::new_for_service(
		backend.clone(),
		Some(shared_authority_set.clone()),
	);
	let import_setup = (babe_import.clone(), grandpa_link, babe_link.clone());
	let rpc_setup = shared_voter_state.clone();
	let babe_config = babe_link.config().clone();
	let shared_epoch_changes = babe_link.epoch_changes().clone();
	let rpc_extensions_builder = {
		let client = client.clone();
		let keystore = keystore_container.sync_keystore();
		let transaction_pool = transaction_pool.clone();
		let select_chain = select_chain.clone();
		let chain_spec = config.chain_spec.cloned_box();

		move |deny_unsafe, subscription_executor| -> RpcExtension {
			let deps = FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				select_chain: select_chain.clone(),
				chain_spec: chain_spec.cloned_box(),
				deny_unsafe,
				babe: BabeDeps {
					babe_config: babe_config.clone(),
					shared_epoch_changes: shared_epoch_changes.clone(),
					keystore: keystore.clone(),
				},
				grandpa: GrandpaDeps {
					shared_voter_state: shared_voter_state.clone(),
					shared_authority_set: shared_authority_set.clone(),
					justification_stream: justification_stream.clone(),
					subscription_executor,
					finality_provider: finality_proof_provider.clone(),
				},
			};

			darwinia_rpc::create_full(deps)
		}
	};

	Ok(PartialComponents {
		client,
		backend,
		task_manager,
		keystore_container,
		select_chain,
		import_queue,
		transaction_pool,
		inherent_data_providers,
		other: (
			rpc_extensions_builder,
			import_setup,
			rpc_setup,
			telemetry_span,
		),
	})
}

fn remote_keystore(_url: &String) -> Result<Arc<LocalKeystore>, &'static str> {
	// FIXME: here would the concrete keystore be built,
	//        must return a concrete type (NOT `LocalKeystore`) that
	//        implements `CryptoStore` and `SyncCryptoStore`
	Err("Remote Keystore not supported.")
}

#[cfg(feature = "full-node")]
fn new_full<RuntimeApi, Executor>(
	mut config: Configuration,
	authority_discovery_disabled: bool,
) -> Result<
	(
		TaskManager,
		Arc<FullClient<RuntimeApi, Executor>>,
		RpcHandlers,
	),
	ServiceError,
>
where
	Executor: 'static + NativeExecutionDispatch,
	RuntimeApi:
		'static + Send + Sync + ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
{
	let role = config.role.clone();
	let is_authority = role.is_authority();
	let force_authoring = config.force_authoring;
	let backoff_authoring_blocks =
		Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
	let disable_grandpa = config.disable_grandpa;
	let name = config.network.node_name.clone();
	let PartialComponents {
		client,
		backend,
		mut task_manager,
		mut keystore_container,
		select_chain,
		import_queue,
		transaction_pool,
		inherent_data_providers,
		other: (rpc_extensions_builder, import_setup, rpc_setup, telemetry_span),
	} = new_partial::<RuntimeApi, Executor>(&mut config)?;

	if let Some(url) = &config.keystore_remote {
		match remote_keystore(url) {
			Ok(k) => keystore_container.set_remote_keystore(k),
			Err(e) => {
				return Err(ServiceError::Other(format!(
					"Error hooking up remote keystore for {}: {}",
					url, e
				)))
			}
		};
	}

	let prometheus_registry = config.prometheus_registry().cloned();
	let shared_voter_state = rpc_setup;

	config
		.network
		.extra_sets
		.push(sc_finality_grandpa::grandpa_peers_set_config());

	#[cfg(feature = "cli")]
	config.network.request_response_protocols.push(
		sc_finality_grandpa_warp_sync::request_response_config_for_chain(
			&config,
			task_manager.spawn_handle(),
			backend.clone(),
		),
	);

	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: None,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			backend.clone(),
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let (rpc_handlers, telemetry_connection_notifier) =
		sc_service::spawn_tasks(SpawnTasksParams {
			config,
			backend: backend.clone(),
			client: client.clone(),
			keystore: keystore_container.sync_keystore(),
			network: network.clone(),
			rpc_extensions_builder: Box::new(rpc_extensions_builder),
			transaction_pool: transaction_pool.clone(),
			task_manager: &mut task_manager,
			on_demand: None,
			remote_blockchain: None,
			telemetry_span,
			network_status_sinks,
			system_rpc_tx,
		})?;

	let (block_import, link_half, babe_link) = import_setup;

	if role.is_authority() {
		let can_author_with = CanAuthorWithNativeVersion::new(client.executor().clone());
		let proposer = ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
		);
		let babe_config = BabeParams {
			keystore: keystore_container.sync_keystore(),
			client: client.clone(),
			select_chain,
			block_import,
			env: proposer,
			sync_oracle: network.clone(),
			inherent_data_providers: inherent_data_providers.clone(),
			force_authoring,
			backoff_authoring_blocks,
			babe_link,
			can_author_with,
		};
		let babe = sc_consensus_babe::start_babe(babe_config)?;

		task_manager
			.spawn_essential_handle()
			.spawn_blocking("babe", babe);
	}

	let keystore = if is_authority {
		Some(keystore_container.sync_keystore())
	} else {
		None
	};
	let grandpa_config = GrandpaConfig {
		// FIXME substrate#1578 make this available through chainspec
		gossip_duration: Duration::from_millis(1000),
		justification_period: 512,
		name: Some(name),
		observer_enabled: false,
		keystore,
		is_authority: role.is_network_authority(),
	};
	let enable_grandpa = !disable_grandpa;

	if enable_grandpa {
		let grandpa_config = GrandpaParams {
			config: grandpa_config,
			link: link_half,
			network: network.clone(),
			telemetry_on_connect: telemetry_connection_notifier.map(|x| x.on_connect_stream()),
			voting_rule: GrandpaVotingRulesBuilder::default().build(),
			prometheus_registry: prometheus_registry.clone(),
			shared_voter_state,
		};

		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	if role.is_authority() && !authority_discovery_disabled {
		let authority_discovery_role =
			sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore());
		let dht_event_stream =
			network
				.event_stream("authority-discovery")
				.filter_map(|e| async move {
					match e {
						Event::Dht(e) => Some(e),
						_ => None,
					}
				});
		let (authority_discovery_worker, _service) = sc_authority_discovery::new_worker_and_service(
			client.clone(),
			network,
			Box::pin(dht_event_stream),
			authority_discovery_role,
			prometheus_registry,
		);

		task_manager.spawn_handle().spawn(
			"authority-discovery-worker",
			authority_discovery_worker.run(),
		);
	}

	network_starter.start_network();

	Ok((task_manager, client, rpc_handlers))
}

fn new_light<RuntimeApi, Executor>(
	mut config: Configuration,
) -> Result<
	(
		TaskManager,
		RpcHandlers,
		Option<TelemetryConnectionNotifier>,
	),
	ServiceError,
>
where
	Executor: 'static + NativeExecutionDispatch,
	RuntimeApi:
		'static + Send + Sync + ConstructRuntimeApi<Block, LightClient<RuntimeApi, Executor>>,
	<RuntimeApi as ConstructRuntimeApi<Block, LightClient<RuntimeApi, Executor>>>::RuntimeApi:
		RuntimeApiCollection<StateBackend = StateBackendFor<LightBackend, Block>>,
{
	set_prometheus_registry(&mut config)?;

	let (client, backend, keystore_container, mut task_manager, on_demand, telemetry_span) =
		sc_service::new_light_parts::<Block, RuntimeApi, Executor>(&config)?;

	config
		.network
		.extra_sets
		.push(sc_finality_grandpa::grandpa_peers_set_config());

	let select_chain = LongestChain::new(backend.clone());
	let transaction_pool = Arc::new(BasicPool::new_light(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
		on_demand.clone(),
	));
	let (grandpa_block_import, _) = sc_finality_grandpa::block_import(
		client.clone(),
		&(client.clone() as Arc<_>),
		select_chain.clone(),
	)?;
	let justification_import = grandpa_block_import.clone();
	let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
		BabeConfig::get_or_compute(&*client)?,
		grandpa_block_import,
		client.clone(),
	)?;
	let inherent_data_providers = InherentDataProviders::new();
	// FIXME: pruning task isn't started since light client doesn't do `AuthoritySetup`.
	let import_queue = sc_consensus_babe::import_queue(
		babe_link,
		babe_block_import,
		Some(Box::new(justification_import)),
		client.clone(),
		select_chain.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
		NeverCanAuthor,
	)?;
	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: Some(on_demand.clone()),
			block_announce_validator_builder: None,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			backend.clone(),
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let light_deps = LightDeps {
		remote_blockchain: backend.remote_blockchain(),
		fetcher: on_demand.clone(),
		client: client.clone(),
		pool: transaction_pool.clone(),
	};
	let rpc_extension = darwinia_rpc::create_light(light_deps);

	let (rpc_handlers, telemetry_connection_notifier) =
		sc_service::spawn_tasks(SpawnTasksParams {
			on_demand: Some(on_demand),
			remote_blockchain: Some(backend.remote_blockchain()),
			rpc_extensions_builder: Box::new(NoopRpcExtensionBuilder(rpc_extension)),
			task_manager: &mut task_manager,
			config,
			keystore: keystore_container.sync_keystore(),
			backend,
			transaction_pool,
			client,
			network,
			network_status_sinks,
			system_rpc_tx,
			telemetry_span,
		})?;

	network_starter.start_network();

	Ok((task_manager, rpc_handlers, telemetry_connection_notifier))
}

/// Builds a new object suitable for chain operations.
#[cfg(feature = "full-node")]
pub fn new_chain_ops<Runtime, Dispatch>(
	config: &mut Configuration,
) -> Result<
	(
		Arc<FullClient<Runtime, Dispatch>>,
		Arc<FullBackend>,
		BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
	),
	ServiceError,
>
where
	Dispatch: 'static + NativeExecutionDispatch,
	Runtime: 'static + Send + Sync + ConstructRuntimeApi<Block, FullClient<Runtime, Dispatch>>,
	Runtime::RuntimeApi: RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
{
	config.keystore = KeystoreConfig::InMemory;

	let PartialComponents {
		client,
		backend,
		import_queue,
		task_manager,
		..
	} = new_partial::<Runtime, Dispatch>(config)?;

	Ok((client, backend, import_queue, task_manager))
}

/// Create a new Crab service for a full node.
#[cfg(feature = "full-node")]
pub fn crab_new_full(
	config: Configuration,
	authority_discovery_disabled: bool,
) -> Result<
	(
		TaskManager,
		Arc<impl DarwiniaClient<Block, FullBackend, crab_runtime::RuntimeApi>>,
		RpcHandlers,
	),
	ServiceError,
> {
	let (components, client, rpc_handlers) =
		new_full::<crab_runtime::RuntimeApi, CrabExecutor>(config, authority_discovery_disabled)?;

	Ok((components, client, rpc_handlers))
}

/// Create a new Crab service for a light client.
pub fn crab_new_light(
	config: Configuration,
) -> Result<
	(
		TaskManager,
		RpcHandlers,
		Option<TelemetryConnectionNotifier>,
	),
	ServiceError,
> {
	new_light::<crab_runtime::RuntimeApi, CrabExecutor>(config)
}

/// Create a new Darwinia service for a full node.
#[cfg(feature = "full-node")]
pub fn darwinia_new_full(
	config: Configuration,
	authority_discovery_disabled: bool,
) -> Result<
	(
		TaskManager,
		Arc<impl DarwiniaClient<Block, FullBackend, darwinia_runtime::RuntimeApi>>,
		RpcHandlers,
	),
	ServiceError,
> {
	let (components, client, rpc_handlers) = new_full::<
		darwinia_runtime::RuntimeApi,
		DarwiniaExecutor,
	>(config, authority_discovery_disabled)?;

	Ok((components, client, rpc_handlers))
}

/// Create a new Darwinia service for a light client.
pub fn darwinia_new_light(
	config: Configuration,
) -> Result<
	(
		TaskManager,
		RpcHandlers,
		Option<TelemetryConnectionNotifier>,
	),
	ServiceError,
> {
	new_light::<darwinia_runtime::RuntimeApi, DarwiniaExecutor>(config)
}
