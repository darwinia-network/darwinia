//! Darwinia-specific RPCs implementation.

#![warn(missing_docs)]

// --- std ---
use std::sync::Arc;
// --- darwinia ---
use darwinia_primitives::{AccountId, Balance, Block, Nonce, Power};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Instantiate all RPC extensions.
pub fn create_full<C, P, UE>(client: Arc<C>, pool: Arc<P>) -> RpcExtension
where
	C: sp_api::ProvideRuntimeApi<Block>,
	C: sp_blockchain::HeaderBackend<Block>,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance, UE>,
	C::Api: darwinia_balances_rpc::BalancesRuntimeApi<Block, AccountId, Balance>,
	C::Api: darwinia_staking_rpc::StakingRuntimeApi<Block, AccountId, Power>,
	P: 'static + Sync + Send + sp_transaction_pool::TransactionPool,
	UE: 'static + Send + Sync + codec::Codec,
{
	// --- substrate ---
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
	use substrate_frame_rpc_system::{FullSystem, SystemApi};
	// --- darwinia ---
	use darwinia_balances_rpc::{Balances, BalancesApi};
	use darwinia_staking_rpc::{Staking, StakingApi};

	let mut io = jsonrpc_core::IoHandler::default();
	io.extend_with(SystemApi::to_delegate(FullSystem::new(
		client.clone(),
		pool,
	)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client.clone(),
	)));
	io.extend_with(BalancesApi::to_delegate(Balances::new(client.clone())));
	io.extend_with(StakingApi::to_delegate(Staking::new(client)));

	io
}

/// Instantiate all RPC extensions for light node.
pub fn create_light<C, P, F, UE>(
	client: Arc<C>,
	remote_blockchain: Arc<dyn sc_client_api::light::RemoteBlockchain<Block>>,
	fetcher: Arc<F>,
	pool: Arc<P>,
) -> RpcExtension
where
	C: sp_api::ProvideRuntimeApi<Block>,
	C: sp_blockchain::HeaderBackend<Block>,
	C: 'static + Send + Sync,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance, UE>,
	C::Api: darwinia_balances_rpc::BalancesRuntimeApi<Block, AccountId, Balance>,
	C::Api: darwinia_staking_rpc::StakingRuntimeApi<Block, AccountId, Power>,
	P: 'static + Send + Sync + sp_transaction_pool::TransactionPool,
	F: 'static + sc_client_api::light::Fetcher<Block>,
	UE: 'static + Send + Sync + codec::Codec,
{
	// --- substrate ---
	use substrate_frame_rpc_system::{LightSystem, SystemApi};

	let mut io = jsonrpc_core::IoHandler::default();
	io.extend_with(SystemApi::<AccountId, Nonce>::to_delegate(
		LightSystem::new(client, remote_blockchain, fetcher, pool),
	));
	io
}
