//! Darwinia-specific RPCs implementation.

#![warn(missing_docs)]

// --- crates ---
pub use jsonrpc_pubsub::manager::SubscriptionManager;
// --- substrate ---
pub use sc_rpc_api::DenyUnsafe;

// --- std ---
use std::sync::Arc;
// --- substrate ---
use sp_api::ProvideRuntimeApi;
// --- darwinia ---
use darwinia_primitives::{AccountId, Balance, Block, BlockNumber, Hash, Nonce, Power};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Extra dependencies for BABE.
pub struct BabeDeps {
	/// BABE protocol config.
	pub babe_config: sc_consensus_babe::Config,
	/// BABE pending epoch changes.
	pub shared_epoch_changes:
		sc_consensus_epochs::SharedEpochChanges<Block, sc_consensus_babe::Epoch>,
	/// The keystore that manages the keys of the node.
	pub keystore: sc_keystore::KeyStorePtr,
}

/// Dependencies for GRANDPA
pub struct GrandpaDeps {
	/// Voting round info.
	pub shared_voter_state: sc_finality_grandpa::SharedVoterState,
	/// Authority set info.
	pub shared_authority_set: sc_finality_grandpa::SharedAuthoritySet<Hash, BlockNumber>,
	/// Receives notifications about justification events from Grandpa.
	pub justification_stream: sc_finality_grandpa::GrandpaJustificationStream<Block>,
	/// Subscription manager to keep track of pubsub subscribers.
	pub subscriptions: jsonrpc_pubsub::manager::SubscriptionManager,
}

/// Full client dependencies
pub struct FullDeps<C, P, SC> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// The SelectChain Strategy
	pub select_chain: SC,
	/// Whether to deny unsafe calls
	pub deny_unsafe: sc_rpc_api::DenyUnsafe,
	/// BABE specific dependencies.
	pub babe: BabeDeps,
	/// GRANDPA specific dependencies.
	pub grandpa: GrandpaDeps,
}

/// Light client extra dependencies.
pub struct LightDeps<C, F, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Remote access to the blockchain (async).
	pub remote_blockchain: Arc<dyn sc_client_api::light::RemoteBlockchain<Block>>,
	/// Fetcher instance.
	pub fetcher: Arc<F>,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, SC>(deps: FullDeps<C, P, SC>) -> RpcExtension
where
	C: 'static + Send + Sync,
	C: ProvideRuntimeApi<Block>,
	C: sp_blockchain::HeaderBackend<Block>
		+ sp_blockchain::HeaderMetadata<Block, Error = sp_blockchain::Error>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: sc_consensus_babe::BabeApi<Block>,
	C::Api: sp_block_builder::BlockBuilder<Block>,
	C::Api: darwinia_balances_rpc::BalancesRuntimeApi<Block, AccountId, Balance>,
	C::Api: darwinia_header_mmr_rpc::HeaderMMRRuntimeApi<Block, Hash>,
	C::Api: darwinia_staking_rpc::StakingRuntimeApi<Block, AccountId, Power>,
	P: 'static + sp_transaction_pool::TransactionPool,
	SC: 'static + sp_consensus::SelectChain<Block>,
{
	// --- substrate ---
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
	use sc_consensus_babe_rpc::{BabeApi, BabeRpcHandler};
	use sc_finality_grandpa_rpc::{GrandpaApi, GrandpaRpcHandler};
	use substrate_frame_rpc_system::{FullSystem, SystemApi};
	// --- darwinia ---
	use darwinia_balances_rpc::{Balances, BalancesApi};
	use darwinia_header_mmr_rpc::{HeaderMMR, HeaderMMRApi};
	use darwinia_staking_rpc::{Staking, StakingApi};

	let FullDeps {
		client,
		pool,
		select_chain,
		deny_unsafe,
		babe,
		grandpa,
	} = deps;
	let mut io = jsonrpc_core::IoHandler::default();

	io.extend_with(SystemApi::to_delegate(FullSystem::new(
		client.clone(),
		pool,
		deny_unsafe,
	)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client.clone(),
	)));
	{
		let BabeDeps {
			keystore,
			babe_config,
			shared_epoch_changes,
		} = babe;
		io.extend_with(BabeApi::to_delegate(BabeRpcHandler::new(
			client.clone(),
			shared_epoch_changes,
			keystore,
			babe_config,
			select_chain,
			deny_unsafe,
		)));
	}
	{
		let GrandpaDeps {
			shared_voter_state,
			shared_authority_set,
			justification_stream,
			subscriptions,
		} = grandpa;
		io.extend_with(GrandpaApi::to_delegate(GrandpaRpcHandler::new(
			shared_authority_set,
			shared_voter_state,
			justification_stream,
			subscriptions,
		)));
	}
	io.extend_with(BalancesApi::to_delegate(Balances::new(client.clone())));
	io.extend_with(HeaderMMRApi::to_delegate(HeaderMMR::new(client.clone())));
	io.extend_with(StakingApi::to_delegate(Staking::new(client)));

	io
}

/// Instantiate all RPC extensions for light node.
pub fn create_light<C, P, F>(deps: LightDeps<C, F, P>) -> RpcExtension
where
	C: 'static + Send + Sync,
	C: ProvideRuntimeApi<Block>,
	C: sp_blockchain::HeaderBackend<Block>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	P: 'static + sp_transaction_pool::TransactionPool,
	F: 'static + sc_client_api::Fetcher<Block>,
{
	// --- substrate ---
	use substrate_frame_rpc_system::{LightSystem, SystemApi};

	let LightDeps {
		client,
		pool,
		remote_blockchain,
		fetcher,
	} = deps;
	let mut io = jsonrpc_core::IoHandler::default();

	io.extend_with(SystemApi::<Hash, AccountId, Nonce>::to_delegate(
		LightSystem::new(client, remote_blockchain, fetcher, pool),
	));

	io
}
