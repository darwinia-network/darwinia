//! Darwinia service. Specialized wrapper over substrate service.

pub mod chain_spec;

// --- crates ---
pub use codec::Codec;
// --- substrate ---
pub use sc_executor::NativeExecutionDispatch;
pub use sc_service::{
	ChainSpec, Configuration, TFullBackend, TFullClient, TLightBackend, TLightClient,
};
// --- darwinia ---
pub use chain_spec::CrabChainSpec;
pub use crab_runtime;
pub use darwinia_primitives::Block;

// --- std ---
use std::{sync::Arc, time::Duration};
// --- crates ---
use futures::stream::StreamExt;
// --- substrate ---
use sc_authority_discovery::{AuthorityDiscovery, Role as AuthorityDiscoveryRole};
use sc_basic_authorship::ProposerFactory;
use sc_client_api::{ExecutorProvider, StateBackendFor};
use sc_consensus::LongestChain;
use sc_consensus_babe::{BabeParams, Config as BabeConfig};
use sc_executor::native_executor_instance;
use sc_finality_grandpa::{
	Config as GrandpaConfig, FinalityProofProvider as GrandpaFinalityProofProvider, GrandpaParams,
	SharedVoterState as GrandpaSharedVoterState,
	StorageAndProofProvider as GrandpaStorageAndProofProvider,
	VotingRulesBuilder as GrandpaVotingRulesBuilder,
};
use sc_network::Event as NetworkEvent;
use sc_service::{
	config::{KeystoreConfig, PrometheusConfig},
	Error as ServiceError, Role as ServiceRole, ServiceBuilder, ServiceComponents, TaskManager,
};
use sc_transaction_pool::{BasicPool, FullChainApi, LightChainApi};
use sp_api::ConstructRuntimeApi;
use sp_consensus::import_queue::BasicQueue;
use sp_core::traits::BareCryptoStorePtr;
use sp_inherents::InherentDataProviders;
use sp_runtime::traits::BlakeTwo256;
use sp_trie::PrefixedMemoryDB;
use substrate_prometheus_endpoint::Registry;
// --- darwinia ---
use darwinia_primitives::{AccountId, Balance, Hash, Nonce, Power};

native_executor_instance!(
	pub CrabExecutor,
	crab_runtime::api::dispatch,
	crab_runtime::native_version,
);

// TODO: mainnet
// native_executor_instance!(
// 	pub DarwiniaExecutor,
// 	darwinia_runtime::api::dispatch,
// 	darwinia_runtime::native_version,
// );

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
macro_rules! new_full_start {
	($config:expr, $runtime:ty, $executor:ty) => {{
		set_prometheus_registry(&mut $config)?;

		let mut import_setup = None;
		let mut rpc_setup = None;
		let inherent_data_providers = InherentDataProviders::new();
		let builder = ServiceBuilder::new_full::<Block, $runtime, $executor>($config)?
			.with_select_chain(|_, backend| Ok(sc_consensus::LongestChain::new(backend.clone())))?
			.with_transaction_pool(|builder| {
				let pool_api =
					FullChainApi::new(builder.client().clone(), builder.prometheus_registry());
				let pool = BasicPool::new_full(
					builder.config().transaction_pool.clone(),
					Arc::new(pool_api),
					builder.prometheus_registry(),
					builder.spawn_handle(),
					builder.client().clone(),
				);

				Ok(pool)
			})?
			.with_import_queue(
				|_, client, mut select_chain, _, spawn_task_handle, registry| {
					let select_chain = select_chain
						.take()
						.ok_or_else(|| ServiceError::SelectChainRequired)?;
					let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
						client.clone(),
						&(client.clone() as Arc<_>),
						select_chain.clone(),
					)?;
					let justification_import = grandpa_block_import.clone();
					let (block_import, babe_link) = sc_consensus_babe::block_import(
						BabeConfig::get_or_compute(&*client)?,
						grandpa_block_import,
						client.clone(),
					)?;

					let import_queue = sc_consensus_babe::import_queue(
						babe_link.clone(),
						block_import.clone(),
						Some(Box::new(justification_import)),
						None,
						client,
						select_chain,
						inherent_data_providers.clone(),
						spawn_task_handle,
						registry,
					)?;

					import_setup = Some((block_import, grandpa_link, babe_link));

					Ok(import_queue)
				},
			)?
			.with_rpc_extensions_builder(|builder| {
				let grandpa_link = import_setup
					.as_ref()
					.map(|s| &s.1)
					.expect("GRANDPA LinkHalf is present for full services or set up failed; qed.");
				let shared_authority_set = grandpa_link.shared_authority_set().clone();
				let shared_voter_state = GrandpaSharedVoterState::empty();

				rpc_setup = Some((shared_voter_state.clone()));

				let babe_link = import_setup
					.as_ref()
					.map(|s| &s.2)
					.expect("BabeLink is present for full services or set up failed; qed.");
				let babe_config = babe_link.config().clone();
				let shared_epoch_changes = babe_link.epoch_changes().clone();
				let client = builder.client().clone();
				let pool = builder.pool().clone();
				let select_chain = builder
					.select_chain()
					.cloned()
					.expect("SelectChain is present for full services or set up failed; qed.");
				let keystore = builder.keystore().clone();

				Ok(move |deny_unsafe| -> darwinia_rpc::RpcExtension {
					let deps = darwinia_rpc::FullDeps {
						client: client.clone(),
						pool: pool.clone(),
						select_chain: select_chain.clone(),
						deny_unsafe,
						babe: darwinia_rpc::BabeDeps {
							babe_config: babe_config.clone(),
							shared_epoch_changes: shared_epoch_changes.clone(),
							keystore: keystore.clone(),
						},
						grandpa: darwinia_rpc::GrandpaDeps {
							shared_voter_state: shared_voter_state.clone(),
							shared_authority_set: shared_authority_set.clone(),
						},
					};

					darwinia_rpc::create_full(deps)
				})
			})?;

		(builder, import_setup, inherent_data_providers, rpc_setup)
		}};
}

/// Builds a new service for a full client.
#[macro_export]
macro_rules! new_full {
	($config:expr, $runtime:ty, $dispatch:ty) => {{
		let (role, force_authoring, name, disable_grandpa) = (
			$config.role.clone(),
			$config.force_authoring,
			$config.network.node_name.clone(),
			$config.disable_grandpa,
			);
		let (builder, mut import_setup, inherent_data_providers, mut rpc_setup) =
			new_full_start!($config, $runtime, $dispatch);
		let ServiceComponents {
			client,
			network,
			select_chain,
			keystore,
			transaction_pool,
			prometheus_registry,
			task_manager,
			telemetry_on_connect_sinks,
			..
		} = builder
			.with_finality_proof_provider(|client, backend| {
				let provider = client as Arc<dyn GrandpaStorageAndProofProvider<_, _>>;
				Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, provider)) as _)
			})?
			.build_full()?;
		let (block_import, link_half, babe_link) = import_setup.take().expect(
			"Link Half and Block Import are present for Full Services or setup failed before. qed",
			);
		let shared_voter_state = rpc_setup.take().expect(
			"The SharedVoterState is present for Full Services or setup failed before. qed",
			);

		if role.is_authority() {
			let select_chain = select_chain.ok_or(ServiceError::SelectChainRequired)?;
			let can_author_with =
				sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());
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
					AuthorityDiscoveryRole::Authority (
						keystore.clone(),
					),
				),
				ServiceRole::Sentry { .. } => (
					vec![],
					AuthorityDiscoveryRole::Sentry,
				),
				_ => unreachable!("Due to outer matches! constraint; qed."),
			};

			let network_event_stream = network.event_stream("authority-discovery");
			let dht_event_stream = network_event_stream.filter_map(|e| async move { match e {
				NetworkEvent::Dht(e) => Some(e),
				_ => None,
			}}).boxed();
			let authority_discovery = AuthorityDiscovery::new(
				client.clone(),
				network.clone(),
				sentries,
				dht_event_stream,
				authority_discovery_role,
				prometheus_registry.clone(),
			);

			task_manager
				.spawn_essential_handle()
				.spawn_blocking("authority-discovery", authority_discovery);
		}

		// if the node isn't actively participating in consensus then it doesn't
		// need a keystore, regardless of which protocol we use below.
		let keystore = if role.is_authority() {
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
			// start the full GRANDPA voter
			// NOTE: non-authorities could run the GRANDPA observer protocol, but at
			// this point the full voter should provide better guarantees of block
			// and vote data availability than the observer. The observer has not
			// been tested extensively yet and having most nodes in a network run it
			// could lead to finality stalls.
			let grandpa_config = GrandpaParams {
				config: grandpa_config,
				link: link_half,
				network: network.clone(),
				inherent_data_providers: inherent_data_providers.clone(),
				telemetry_on_connect: Some(telemetry_on_connect_sinks.on_connect_stream()),
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
				network,
			)?;
			}

		(task_manager, client)
	}}
}

/// Builds a new service for a light client.
macro_rules! new_light {
	($config:expr, $runtime:ty, $dispatch:ty) => {{
		set_prometheus_registry(&mut $config)?;

		let inherent_data_providers = InherentDataProviders::new();

		ServiceBuilder::new_light::<Block, $runtime, $dispatch>($config)?
			.with_select_chain(|_, backend| Ok(LongestChain::new(backend.clone())))?
			.with_transaction_pool(|builder| {
				let fetcher = builder.fetcher().ok_or_else(|| {
					"Trying to start light transaction pool without active fetcher"
				})?;
				let pool_api = LightChainApi::new(builder.client().clone(), fetcher);
				let pool = Arc::new(BasicPool::new_light(
					builder.config().transaction_pool.clone(),
					Arc::new(pool_api),
					builder.prometheus_registry(),
					builder.spawn_handle(),
				));

				Ok(pool)
			})?
			.with_import_queue_and_fprb(
				|_, client, backend, fetcher, mut select_chain, _, spawn_task_handle, registry| {
					let select_chain = select_chain
						.take()
						.ok_or_else(|| ServiceError::SelectChainRequired)?;
					let fetch_checker = fetcher
						.map(|fetcher| fetcher.checker().clone())
						.ok_or_else(|| {
							"Trying to start light import queue without active fetch checker"
						})?;
					let grandpa_block_import = sc_finality_grandpa::light_block_import(
						client.clone(),
						backend,
						&(client.clone() as Arc<_>),
						Arc::new(fetch_checker),
					)?;
					let finality_proof_import = grandpa_block_import.clone();
					let finality_proof_request_builder =
						finality_proof_import.create_finality_proof_request_builder();
					let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
						BabeConfig::get_or_compute(&*client)?,
						grandpa_block_import,
						client.clone(),
					)?;
					// FIXME: pruning task isn't started since light client doesn't do `AuthoritySetup`.
					let import_queue = sc_consensus_babe::import_queue(
						babe_link,
						babe_block_import,
						None,
						Some(Box::new(finality_proof_import)),
						client,
						select_chain,
						inherent_data_providers.clone(),
						spawn_task_handle,
						registry,
					)?;

					Ok((import_queue, finality_proof_request_builder))
				},
			)?
			.with_finality_proof_provider(|client, backend| {
				let provider = client as Arc<dyn GrandpaStorageAndProofProvider<_, _>>;
				Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, provider)) as _)
			})?
			.with_rpc_extensions(|builder| {
				let fetcher = builder
					.fetcher()
					.ok_or_else(|| "Trying to start node RPC without active fetcher")?;
				let remote_blockchain = builder
					.remote_backend()
					.ok_or_else(|| "Trying to start node RPC without active remote blockchain")?;
				let light_deps = darwinia_rpc::LightDeps {
					remote_blockchain,
					fetcher,
					client: builder.client().clone(),
					pool: builder.pool(),
				};

				Ok(darwinia_rpc::create_light(light_deps))
			})?
			.build_light()
			.map(|ServiceComponents { task_manager, .. }| task_manager)
		}};
}

/// A set of APIs that polkadot-like runtimes must implement.
pub trait RuntimeApiCollection<Extrinsic: 'static + Send + Sync + codec::Codec>:
	sp_api::ApiExt<Block, Error = sp_blockchain::Error>
	+ sp_api::Metadata<Block>
	+ sp_authority_discovery::AuthorityDiscoveryApi<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ sp_consensus_babe::BabeApi<Block>
	+ sp_offchain::OffchainWorkerApi<Block>
	+ sp_session::SessionKeys<Block>
	+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
	+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
	+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance, Extrinsic>
	+ darwinia_balances_rpc_runtime_api::BalancesApi<Block, AccountId, Balance>
	+ darwinia_header_mmr_rpc_runtime_api::HeaderMMRApi<Block, Hash>
	+ darwinia_staking_rpc_runtime_api::StakingApi<Block, AccountId, Power>
where
	Extrinsic: RuntimeExtrinsic,
	<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}
impl<Api, Extrinsic> RuntimeApiCollection<Extrinsic> for Api
where
	Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::ApiExt<Block, Error = sp_blockchain::Error>
		+ sp_api::Metadata<Block>
		+ sp_authority_discovery::AuthorityDiscoveryApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_consensus_babe::BabeApi<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance, Extrinsic>
		+ darwinia_balances_rpc_runtime_api::BalancesApi<Block, AccountId, Balance>
		+ darwinia_header_mmr_rpc_runtime_api::HeaderMMRApi<Block, Hash>
		+ darwinia_staking_rpc_runtime_api::StakingApi<Block, AccountId, Power>,
	Extrinsic: RuntimeExtrinsic,
	<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

pub trait RuntimeExtrinsic: codec::Codec + Send + Sync + 'static {}
impl<E> RuntimeExtrinsic for E where E: codec::Codec + Send + Sync + 'static {}

/// Crab client abstraction, this super trait only pulls in functionality required for
/// Crab internal crates like Crab-collator.
pub trait CrabClient<Block, Backend, Runtime>:
	Sized
	+ Send
	+ Sync
	+ sc_client_api::BlockchainEvents<Block>
	+ sp_api::CallApiAt<Block, Error = sp_blockchain::Error, StateBackend = Backend::State>
	+ sp_api::ProvideRuntimeApi<Block, Api = Runtime::RuntimeApi>
	+ sp_blockchain::HeaderBackend<Block>
where
	Backend: sc_client_api::Backend<Block>,
	Block: sp_runtime::traits::Block,
	Runtime: sp_api::ConstructRuntimeApi<Block, Self>,
{
}
impl<Block, Backend, Runtime, Client> CrabClient<Block, Backend, Runtime> for Client
where
	Backend: sc_client_api::Backend<Block>,
	Block: sp_runtime::traits::Block,
	Client: Sized
		+ Send
		+ Sync
		+ sp_api::CallApiAt<Block, Error = sp_blockchain::Error, StateBackend = Backend::State>
		+ sp_api::ProvideRuntimeApi<Block, Api = Runtime::RuntimeApi>
		+ sp_blockchain::HeaderBackend<Block>
		+ sc_client_api::BlockchainEvents<Block>,
	Runtime: sp_api::ConstructRuntimeApi<Block, Self>,
{
}

// /// Darwinia client abstraction, this super trait only pulls in functionality required for
// /// Darwinia internal crates like Darwinia-collator.
// pub trait DarwiniaClient<Block, Backend, Runtime>:
// 	Sized
// 	+ Send
// 	+ Sync
// 	+ sc_client_api::BlockchainEvents<Block>
// 	+ sp_api::CallApiAt<Block, Error = sp_blockchain::Error, StateBackend = Backend::State>
// 	+ sp_api::ProvideRuntimeApi<Block, Api = Runtime::RuntimeApi>
// 	+ sp_blockchain::HeaderBackend<Block>
// where
// 	Backend: sc_client_api::Backend<Block>,
// 	Block: sp_runtime::traits::Block,
// 	Runtime: sp_api::ConstructRuntimeApi<Block, Self>,
// {
// }
// impl<Block, Backend, Runtime, Client> DarwiniaClient<Block, Backend, Runtime> for Client
// where
// 	Backend: sc_client_api::Backend<Block>,
// 	Block: sp_runtime::traits::Block,
// 	Client: Sized
// 		+ Send
// 		+ Sync
// 		+ sp_api::CallApiAt<Block, Error = sp_blockchain::Error, StateBackend = Backend::State>
// 		+ sp_api::ProvideRuntimeApi<Block, Api = Runtime::RuntimeApi>
// 		+ sp_blockchain::HeaderBackend<Block>
// 		+ sc_client_api::BlockchainEvents<Block>,
// 	Runtime: sp_api::ConstructRuntimeApi<Block, Self>,
// {
// }

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

/// Builds a new object suitable for chain operations.
pub fn new_chain_ops<Runtime, Dispatch, Extrinsic>(
	mut config: Configuration,
) -> Result<
	(
		Arc<TFullClient<Block, Runtime, Dispatch>>,
		Arc<TFullBackend<Block>>,
		BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
	),
	ServiceError,
>
where
	Runtime:
		'static + Send + Sync + ConstructRuntimeApi<Block, TFullClient<Block, Runtime, Dispatch>>,
	Runtime::RuntimeApi:
		RuntimeApiCollection<Extrinsic, StateBackend = StateBackendFor<TFullBackend<Block>, Block>>,
	Dispatch: 'static + NativeExecutionDispatch,
	Extrinsic: RuntimeExtrinsic,
{
	config.keystore = KeystoreConfig::InMemory;

	let (builder, _, _, _) = new_full_start!(config, Runtime, Dispatch);

	Ok(builder.to_chain_ops_parts())
}

// If we're using prometheus, use a registry with a prefix of `darwinia`.
fn set_prometheus_registry(config: &mut Configuration) -> Result<(), ServiceError> {
	if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
		*registry = Registry::new_custom(Some("darwinia".into()), None)?;
	}

	Ok(())
}

/// Create a new Crab service for a full node.
#[cfg(feature = "full-node")]
pub fn crab_new_full(
	mut config: Configuration,
) -> Result<
	(
		TaskManager,
		Arc<impl CrabClient<Block, TFullBackend<Block>, crab_runtime::RuntimeApi>>,
	),
	ServiceError,
> {
	let (components, client) = new_full!(config, crab_runtime::RuntimeApi, CrabExecutor);

	Ok((components, client))
}

/// Create a new Crab service for a light client.
pub fn crab_new_light(mut config: Configuration) -> Result<TaskManager, ServiceError> {
	new_light!(config, crab_runtime::RuntimeApi, CrabExecutor)
}

// /// Create a new Darwinia service for a full node.
// #[cfg(feature = "full-node")]
// pub fn darwinia_new_full(
// 	mut config: Configuration,
// ) -> Result<
// 	(
// 		TaskManager,
// 		Arc<impl CrabClient<Block, TFullBackend<Block>, darwinia_runtime::RuntimeApi>>,
// 	),
// 	ServiceError,
// > {
// 	let (components, client) = new_full!(config, darwinia_runtime::RuntimeApi, DarwiniaExecutor);

// 	Ok((components, client))
// }

// /// Create a new Darwinia service for a light client.
// pub fn darwinia_new_light(mut config: Configuration) -> Result<TaskManager, ServiceError> {
// 	new_light!(config, darwinia_runtime::RuntimeApi, DarwiniaExecutor)
// }
