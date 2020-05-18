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
// --- substrate ---
use sc_client::LongestChain;
use sc_executor::native_executor_instance;
use sc_finality_grandpa::FinalityProofProvider as GrandpaFinalityProofProvider;
use sc_service::{
	config::PrometheusConfig, error::Error as ServiceError, AbstractService, Role, ServiceBuilder,
	ServiceBuilderCommand, TLightCallExecutor,
};
use sp_api::ConstructRuntimeApi;
use sp_runtime::traits::BlakeTwo256;
use substrate_prometheus_endpoint::Registry;
// --- darwinia ---
use darwinia_primitives::{AccountId, Balance, Nonce, Power};

native_executor_instance!(
	pub CrabExecutor,
	crab_runtime::api::dispatch,
	crab_runtime::native_version,
	// TODO: benchmarking
	// frame_benchmarking::benchmarking::HostFunctions,
);

/// A set of APIs that darwinia-like runtimes must implement.
pub trait RuntimeApiCollection<Extrinsic: 'static + Send + Sync + codec::Codec>:
	sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
	+ sp_api::ApiExt<Block, Error = sp_blockchain::Error>
	+ sp_consensus_babe::BabeApi<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
	+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance, Extrinsic>
	+ darwinia_balances_rpc_runtime_api::BalancesApi<Block, AccountId, Balance>
	+ darwinia_staking_rpc_runtime_api::StakingApi<Block, AccountId, Power>
	+ sp_api::Metadata<Block>
	+ sp_offchain::OffchainWorkerApi<Block>
	+ sp_session::SessionKeys<Block>
	+ sp_authority_discovery::AuthorityDiscoveryApi<Block>
where
	Extrinsic: RuntimeExtrinsic,
	<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api, Extrinsic> RuntimeApiCollection<Extrinsic> for Api
where
	Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::ApiExt<Block, Error = sp_blockchain::Error>
		+ sp_consensus_babe::BabeApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance, Extrinsic>
		+ darwinia_balances_rpc_runtime_api::BalancesApi<Block, AccountId, Balance>
		+ darwinia_staking_rpc_runtime_api::StakingApi<Block, AccountId, Power>
		+ sp_api::Metadata<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_authority_discovery::AuthorityDiscoveryApi<Block>,
	Extrinsic: RuntimeExtrinsic,
	<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

pub trait RuntimeExtrinsic: 'static + Send + Sync + codec::Codec {}

impl<E> RuntimeExtrinsic for E where E: 'static + Send + Sync + codec::Codec {}

/// Can be called for a `Configuration` to check if it is a configuration for the `Kusama` network.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Crab` network.
	fn is_crab(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_crab(&self) -> bool {
		self.id().starts_with("crab")
	}
}

// If we're using prometheus, use a registry with a prefix of `darwinia`.
fn set_prometheus_registry(config: &mut Configuration) -> Result<(), ServiceError> {
	if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
		*registry = Registry::new_custom(Some("darwinia".into()), None)?;
	}

	Ok(())
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
macro_rules! new_full_start {
	($config:expr, $runtime:ty, $executor:ty) => {{
		set_prometheus_registry(&mut $config)?;

		let mut import_setup = None;
		let inherent_data_providers = sp_inherents::InherentDataProviders::new();
		let builder = sc_service::ServiceBuilder::new_full::<Block, $runtime, $executor>($config)?
			.with_select_chain(|_, backend| Ok(LongestChain::new(backend.clone())))?
			.with_transaction_pool(|config, client, _, prometheus_registry| {
				let pool_api = sc_transaction_pool::FullChainApi::new(client.clone());
				let pool = sc_transaction_pool::BasicPool::new(
					config,
					std::sync::Arc::new(pool_api),
					prometheus_registry,
				);
				Ok(pool)
			})?
			.with_import_queue(|_config, client, mut select_chain, _| {
				let select_chain = select_chain
					.take()
					.ok_or_else(|| sc_service::Error::SelectChainRequired)?;

				let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
					client.clone(),
					&(client.clone() as Arc<_>),
					select_chain,
				)?;

				let justification_import = grandpa_block_import.clone();

				let (block_import, babe_link) = sc_consensus_babe::block_import(
					sc_consensus_babe::Config::get_or_compute(&*client)?,
					grandpa_block_import,
					client.clone(),
				)?;

				let import_queue = sc_consensus_babe::import_queue(
					babe_link.clone(),
					block_import.clone(),
					Some(Box::new(justification_import)),
					None,
					client,
					inherent_data_providers.clone(),
				)?;

				import_setup = Some((block_import, grandpa_link, babe_link));
				Ok(import_queue)
			})?
			.with_rpc_extensions(|builder| -> Result<darwinia_rpc::RpcExtension, _> {
				Ok(darwinia_rpc::create_full(
					builder.client().clone(),
					builder.pool(),
				))
			})?;

		(builder, import_setup, inherent_data_providers)
		}};
}

/// Builds a new service for a full client.
#[macro_export]
macro_rules! new_full {
	(
		$config:expr,
		$runtime:ty,
		$dispatch:ty
	) => {{
		// --- crates ---
		use futures::stream::StreamExt;
		// --- substrate ---
		use sc_network::Event;
		use sc_client_api::ExecutorProvider;

		let (role, is_authority, force_authoring, name, disable_grandpa) = (
			$config.role.clone(),
			$config.role.is_authority(),
			$config.force_authoring,
			$config.network.node_name.clone(),
			$config.disable_grandpa,
		);

		let (builder, mut import_setup, inherent_data_providers) = new_full_start!($config, $runtime, $dispatch);

		let service = builder
			.with_finality_proof_provider(|client, backend| {
				let provider = client as Arc<dyn sc_finality_grandpa::StorageAndProofProvider<_, _>>;
				Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, provider)) as _)
			})?
			.build()?;

		let (block_import, link_half, babe_link) = import_setup.take()
			.expect("Link Half and Block Import are present for Full Services or setup failed before. qed");

		let client = service.client();

		if is_authority {
			let proposer = sc_basic_authorship::ProposerFactory::new(
				service.client(),
				service.transaction_pool(),
			);

			let select_chain = service.select_chain().ok_or(ServiceError::SelectChainRequired)?;
			let can_author_with =
				sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

			let babe_config = sc_consensus_babe::BabeParams {
				keystore: service.keystore(),
				client: client.clone(),
				select_chain,
				block_import,
				env: proposer,
				sync_oracle: service.network(),
				inherent_data_providers: inherent_data_providers.clone(),
				force_authoring,
				babe_link,
				can_author_with,
			};

			let babe = sc_consensus_babe::start_babe(babe_config)?;
			service.spawn_essential_task("babe", babe);
		}

		if matches!(role, Role::Authority{..} | Role::Sentry{..}) {
			let (sentries, authority_discovery_role) = match role {
				Role::Authority { ref sentry_nodes } => (
					sentry_nodes.clone(),
					sc_authority_discovery::Role::Authority (
						service.keystore(),
					),
				),
				Role::Sentry {..} => (
					vec![],
					sc_authority_discovery::Role::Sentry,
				),
				_ => unreachable!("Due to outer matches! constraint; qed."),
			};

			let network = service.network();
			let network_event_stream = network.event_stream("authority-discovery");
			let dht_event_stream = network_event_stream.filter_map(|e| async move { match e {
				Event::Dht(e) => Some(e),
				_ => None,
			}}).boxed();
			let authority_discovery = sc_authority_discovery::AuthorityDiscovery::new(
				service.client(),
				network,
				sentries,
				dht_event_stream,
				authority_discovery_role,
				service.prometheus_registry(),
			);

			service.spawn_task("authority-discovery", authority_discovery);
		}

		// if the node isn't actively participating in consensus then it doesn't
		// need a keystore, regardless of which protocol we use below.
		let keystore = if is_authority {
			Some(service.keystore())
		} else {
			None
		};

		let config = sc_finality_grandpa::Config {
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
			// NOTE: unlike in substrate we are currently running the full
			// GRANDPA voter protocol for all full nodes (regardless of whether
			// they're validators or not). at this point the full voter should
			// provide better guarantees of block and vote data availability than
			// the observer.

			let grandpa_config = sc_finality_grandpa::GrandpaParams {
				config,
				link: link_half,
				network: service.network(),
				inherent_data_providers: inherent_data_providers.clone(),
				telemetry_on_connect: Some(service.telemetry_on_connect_stream()),
				voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
				prometheus_registry: service.prometheus_registry(),
			};

			service.spawn_essential_task(
				"grandpa-voter",
				sc_finality_grandpa::run_grandpa_voter(grandpa_config)?
			);
		} else {
			sc_finality_grandpa::setup_disabled_grandpa(
				client.clone(),
				&inherent_data_providers,
				service.network(),
			)?;
		}

		service
	}}
}

/// Builds a new service for a light client.
#[macro_export]
macro_rules! new_light {
	($config:expr, $runtime:ty, $dispatch:ty) => {{
		crate::set_prometheus_registry(&mut $config)?;
		let inherent_data_providers = sp_inherents::InherentDataProviders::new();

		ServiceBuilder::new_light::<Block, $runtime, $dispatch>($config)?
			.with_select_chain(|_, backend| Ok(LongestChain::new(backend.clone())))?
			.with_transaction_pool(|config, client, fetcher, prometheus_registry| {
				let fetcher = fetcher.ok_or_else(|| {
					"Trying to start light transaction pool without active fetcher"
				})?;
				let pool_api =
					sc_transaction_pool::LightChainApi::new(client.clone(), fetcher.clone());
				let pool = sc_transaction_pool::BasicPool::with_revalidation_type(
					config,
					Arc::new(pool_api),
					prometheus_registry,
					sc_transaction_pool::RevalidationType::Light,
				);
				Ok(pool)
			})?
			.with_import_queue_and_fprb(|_, client, backend, fetcher, _, _| {
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
					sc_consensus_babe::Config::get_or_compute(&*client)?,
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
					inherent_data_providers.clone(),
				)?;

				Ok((import_queue, finality_proof_request_builder))
			})?
			.with_finality_proof_provider(|client, backend| {
				let provider =
					client as Arc<dyn sc_finality_grandpa::StorageAndProofProvider<_, _>>;
				Ok(Arc::new(sc_finality_grandpa::FinalityProofProvider::new(
					backend, provider,
				)) as _)
			})?
			.with_rpc_extensions(|builder| -> Result<darwinia_rpc::RpcExtension, _> {
				let fetcher = builder
					.fetcher()
					.ok_or_else(|| "Trying to start node RPC without active fetcher")?;
				let remote_blockchain = builder
					.remote_backend()
					.ok_or_else(|| "Trying to start node RPC without active remote blockchain")?;

				Ok(darwinia_rpc::create_light(
					builder.client().clone(),
					remote_blockchain,
					fetcher,
					builder.pool(),
				))
			})?
			.build()
		}};
}

/// Builds a new object suitable for chain operations.
pub fn new_chain_ops<Runtime, Dispatch, Extrinsic>(
	mut config: Configuration,
) -> Result<impl ServiceBuilderCommand<Block = Block>, ServiceError>
where
	Runtime: 'static
		+ Send
		+ Sync
		+ ConstructRuntimeApi<Block, sc_service::TFullClient<Block, Runtime, Dispatch>>,
	Runtime::RuntimeApi: RuntimeApiCollection<
		Extrinsic,
		StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
	>,
	Dispatch: 'static + NativeExecutionDispatch,
	Extrinsic: RuntimeExtrinsic,
	<Runtime::RuntimeApi as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
	config.keystore = sc_service::config::KeystoreConfig::InMemory;
	Ok(new_full_start!(config, Runtime, Dispatch).0)
}

/// Create a new Crab service for a full node.
#[cfg(feature = "full-node")]
pub fn crab_new_full(mut config: Configuration) -> Result<impl AbstractService, ServiceError> {
	Ok(new_full!(config, crab_runtime::RuntimeApi, CrabExecutor))
}

/// Create a new Crab service for a light client.
pub fn crab_new_light(
	mut config: Configuration,
) -> Result<
	impl AbstractService<
		Block = Block,
		RuntimeApi = crab_runtime::RuntimeApi,
		Backend = TLightBackend<Block>,
		SelectChain = LongestChain<TLightBackend<Block>, Block>,
		CallExecutor = TLightCallExecutor<Block, CrabExecutor>,
	>,
	ServiceError,
> {
	new_light!(config, crab_runtime::RuntimeApi, CrabExecutor)
}
