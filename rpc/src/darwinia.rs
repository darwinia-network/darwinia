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

//! Darwinia-specific RPCs implementation.

// --- std ---
use std::sync::Arc;
// --- crates.io ---
use jsonrpc_core::IoHandler;
// --- darwinia-network ---
use crate::*;
use darwinia_common_primitives::*;

/// Full client dependencies
pub struct FullDeps<C, P, SC, B> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// The SelectChain Strategy
	pub select_chain: SC,
	/// A copy of the chain spec.
	pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: sc_rpc::DenyUnsafe,
	/// BABE specific dependencies.
	pub babe: BabeDeps,
	/// GRANDPA specific dependencies.
	pub grandpa: GrandpaDeps<B>,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, SC, B>(deps: FullDeps<C, P, SC, B>) -> RpcResult
where
	C: 'static
		+ Send
		+ Sync
		+ sp_api::ProvideRuntimeApi<Block>
		+ sc_client_api::AuxStore
		+ sp_blockchain::HeaderBackend<Block>
		+ sp_blockchain::HeaderMetadata<Block, Error = sp_blockchain::Error>,
	C::Api: sp_block_builder::BlockBuilder<Block>
		+ sc_consensus_babe::BabeApi<Block>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ darwinia_balances_rpc::BalancesRuntimeApi<Block, AccountId, Balance>
		+ darwinia_fee_market_rpc::FeeMarketRuntimeApi<Block, Balance>
		+ darwinia_staking_rpc::StakingRuntimeApi<Block, AccountId, Power>,
	P: 'static + sc_transaction_pool_api::TransactionPool,
	SC: 'static + sp_consensus::SelectChain<Block>,
	B: 'static + Send + Sync + sc_client_api::Backend<Block>,
	B::State: sc_client_api::StateBackend<sp_runtime::traits::HashFor<Block>>,
{
	// --- paritytech ---
	use pallet_transaction_payment_rpc::*;
	use sc_consensus_babe_rpc::*;
	use sc_finality_grandpa_rpc::*;
	use sc_sync_state_rpc::*;
	use substrate_frame_rpc_system::*;
	// --- darwinia-network ---
	use darwinia_balances_rpc::*;
	use darwinia_fee_market_rpc::*;
	use darwinia_staking_rpc::*;

	let FullDeps {
		client,
		pool,
		select_chain,
		chain_spec,
		deny_unsafe,
		babe: BabeDeps {
			keystore,
			babe_config,
			shared_epoch_changes,
		},
		grandpa:
			GrandpaDeps {
				shared_voter_state,
				shared_authority_set,
				justification_stream,
				subscription_executor,
				finality_proof_provider,
			},
	} = deps;
	let mut io = IoHandler::default();

	io.extend_with(SystemApi::to_delegate(FullSystem::new(
		client.clone(),
		pool,
		deny_unsafe,
	)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client.clone(),
	)));
	io.extend_with(BabeApi::to_delegate(BabeRpcHandler::new(
		client.clone(),
		shared_epoch_changes.clone(),
		keystore,
		babe_config,
		select_chain,
		deny_unsafe,
	)));
	io.extend_with(GrandpaApi::to_delegate(GrandpaRpcHandler::new(
		shared_authority_set.clone(),
		shared_voter_state,
		justification_stream,
		subscription_executor,
		finality_proof_provider,
	)));
	io.extend_with(SyncStateRpcApi::to_delegate(SyncStateRpcHandler::new(
		chain_spec,
		client.clone(),
		shared_authority_set,
		shared_epoch_changes,
		deny_unsafe,
	)?));
	io.extend_with(BalancesApi::to_delegate(Balances::new(client.clone())));
	io.extend_with(FeeMarketApi::to_delegate(FeeMarket::new(client.clone())));
	io.extend_with(StakingApi::to_delegate(Staking::new(client)));

	Ok(io)
}
