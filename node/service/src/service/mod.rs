// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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

pub mod api;
use api::*;

pub mod crab;
pub use crab::Executor as CrabExecutor;

pub mod darwinia;
pub use darwinia::Executor as DarwiniaExecutor;

pub mod dvm;

// --- std ---
use std::sync::Arc;
// --- darwinia-network ---
use darwinia_primitives::{OpaqueBlock as Block, *};

type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FullClient<RuntimeApi, Executor> =
	sc_service::TFullClient<Block, RuntimeApi, sc_executor::NativeElseWasmExecutor<Executor>>;
type FullGrandpaBlockImport<RuntimeApi, Executor> = sc_finality_grandpa::GrandpaBlockImport<
	FullBackend,
	Block,
	FullClient<RuntimeApi, Executor>,
	FullSelectChain,
>;

type ServiceResult<T> = Result<T, sc_service::Error>;
type RpcServiceResult = Result<darwinia_rpc::RpcExtension, sc_service::Error>;

/// Can be called for a `Configuration` to check if it is a configuration for the `Crab` network.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Crab` network.
	fn is_crab(&self) -> bool;

	/// Returns true if this configuration is for a development network.
	fn is_dev(&self) -> bool;
}
impl IdentifyVariant for Box<dyn sc_service::ChainSpec> {
	fn is_crab(&self) -> bool {
		self.id().starts_with("crab")
	}

	fn is_dev(&self) -> bool {
		self.id().ends_with("dev")
	}
}

// If we're using prometheus, use a registry with a prefix of `darwinia`.
fn set_prometheus_registry(config: &mut sc_service::Configuration) -> ServiceResult<()> {
	// --- paritytech ---
	use sc_service::config::PrometheusConfig;
	use substrate_prometheus_endpoint::Registry;

	if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
		*registry = Registry::new_custom(Some("darwinia".into()), None)?;
	}

	Ok(())
}

/// Builds a new object suitable for chain operations.
#[cfg(feature = "full-node")]
pub fn new_chain_ops<Runtime, Dispatch>(
	config: &mut sc_service::Configuration,
) -> ServiceResult<(
	Arc<FullClient<Runtime, Dispatch>>,
	Arc<FullBackend>,
	sc_consensus::BasicQueue<Block, sp_trie::PrefixedMemoryDB<Hashing>>,
	sc_service::TaskManager,
)>
where
	Runtime:
		'static + Send + Sync + sp_api::ConstructRuntimeApi<Block, FullClient<Runtime, Dispatch>>,
	Runtime::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Dispatch: 'static + sc_executor::NativeExecutionDispatch,
{
	// --- paritytech ---
	use sc_service::{config::KeystoreConfig, PartialComponents};

	config.keystore = KeystoreConfig::InMemory;

	let PartialComponents { client, backend, import_queue, task_manager, .. } =
		new_partial::<Runtime, Dispatch>(config)?;

	Ok((client, backend, import_queue, task_manager))
}

#[cfg(feature = "full-node")]
fn new_full<RuntimeApi, Executor>(
	mut config: sc_service::Configuration,
	authority_discovery_disabled: bool,
	eth_rpc_config: darwinia_rpc::EthRpcConfig,
) -> ServiceResult<(
	sc_service::TaskManager,
	Arc<FullClient<RuntimeApi, Executor>>,
	sc_service::RpcHandlers,
)>
where
	RuntimeApi: 'static
		+ Send
		+ Sync
		+ sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: 'static + sc_executor::NativeExecutionDispatch,
{
	// -- std ---
	use std::{collections::BTreeMap, sync::Mutex, time::Duration};
	// --- crates.io ---
	use futures::stream::StreamExt;
	// --- paritytech ---
	use fc_rpc::EthBlockDataCache;
	use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
	use sc_authority_discovery::WorkerConfig;
	use sc_basic_authorship::ProposerFactory;
	use sc_client_api::ExecutorProvider;
	use sc_consensus_babe::{BabeParams, SlotProportion};
	use sc_finality_grandpa::{
		warp_proof::NetworkProvider, Config as GrandpaConfig,
		FinalityProofProvider as GrandpaFinalityProofProvider, GrandpaParams,
		SharedVoterState as GrandpaSharedVoterState,
		VotingRulesBuilder as GrandpaVotingRulesBuilder,
	};
	use sc_network::Event;
	use sc_service::{BuildNetworkParams, PartialComponents, SpawnTasksParams};
	use sp_consensus::CanAuthorWithNativeVersion;
	// --- darwinia-network ---
	use darwinia_rpc::*;
	use dvm::DvmTaskParams;

	let role = config.role.clone();
	let is_authority = role.is_authority();
	let is_archive = config.state_pruning.is_archive();
	let force_authoring = config.force_authoring;
	let disable_grandpa = config.disable_grandpa;
	let name = config.network.node_name.clone();
	let prometheus_registry = config.prometheus_registry().cloned();
	let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;

	config.network.extra_sets.push(sc_finality_grandpa::grandpa_peers_set_config());

	let backoff_authoring_blocks =
		Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
	let PartialComponents {
		client,
		backend,
		mut task_manager,
		keystore_container,
		select_chain,
		import_queue,
		transaction_pool,
		other: ((babe_import, grandpa_link, babe_link), mut telemetry),
	} = new_partial::<RuntimeApi, Executor>(&mut config)?;

	// if let Some(url) = &config.keystore_remote {
	// 	match service::remote_keystore(url) {
	// 		Ok(k) => keystore_container.set_remote_keystore(k),
	// 		Err(e) => {
	// 			return Err(ServiceError::Other(format!(
	// 				"Error hooking up remote keystore for {}: {}",
	// 				url, e
	// 			)))
	// 		}
	// 	};
	// }

	let warp_sync = Arc::new(NetworkProvider::new(
		backend.clone(),
		grandpa_link.shared_authority_set().clone(),
		Vec::new(),
	));
	let (network, system_rpc_tx, network_starter) =
		sc_service::build_network(BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: None,
			warp_sync: Some(warp_sync),
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let dvm_backend = dvm::open_backend(&config)?;
	let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));
	let overrides = overrides_handle(client.clone());
	let block_data_cache = Arc::new(EthBlockDataCache::new(
		task_manager.spawn_handle(),
		overrides.clone(),
		eth_rpc_config.eth_log_block_cache,
		eth_rpc_config.eth_log_block_cache,
	));
	let fee_history_cache: FeeHistoryCache = Arc::new(Mutex::new(BTreeMap::new()));
	let eth_rpc_requesters = DvmTaskParams {
		task_manager: &task_manager,
		client: client.clone(),
		substrate_backend: backend.clone(),
		dvm_backend: dvm_backend.clone(),
		filter_pool: filter_pool.clone(),
		is_archive,
		rpc_config: eth_rpc_config.clone(),
		fee_history_cache: fee_history_cache.clone(),
		overrides: overrides.clone(),
		sync_from: match config.chain_spec.name() {
			"Darwinia" => 9_453_251,
			"Darwinia Crab" => 4_969_901,
			_ => 0,
		},
	}
	.spawn_task();
	let subscription_task_executor = SubscriptionTaskExecutor::new(task_manager.spawn_handle());
	let shared_voter_state = GrandpaSharedVoterState::empty();
	let babe_config = babe_link.config().clone();
	let shared_epoch_changes = babe_link.epoch_changes().clone();
	let justification_stream = grandpa_link.justification_stream();
	let shared_authority_set = grandpa_link.shared_authority_set().clone();
	let finality_proof_provider = GrandpaFinalityProofProvider::new_for_service(
		backend.clone(),
		Some(shared_authority_set.clone()),
	);
	let rpc_extensions_builder = Box::new({
		let client = client.clone();
		let keystore = keystore_container.sync_keystore();
		let transaction_pool = transaction_pool.clone();
		let select_chain = select_chain.clone();
		let chain_spec = config.chain_spec.cloned_box();
		let shared_voter_state = shared_voter_state.clone();
		let network = network.clone();

		move |deny_unsafe, subscription_executor: SubscriptionTaskExecutor| -> RpcServiceResult {
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
					subscription_executor: subscription_executor.clone(),
					finality_proof_provider: finality_proof_provider.clone(),
				},
				eth: EthDeps {
					config: eth_rpc_config.clone(),
					graph: transaction_pool.pool().clone(),
					is_authority,
					network: network.clone(),
					filter_pool: filter_pool.clone(),
					backend: dvm_backend.clone(),
					fee_history_cache: fee_history_cache.clone(),
					overrides: overrides.clone(),
					block_data_cache: block_data_cache.clone(),
					rpc_requesters: eth_rpc_requesters.clone(),
				},
			};

			darwinia_rpc::create_full(deps, subscription_task_executor.clone()).map_err(Into::into)
		}
	});
	let rpc_handlers = sc_service::spawn_tasks(SpawnTasksParams {
		config,
		backend: backend.clone(),
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		network: network.clone(),
		rpc_extensions_builder,
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		on_demand: None,
		remote_blockchain: None,
		system_rpc_tx,
		telemetry: telemetry.as_mut(),
	})?;

	if is_authority {
		let can_author_with = CanAuthorWithNativeVersion::new(client.executor().clone());
		let proposer = ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);
		let slot_duration = babe_link.config().slot_duration();
		let babe_config = BabeParams {
			keystore: keystore_container.sync_keystore(),
			client: client.clone(),
			select_chain,
			block_import: babe_import,
			env: proposer,
			sync_oracle: network.clone(),
			justification_sync_link: network.clone(),
			create_inherent_data_providers: move |_, ()| {
				async move {
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
					let slot =

					sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_duration(
						*timestamp,
						slot_duration,
					);

					Ok((timestamp, slot))
				}
			},
			force_authoring,
			backoff_authoring_blocks,
			babe_link,
			can_author_with,
			block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
			max_block_proposal_slot_portion: None,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
		};
		let babe = sc_consensus_babe::start_babe(babe_config)?;

		task_manager.spawn_essential_handle().spawn_blocking("babe", babe);
	}

	if is_authority && !authority_discovery_disabled {
		let authority_discovery_role =
			sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore());
		let dht_event_stream =
			network.event_stream("authority-discovery").filter_map(|e| async move {
				match e {
					Event::Dht(e) => Some(e),
					_ => None,
				}
			});
		let (authority_discovery_worker, _service) =
			sc_authority_discovery::new_worker_and_service_with_config(
				WorkerConfig {
					publish_non_global_ips: auth_disc_publish_non_global_ips,
					..Default::default()
				},
				client.clone(),
				network.clone(),
				Box::pin(dht_event_stream),
				authority_discovery_role,
				prometheus_registry.clone(),
			);

		task_manager
			.spawn_handle()
			.spawn("authority-discovery-worker", authority_discovery_worker.run());
	}

	let keystore = if is_authority { Some(keystore_container.sync_keystore()) } else { None };

	if !disable_grandpa {
		let grandpa_config = GrandpaParams {
			config: GrandpaConfig {
				// FIXME substrate#1578 make this available through chainspec
				gossip_duration: Duration::from_millis(1000),
				justification_period: 512,
				name: Some(name),
				observer_enabled: false,
				keystore,
				local_role: role,
				telemetry: telemetry.as_ref().map(|x| x.handle()),
			},
			link: grandpa_link,
			network,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			voting_rule: GrandpaVotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state,
		};

		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	network_starter.start_network();

	Ok((task_manager, client, rpc_handlers))
}

#[cfg(feature = "full-node")]
fn new_partial<RuntimeApi, Executor>(
	config: &mut sc_service::Configuration,
) -> ServiceResult<
	sc_service::PartialComponents<
		FullClient<RuntimeApi, Executor>,
		FullBackend,
		FullSelectChain,
		sc_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
		sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
		(
			(
				sc_consensus_babe::BabeBlockImport<
					Block,
					FullClient<RuntimeApi, Executor>,
					FullGrandpaBlockImport<RuntimeApi, Executor>,
				>,
				sc_finality_grandpa::LinkHalf<
					Block,
					FullClient<RuntimeApi, Executor>,
					FullSelectChain,
				>,
				sc_consensus_babe::BabeLink<Block>,
			),
			Option<sc_telemetry::Telemetry>,
		),
	>,
>
where
	RuntimeApi: 'static
		+ Send
		+ Sync
		+ sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: 'static + sc_executor::NativeExecutionDispatch,
{
	// --- paritytech ---
	use sc_client_api::ExecutorProvider;
	use sc_consensus::LongestChain;
	use sc_consensus_babe::Config as BabeConfig;
	use sc_executor::NativeElseWasmExecutor;
	use sc_service::{Error as ServiceError, PartialComponents};
	use sc_telemetry::{Error as TelemetryError, TelemetryWorker};
	use sc_transaction_pool::BasicPool;
	use sp_consensus::CanAuthorWithNativeVersion;

	if config.keystore_remote.is_some() {
		return Err(ServiceError::Other(format!("Remote Keystores are not supported.")));
	}

	set_prometheus_registry(config)?;

	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, TelemetryError> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);

			Ok((worker, telemetry))
		})
		.transpose()?;
	let executor = <NativeElseWasmExecutor<Executor>>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
	);
	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			&config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", worker.run());
		telemetry
	});
	let client = Arc::new(client);
	let select_chain = LongestChain::new(backend.clone());
	let transaction_pool = BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);
	let grandpa_hard_forks = vec![];
	let (grandpa_block_import, grandpa_link) =
		sc_finality_grandpa::block_import_with_authority_set_hard_forks(
			client.clone(),
			&(client.clone() as Arc<_>),
			select_chain.clone(),
			grandpa_hard_forks,
			telemetry.as_ref().map(|x| x.handle()),
		)?;
	let justification_import = grandpa_block_import.clone();
	let (babe_import, babe_link) = sc_consensus_babe::block_import(
		BabeConfig::get_or_compute(&*client)?,
		grandpa_block_import,
		client.clone(),
	)?;
	let slot_duration = babe_link.config().slot_duration();
	let import_queue = sc_consensus_babe::import_queue(
		babe_link.clone(),
		babe_import.clone(),
		Some(Box::new(justification_import)),
		client.clone(),
		select_chain.clone(),
		move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			let slot =
				sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_duration(
					*timestamp,
					slot_duration,
				);

			Ok((timestamp, slot))
		},
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
		CanAuthorWithNativeVersion::new(client.executor().clone()),
		telemetry.as_ref().map(|x| x.handle()),
	)?;
	let import_setup = (babe_import.clone(), grandpa_link, babe_link.clone());

	Ok(PartialComponents {
		client,
		backend,
		task_manager,
		keystore_container,
		select_chain,
		import_queue,
		transaction_pool,
		other: (import_setup, telemetry),
	})
}
