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
use std::sync::Arc;
// --- darwinia-network ---
use crate::*;
use darwinia_common_primitives::*;

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
	pub deny_unsafe: sc_rpc::DenyUnsafe,
	/// BABE specific dependencies.
	pub babe: BabeDeps,
	/// GRANDPA specific dependencies.
	pub grandpa: GrandpaDeps<B>,
	/// DVM related rpc helper.
	pub eth: EthDeps<A>,
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
		+ sc_client_api::BlockchainEvents<Block>
		+ sc_client_api::StorageProvider<Block, B>
		+ sp_blockchain::HeaderBackend<Block>
		+ sp_blockchain::HeaderMetadata<Block, Error = sp_blockchain::Error>,
	C::Api: sp_block_builder::BlockBuilder<Block>
		+ sc_consensus_babe::BabeApi<Block>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ darwinia_balances_rpc::BalancesRuntimeApi<Block, AccountId, Balance>
		+ darwinia_fee_market_rpc::FeeMarketRuntimeApi<Block, Balance>
		+ darwinia_staking_rpc::StakingRuntimeApi<Block, AccountId, Power>
		+ dp_evm_trace_apis::DebugRuntimeApi<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
		+ fp_rpc::ConvertTransactionRuntimeApi<Block>,
	P: 'static + Sync + Send + sc_transaction_pool_api::TransactionPool<Block = Block>,
	SC: 'static + sp_consensus::SelectChain<Block>,
	B: 'static + Send + Sync + sc_client_api::Backend<Block>,
	B::State: sc_client_api::StateBackend<sp_runtime::traits::HashFor<Block>>,
	A: 'static + sc_transaction_pool::ChainApi<Block = Block>,
{
	// --- std ---
	use std::collections::BTreeMap;
	// --- crates.io ---
	use jsonrpc_core::IoHandler;
	use jsonrpc_pubsub::manager::SubscriptionManager;
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
	use dc_rpc::*;
	use dvm_ethereum::EthereumStorageSchema;

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
		eth:
			EthDeps {
				config:
					EthRpcConfig {
						ethapi_debug_targets,
						ethapi_trace_max_count,
						max_past_logs,
						fee_history_limit,
						..
					},
				graph,
				is_authority,
				network,
				filter_pool,
				backend,
				fee_history_cache,
				overrides,
				block_data_cache,
				rpc_requesters,
			},
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
	io.extend_with(StakingApi::to_delegate(Staking::new(client.clone())));

	let convert_transaction: Option<NoTransactionConverter> = None;
	io.extend_with(EthApiServer::to_delegate(EthApi::new(
		client.clone(),
		pool.clone(),
		graph,
		<Option<NoTransactionConverter>>::None,
		network.clone(),
		vec![],
		overrides.clone(),
		backend.clone(),
		is_authority,
		max_past_logs,
		block_data_cache.clone(),
		fee_history_limit,
		fee_history_cache,
	)));
	if let Some(filter_pool) = filter_pool {
		io.extend_with(EthFilterApiServer::to_delegate(EthFilterApi::new(
			client.clone(),
			backend,
			filter_pool.clone(),
			500 as usize, // max stored filters
			max_past_logs,
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

	if ethapi_debug_targets
		.iter()
		.any(|cmd| matches!(cmd.as_str(), "debug" | "trace"))
	{
		if let Some(trace_filter_requester) = rpc_requesters.trace {
			io.extend_with(TraceApiServer::to_delegate(Trace::new(
				client,
				trace_filter_requester,
				ethapi_trace_max_count,
			)));
		}

		if let Some(debug_requester) = rpc_requesters.debug {
			io.extend_with(DebugApiServer::to_delegate(Debug::new(debug_requester)));
		}
	}

	Ok(io)
}

pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<fc_rpc::OverrideHandle<Block>>
where
	C: 'static
		+ Send
		+ Sync
		+ sc_client_api::backend::AuxStore
		+ sc_client_api::backend::StorageProvider<Block, BE>
		+ sp_api::ProvideRuntimeApi<Block>
		+ sp_blockchain::HeaderBackend<Block>
		+ sp_blockchain::HeaderMetadata<Block, Error = sp_blockchain::Error>,
	C::Api: sp_api::ApiExt<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
		+ fp_rpc::ConvertTransactionRuntimeApi<Block>,
	BE: 'static + sc_client_api::backend::Backend<Block>,
	BE::State: sc_client_api::backend::StateBackend<Hashing>,
{
	// --- std ---
	use std::collections::BTreeMap;
	// --- paritytech ---
	use fc_rpc::*;
	use fp_storage::EthereumStorageSchema;

	Arc::new(OverrideHandle {
		schemas: BTreeMap::from_iter([
			(
				EthereumStorageSchema::V1,
				Box::new(SchemaV1Override::new(client.clone()))
					as Box<dyn StorageOverride<_> + Send + Sync>,
			),
			(
				EthereumStorageSchema::V2,
				Box::new(SchemaV2Override::new(client.clone()))
					as Box<dyn StorageOverride<_> + Send + Sync>,
			),
			(
				EthereumStorageSchema::V3,
				Box::new(SchemaV3Override::new(client.clone()))
					as Box<dyn StorageOverride<_> + Send + Sync>,
			),
		]),
		fallback: Box::new(RuntimeApiStorageOverride::new(client)),
	})
}
