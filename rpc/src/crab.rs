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

//! Crab-specific RPCs implementation.

// --- std ---
use std::{collections::BTreeMap, sync::Arc};
// --- crates.io ---
use jsonrpc_core::IoHandler;
// --- parity-tech ---
use sc_rpc::SubscriptionTaskExecutor;
// --- darwinia-network ---
use crate::*;
use darwinia_common_primitives::*;
use dvm_ethereum::EthereumStorageSchema;

/// Full client dependencies
pub struct FullDeps<C, P, SC, B, A>
where
	A: sc_transaction_pool::ChainApi,
{
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// The SelectChain Strategy
	pub select_chain: SC,
	/// A copy of the chain spec.
	pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// BABE specific dependencies.
	pub babe: BabeDeps,
	/// GRANDPA specific dependencies.
	pub grandpa: GrandpaDeps<B>,
	/// The Node authority flag
	pub is_authority: bool,
	/// Network service
	pub network: Arc<sc_network::NetworkService<Block, Hash>>,
	/// EthFilterApi pool.
	pub filter_pool: Option<fc_rpc_core::types::FilterPool>,
	/// Backend.
	pub backend: Arc<dc_db::Backend<Block>>,
	/// Graph pool instance.
	pub graph: Arc<sc_transaction_pool::Pool<A>>,
	/// Rpc requester for evm trace
	pub tracing_requesters: EthRpcRequesters,
	/// Ethereum RPC Config.
	pub eth_rpc_config: EthRpcConfig,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, SC, B, A>(
	deps: FullDeps<C, P, SC, B, A>,
	subscription_task_executor: SubscriptionTaskExecutor,
) -> RpcResult
where
	C: 'static
		+ Send
		+ Sync
		+ sp_api::ProvideRuntimeApi<Block>
		+ sc_client_api::AuxStore
		// <--- dvm ---
		+ sc_client_api::BlockchainEvents<Block>
		+ sc_client_api::StorageProvider<Block, B>
		// --- dvm --->
		+ sp_blockchain::HeaderBackend<Block>
		+ sp_blockchain::HeaderMetadata<Block, Error = sp_blockchain::Error>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: sc_consensus_babe::BabeApi<Block>,
	C::Api: sp_block_builder::BlockBuilder<Block>,
	C::Api: darwinia_balances_rpc::BalancesRuntimeApi<Block, AccountId, Balance>,
	C::Api: darwinia_fee_market_rpc::FeeMarketRuntimeApi<Block, Balance>,
	C::Api: darwinia_header_mmr_rpc::HeaderMMRRuntimeApi<Block, Hash>,
	C::Api: darwinia_staking_rpc::StakingRuntimeApi<Block, AccountId, Power>,
	C::Api: dp_evm_trace_apis::DebugRuntimeApi<Block>,
	C::Api: dvm_rpc_runtime_api::EthereumRuntimeRPCApi<Block>,
	P: 'static + Sync + Send + sc_transaction_pool_api::TransactionPool<Block = Block>,
	SC: 'static + sp_consensus::SelectChain<Block>,
	B: 'static + Send + Sync + sc_client_api::Backend<Block>,
	B::State: sc_client_api::StateBackend<sp_runtime::traits::HashFor<Block>>,
	A: 'static + sc_transaction_pool::ChainApi<Block = Block>,
{
	// --- crates.io ---
	use jsonrpc_pubsub::manager::SubscriptionManager;
	// --- paritytech ---
	use pallet_transaction_payment_rpc::*;
	use sc_consensus_babe_rpc::*;
	use sc_finality_grandpa_rpc::*;
	use sc_sync_state_rpc::*;
	use substrate_frame_rpc_system::*;
	// --- darwinia-network ---
	use crab_runtime::TransactionConverter;
	use darwinia_balances_rpc::*;
	use darwinia_fee_market_rpc::*;
	use darwinia_header_mmr_rpc::*;
	use darwinia_staking_rpc::*;
	use dc_rpc::*;

	let FullDeps {
		client,
		pool,
		select_chain,
		chain_spec,
		deny_unsafe,
		babe,
		grandpa,
		is_authority,
		network,
		filter_pool,
		backend,
		graph,
		tracing_requesters,
		eth_rpc_config,
	} = deps;
	let mut io = IoHandler::default();

	io.extend_with(SystemApi::to_delegate(FullSystem::new(
		client.clone(),
		pool.clone(),
		deny_unsafe,
	)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client.clone(),
	)));
	let BabeDeps {
		keystore,
		babe_config,
		shared_epoch_changes,
	} = babe;
	io.extend_with(BabeApi::to_delegate(BabeRpcHandler::new(
		client.clone(),
		shared_epoch_changes.clone(),
		keystore,
		babe_config,
		select_chain,
		deny_unsafe,
	)));
	let GrandpaDeps {
		shared_voter_state,
		shared_authority_set,
		justification_stream,
		subscription_executor,
		finality_provider,
	} = grandpa;
	io.extend_with(GrandpaApi::to_delegate(GrandpaRpcHandler::new(
		shared_authority_set.clone(),
		shared_voter_state,
		justification_stream,
		subscription_executor,
		finality_provider,
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
	io.extend_with(HeaderMMRApi::to_delegate(HeaderMMR::new(client.clone())));
	io.extend_with(StakingApi::to_delegate(Staking::new(client.clone())));

	let mut overrides_map = BTreeMap::new();
	overrides_map.insert(
		EthereumStorageSchema::V1,
		Box::new(SchemaV1Override::new(client.clone()))
			as Box<dyn StorageOverride<_> + Send + Sync>,
	);
	let overrides = Arc::new(OverrideHandle {
		schemas: overrides_map,
		fallback: Box::new(RuntimeApiStorageOverride::new(client.clone())),
	});
	let block_data_cache = Arc::new(EthBlockDataCache::new(50, 50));

	io.extend_with(EthApiServer::to_delegate(EthApi::new(
		client.clone(),
		pool.clone(),
		graph,
		TransactionConverter,
		network.clone(),
		overrides.clone(),
		backend.clone(),
		is_authority,
		vec![],
		eth_rpc_config.max_past_logs,
		block_data_cache.clone(),
	)));
	if let Some(filter_pool) = filter_pool {
		io.extend_with(EthFilterApiServer::to_delegate(EthFilterApi::new(
			client.clone(),
			backend,
			filter_pool.clone(),
			500 as usize, // max stored filters
			overrides.clone(),
			eth_rpc_config.max_past_logs,
			block_data_cache.clone(),
		)));
	}
	io.extend_with(EthPubSubApiServer::to_delegate(EthPubSubApi::new(
		pool,
		client.clone(),
		network.clone(),
		SubscriptionManager::<HexEncodedIdProvider>::with_id_provider(
			HexEncodedIdProvider::default(),
			Arc::new(subscription_task_executor),
		),
		overrides,
	)));
	io.extend_with(NetApiServer::to_delegate(NetApi::new(
		client.clone(),
		network,
		// Whether to format the `peer_count` response as Hex (default) or not.
		true,
	)));
	io.extend_with(Web3ApiServer::to_delegate(Web3Api::new(client.clone())));

	let ethapi_cmd = eth_rpc_config.ethapi.clone();
	if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace) {
		if let Some(trace_filter_requester) = tracing_requesters.trace {
			io.extend_with(TraceApiServer::to_delegate(Trace::new(
				client,
				trace_filter_requester,
				eth_rpc_config.ethapi_trace_max_count,
			)));
		}

		if let Some(debug_requester) = tracing_requesters.debug {
			io.extend_with(DebugApiServer::to_delegate(Debug::new(debug_requester)));
		}
	}

	Ok(io)
}
