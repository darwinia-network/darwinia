//! Darwinia service. Specialized wrapper over substrate service.

// --- darwinia ---
pub mod chain_spec;
pub mod client;

// --- crates ---
pub use codec::Codec;
// --- substrate ---
pub use sc_executor::NativeExecutionDispatch;
use sc_network::NetworkService;
pub use sc_service::{
	ChainSpec, Configuration, TFullBackend, TFullClient, TLightBackend, TLightClient,
};
// --- darwinia ---
pub use chain_spec::{CrabChainSpec, DarwiniaChainSpec};
pub use client::DarwiniaClient;
pub use crab_runtime;
pub use darwinia_primitives::Block;
pub use darwinia_runtime;

// --- std ---
use std::{sync::Arc, time::Duration};
// --- crates ---
use futures::stream::StreamExt;
// --- substrate ---
use sc_authority_discovery::Role as AuthorityDiscoveryRole;
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
use sc_network::Event as NetworkEvent;
use sc_service::{
	config::{KeystoreConfig, PrometheusConfig},
	BuildNetworkParams, Error as ServiceError, NoopRpcExtensionBuilder, PartialComponents,
	Role as ServiceRole, SpawnTasksParams, TaskManager, TelemetryConnectionSinks,
};
use sc_transaction_pool::{BasicPool, FullPool};
use sp_api::ConstructRuntimeApi;
use sp_consensus::{
	import_queue::BasicQueue, CanAuthorWithNativeVersion, DefaultImportQueue, NeverCanAuthor,
};
use sp_core::traits::BareCryptoStorePtr;
use sp_inherents::InherentDataProviders;
use sp_runtime::traits::BlakeTwo256;
use sp_trie::PrefixedMemoryDB;
use substrate_prometheus_endpoint::Registry;
// --- darwinia ---
use darwinia_primitives::{AccountId, Balance, Hash, Nonce, Power};
use darwinia_rpc::{
	BabeDeps, DenyUnsafe, FullDeps, GrandpaDeps, LightDeps, RpcExtension, SubscriptionTaskExecutor,
};
use dvm_consensus::FrontierBlockImport;

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
	+ dvm_rpc_primitives::EthereumRuntimeRPCApi<Block>
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
		+ darwinia_staking_rpc_runtime_api::StakingApi<Block, AccountId, Power>
		+ dvm_rpc_primitives::EthereumRuntimeRPCApi<Block>,
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
			impl Fn(
				DenyUnsafe,
				bool,
				Arc<NetworkService<Block, Hash>>,
				SubscriptionTaskExecutor,
			) -> RpcExtension,
			(
				BabeBlockImport<
					Block,
					FullClient<RuntimeApi, Executor>,
					FrontierBlockImport<
						Block,
						FullGrandpaBlockImport<RuntimeApi, Executor>,
						FullClient<RuntimeApi, Executor>,
					>,
				>,
				LinkHalf<Block, FullClient<RuntimeApi, Executor>, FullSelectChain>,
				BabeLink<Block>,
			),
			(
				GrandpaSharedVoterState,
				Arc<GrandpaFinalityProofProvider<FullBackend, Block>>,
			),
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
	set_prometheus_registry(config)?;

	let inherent_data_providers = InherentDataProviders::new();
	let (client, backend, keystore, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);
	let select_chain = LongestChain::new(backend.clone());
	let transaction_pool = BasicPool::new_full(
		config.transaction_pool.clone(),
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
	let frontier_block_import =
		FrontierBlockImport::new(grandpa_block_import.clone(), client.clone(), true);
	let (babe_import, babe_link) = sc_consensus_babe::block_import(
		BabeConfig::get_or_compute(&*client)?,
		frontier_block_import,
		client.clone(),
	)?;
	let import_queue = sc_consensus_babe::import_queue(
		babe_link.clone(),
		babe_import.clone(),
		Some(Box::new(justification_import)),
		None,
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
	let finality_proof_provider =
		GrandpaFinalityProofProvider::new_for_service(backend.clone(), client.clone());
	let import_setup = (babe_import.clone(), grandpa_link, babe_link.clone());
	let rpc_setup = (shared_voter_state.clone(), finality_proof_provider.clone());
	let babe_config = babe_link.config().clone();
	let shared_epoch_changes = babe_link.epoch_changes().clone();
	let subscription_task_executor = SubscriptionTaskExecutor::new(task_manager.spawn_handle());
	let rpc_extensions_builder = {
		let client = client.clone();
		let keystore = keystore.clone();
		let transaction_pool = transaction_pool.clone();
		let select_chain = select_chain.clone();

		move |deny_unsafe, is_authority, network, subscription_executor| -> RpcExtension {
			let deps = FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				select_chain: select_chain.clone(),
				deny_unsafe,
				is_authority,
				network,
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

			darwinia_rpc::create_full(deps, subscription_task_executor.clone())
		}
	};

	Ok(PartialComponents {
		client,
		backend,
		task_manager,
		keystore,
		select_chain,
		import_queue,
		transaction_pool,
		inherent_data_providers,
		other: (rpc_extensions_builder, import_setup, rpc_setup),
	})
}

#[cfg(feature = "full-node")]
fn new_full<RuntimeApi, Executor>(
	mut config: Configuration,
) -> Result<(TaskManager, Arc<FullClient<RuntimeApi, Executor>>), ServiceError>
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
	let disable_grandpa = config.disable_grandpa;
	let name = config.network.node_name.clone();
	let PartialComponents {
		client,
		backend,
		mut task_manager,
		keystore,
		select_chain,
		import_queue,
		transaction_pool,
		inherent_data_providers,
		other: (rpc_extensions_builder, import_setup, rpc_setup),
	} = new_partial::<RuntimeApi, Executor>(&mut config)?;
	let prometheus_registry = config.prometheus_registry().cloned();
	let (shared_voter_state, finality_proof_provider) = rpc_setup;
	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: None,
			finality_proof_request_builder: None,
			finality_proof_provider: Some(finality_proof_provider.clone()),
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

	let telemetry_connection_sinks = TelemetryConnectionSinks::default();

	sc_service::spawn_tasks(SpawnTasksParams {
		config,
		backend: backend.clone(),
		client: client.clone(),
		keystore: keystore.clone(),
		network: network.clone(),
		rpc_extensions_builder: {
			let wrap_rpc_extensions_builder = {
				let network = network.clone();

				move |deny_unsafe, subscription_executor| -> RpcExtension {
					rpc_extensions_builder(
						deny_unsafe,
						is_authority,
						network.clone(),
						subscription_executor,
					)
				}
			};

			Box::new(wrap_rpc_extensions_builder)
		},
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		on_demand: None,
		remote_blockchain: None,
		telemetry_connection_sinks: telemetry_connection_sinks.clone(),
		network_status_sinks,
		system_rpc_tx,
	})?;

	let (block_import, link_half, babe_link) = import_setup;

	if role.is_authority() {
		let can_author_with = CanAuthorWithNativeVersion::new(client.executor().clone());
		let proposer = ProposerFactory::new(
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
		);
		let babe_config = BabeParams {
			keystore: keystore.clone(),
			client: client.clone(),
			select_chain,
			block_import,
			env: proposer,
			sync_oracle: network.clone(),
			inherent_data_providers: inherent_data_providers.clone(),
			force_authoring,
			babe_link,
			can_author_with,
		};
		let babe = sc_consensus_babe::start_babe(babe_config)?;

		task_manager
			.spawn_essential_handle()
			.spawn_blocking("babe", babe);
	}

	if matches!(role, ServiceRole::Authority { .. } | ServiceRole::Sentry { .. }) {
		let (sentries, authority_discovery_role) = match role {
			ServiceRole::Authority { ref sentry_nodes } => (
				sentry_nodes.clone(),
				AuthorityDiscoveryRole::Authority(keystore.clone()),
			),
			ServiceRole::Sentry { .. } => (vec![], AuthorityDiscoveryRole::Sentry),
			_ => unreachable!("Due to outer matches! constraint; qed."),
		};

		let network_event_stream = network.event_stream("authority-discovery");
		let dht_event_stream = network_event_stream
			.filter_map(|e| async move {
				match e {
					NetworkEvent::Dht(e) => Some(e),
					_ => None,
				}
			})
			.boxed();
		let (authority_discovery_worker, _) = sc_authority_discovery::new_worker_and_service(
			client.clone(),
			network.clone(),
			sentries,
			dht_event_stream,
			authority_discovery_role,
			prometheus_registry.clone(),
		);

		task_manager
			.spawn_handle()
			.spawn("authority-discovery-worker", authority_discovery_worker);
	}

	let keystore = if is_authority {
		Some(keystore.clone() as BareCryptoStorePtr)
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
			inherent_data_providers: inherent_data_providers.clone(),
			telemetry_on_connect: Some(telemetry_connection_sinks.on_connect_stream()),
			voting_rule: GrandpaVotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state,
		};

		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	} else {
		sc_finality_grandpa::setup_disabled_grandpa(
			client.clone(),
			&inherent_data_providers,
			network.clone(),
		)?;
	}

	network_starter.start_network();

	Ok((task_manager, client))
}

fn new_light<RuntimeApi, Executor>(mut config: Configuration) -> Result<TaskManager, ServiceError>
where
	Executor: 'static + NativeExecutionDispatch,
	RuntimeApi:
		'static + Send + Sync + ConstructRuntimeApi<Block, LightClient<RuntimeApi, Executor>>,
	<RuntimeApi as ConstructRuntimeApi<Block, LightClient<RuntimeApi, Executor>>>::RuntimeApi:
		RuntimeApiCollection<StateBackend = StateBackendFor<LightBackend, Block>>,
{
	set_prometheus_registry(&mut config)?;

	let (client, backend, keystore, mut task_manager, on_demand) =
		sc_service::new_light_parts::<Block, RuntimeApi, Executor>(&config)?;
	let select_chain = LongestChain::new(backend.clone());
	let transaction_pool = Arc::new(BasicPool::new_light(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
		on_demand.clone(),
	));
	let grandpa_block_import = sc_finality_grandpa::light_block_import(
		client.clone(),
		backend.clone(),
		&(client.clone() as Arc<_>),
		Arc::new(on_demand.checker().clone()),
	)?;
	let finality_proof_import = grandpa_block_import.clone();
	let finality_proof_request_builder =
		finality_proof_import.create_finality_proof_request_builder();
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
		None,
		Some(Box::new(finality_proof_import)),
		client.clone(),
		select_chain.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
		NeverCanAuthor,
	)?;
	let finality_proof_provider =
		GrandpaFinalityProofProvider::new_for_service(backend.clone(), client.clone());
	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: Some(on_demand.clone()),
			block_announce_validator_builder: None,
			finality_proof_request_builder: Some(finality_proof_request_builder),
			finality_proof_provider: Some(finality_proof_provider),
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

	sc_service::spawn_tasks(SpawnTasksParams {
		on_demand: Some(on_demand),
		remote_blockchain: Some(backend.remote_blockchain()),
		rpc_extensions_builder: Box::new(NoopRpcExtensionBuilder(rpc_extension)),
		task_manager: &mut task_manager,
		telemetry_connection_sinks: TelemetryConnectionSinks::default(),
		config,
		keystore,
		backend,
		transaction_pool,
		client,
		network,
		network_status_sinks,
		system_rpc_tx,
	})?;

	network_starter.start_network();

	Ok(task_manager)
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
) -> Result<
	(
		TaskManager,
		Arc<impl DarwiniaClient<Block, FullBackend, crab_runtime::RuntimeApi>>,
	),
	ServiceError,
> {
	let (components, client) = new_full::<crab_runtime::RuntimeApi, CrabExecutor>(config)?;

	Ok((components, client))
}

/// Create a new Crab service for a light client.
pub fn crab_new_light(config: Configuration) -> Result<TaskManager, ServiceError> {
	new_light::<crab_runtime::RuntimeApi, CrabExecutor>(config)
}

/// Create a new Darwinia service for a full node.
#[cfg(feature = "full-node")]
pub fn darwinia_new_full(
	config: Configuration,
) -> Result<
	(
		TaskManager,
		Arc<impl DarwiniaClient<Block, FullBackend, darwinia_runtime::RuntimeApi>>,
	),
	ServiceError,
> {
	let (components, client) = new_full::<darwinia_runtime::RuntimeApi, DarwiniaExecutor>(config)?;

	Ok((components, client))
}

/// Create a new Darwinia service for a light client.
pub fn darwinia_new_light(config: Configuration) -> Result<TaskManager, ServiceError> {
	new_light::<darwinia_runtime::RuntimeApi, DarwiniaExecutor>(config)
}
