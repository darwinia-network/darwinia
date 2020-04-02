//! Darwinia service. Specialized wrapper over substrate service.

// --- substrate ---
pub use sc_client::Client;
pub use sc_executor::NativeExecutionDispatch;
pub use sc_service::{
	config::PrometheusConfig, AbstractService, Configuration, ServiceBuilderCommand, TFullBackend,
	TLightBackend, TLightCallExecutor,
};
pub use sp_api::ConstructRuntimeApi;
pub use sp_runtime::traits::BlakeTwo256;

// --- std ---
use std::sync::Arc;
// --- substrate ---
use sc_client::LongestChain;
use sc_executor::native_executor_instance;
use sc_finality_grandpa::FinalityProofProvider as GrandpaFinalityProofProvider;
use sc_service::{error::Error as ServiceError, ServiceBuilder};
use sp_inherents::InherentDataProviders;
use substrate_prometheus_endpoint::Registry;
// --- darwinia ---
use darwinia_primitives::{AccountId, Balance, Block, Nonce};

native_executor_instance!(
	pub CrabExecutor,
	crab_runtime::api::dispatch,
	crab_runtime::native_version,
	// TODO: benchmarking
	// frame_benchmarking::benchmarking::HostFunctions,
);

/// A set of APIs that darwinia-like runtimes must implement.
pub trait RuntimeApiCollection<Extrinsic: codec::Codec + Send + Sync + 'static>:
	sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
	+ sp_api::ApiExt<Block, Error = sp_blockchain::Error>
	+ sp_consensus_babe::BabeApi<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
	+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance, Extrinsic>
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
		+ sp_api::Metadata<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_authority_discovery::AuthorityDiscoveryApi<Block>,
	Extrinsic: RuntimeExtrinsic,
	<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

pub trait RuntimeExtrinsic: codec::Codec + Send + Sync + 'static {}

impl<E> RuntimeExtrinsic for E where E: codec::Codec + Send + Sync + 'static {}

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
			.with_select_chain(|_, backend| Ok(sc_client::LongestChain::new(backend.clone())))?
			.with_transaction_pool(|config, client, _fetcher| {
				let pool_api = sc_transaction_pool::FullChainApi::new(client.clone());
				Ok(sc_transaction_pool::BasicPool::new(
					config,
					std::sync::Arc::new(pool_api),
				))
			})?
			.with_import_queue(|_config, client, mut select_chain, _transaction_pool| {
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

/// Builds a new object suitable for chain operations.
pub fn new_chain_ops<Runtime, Dispatch, Extrinsic>(
	mut config: Configuration,
) -> Result<impl ServiceBuilderCommand<Block = Block>, ServiceError>
where
	Runtime: ConstructRuntimeApi<Block, sc_service::TFullClient<Block, Runtime, Dispatch>>
		+ Send
		+ Sync
		+ 'static,
	Runtime::RuntimeApi: RuntimeApiCollection<
		Extrinsic,
		StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
	>,
	Dispatch: NativeExecutionDispatch + 'static,
	Extrinsic: RuntimeExtrinsic,
	<Runtime::RuntimeApi as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
	config.keystore = sc_service::config::KeystoreConfig::InMemory;
	Ok(new_full_start!(config, Runtime, Dispatch).0)
}

/// Create a new Crab service for a light client.
pub fn crab_new_light(
	config: Configuration,
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
	new_light(config)
}

// We can't use service::TLightClient due to
// Rust bug: https://github.com/rust-lang/rust/issues/43580
type TLocalLightClient<Runtime, Dispatch> = Client<
	sc_client::light::backend::Backend<sc_client_db::light::LightStorage<Block>, BlakeTwo256>,
	sc_client::light::call_executor::GenesisCallExecutor<
		sc_client::light::backend::Backend<sc_client_db::light::LightStorage<Block>, BlakeTwo256>,
		sc_client::LocalCallExecutor<
			sc_client::light::backend::Backend<
				sc_client_db::light::LightStorage<Block>,
				BlakeTwo256,
			>,
			sc_executor::NativeExecutor<Dispatch>,
		>,
	>,
	Block,
	Runtime,
>;

/// Builds a new service for a light client.
pub fn new_light<Runtime, Dispatch, Extrinsic>(
	mut config: Configuration,
) -> Result<
	impl AbstractService<
		Block = Block,
		RuntimeApi = Runtime,
		Backend = TLightBackend<Block>,
		SelectChain = LongestChain<TLightBackend<Block>, Block>,
		CallExecutor = TLightCallExecutor<Block, Dispatch>,
	>,
	ServiceError,
>
where
	Runtime: Send + Sync + 'static,
	Runtime::RuntimeApi: RuntimeApiCollection<
		Extrinsic,
		StateBackend = sc_client_api::StateBackendFor<TLightBackend<Block>, Block>,
	>,
	Dispatch: NativeExecutionDispatch + 'static,
	Extrinsic: RuntimeExtrinsic,
	Runtime: sp_api::ConstructRuntimeApi<Block, TLocalLightClient<Runtime, Dispatch>>,
{
	set_prometheus_registry(&mut config)?;

	let inherent_data_providers = InherentDataProviders::new();

	ServiceBuilder::new_light::<Block, Runtime, Dispatch>(config)?
		.with_select_chain(|_, backend| Ok(LongestChain::new(backend.clone())))?
		.with_transaction_pool(|config, client, fetcher| {
			let fetcher = fetcher
				.ok_or_else(|| "Trying to start light transaction pool without active fetcher")?;
			let pool_api = sc_transaction_pool::LightChainApi::new(client.clone(), fetcher.clone());
			let pool = sc_transaction_pool::BasicPool::with_revalidation_type(
				config,
				Arc::new(pool_api),
				sc_transaction_pool::RevalidationType::Light,
			);
			Ok(pool)
		})?
		.with_import_queue_and_fprb(|_config, client, backend, fetcher, _select_chain, _| {
			let fetch_checker = fetcher
				.map(|fetcher| fetcher.checker().clone())
				.ok_or_else(|| "Trying to start light import queue without active fetch checker")?;
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
			let provider = client as Arc<dyn sc_finality_grandpa::StorageAndProofProvider<_, _>>;
			Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, provider)) as _)
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
}
