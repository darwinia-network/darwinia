//! Darwinia-specific RPCs implementation.

#![warn(missing_docs)]

// --- std ---
use std::sync::Arc;
// --- substrate ---
use sp_api::ProvideRuntimeApi;
use sp_transaction_pool::TransactionPool;
use substrate_frame_rpc_system::SystemApi;
// --- darwinia ---
use darwinia_primitives::{AccountId, Balance, Block, Nonce};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Instantiate all RPC extensions.
pub fn create_full<C, P, UE>(client: Arc<C>, pool: Arc<P>) -> RpcExtension
where
	C: ProvideRuntimeApi<Block>,
	C: sc_client::blockchain::HeaderBackend<Block>,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance, UE>,
	P: TransactionPool + Sync + Send + 'static,
	UE: codec::Codec + Send + Sync + 'static,
{
	// --- substrate ---
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
	use substrate_frame_rpc_system::FullSystem;

	let mut io = jsonrpc_core::IoHandler::default();
	io.extend_with(SystemApi::to_delegate(FullSystem::new(
		client.clone(),
		pool,
	)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client,
	)));
	io
}

/// Instantiate all RPC extensions for light node.
pub fn create_light<C, P, F, UE>(
	client: Arc<C>,
	remote_blockchain: Arc<dyn sc_client::light::blockchain::RemoteBlockchain<Block>>,
	fetcher: Arc<F>,
	pool: Arc<P>,
) -> RpcExtension
where
	C: ProvideRuntimeApi<Block>,
	C: sc_client::blockchain::HeaderBackend<Block>,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance, UE>,
	P: TransactionPool + Sync + Send + 'static,
	F: sc_client::light::fetcher::Fetcher<Block> + 'static,
	UE: codec::Codec + Send + Sync + 'static,
{
	// --- substrate ---
	use substrate_frame_rpc_system::LightSystem;

	let mut io = jsonrpc_core::IoHandler::default();
	io.extend_with(SystemApi::<AccountId, Nonce>::to_delegate(
		LightSystem::new(client, remote_blockchain, fetcher, pool),
	));
	io
}
