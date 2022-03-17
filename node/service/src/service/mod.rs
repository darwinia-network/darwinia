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

macro_rules! impl_runtime_apis {
	($($extra_apis:path),*) => {
		/// A set of APIs that darwinia-like runtimes must implement.
		pub trait RuntimeApiCollection:
			sp_api::ApiExt<Block>
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
			$(+ $extra_apis)*
		where
			<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<Hashing>,
		{
		}
		impl<Api> RuntimeApiCollection for Api
		where
			Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
				+ sp_api::ApiExt<Block>
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
				$(+ $extra_apis)*,
			<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<Hashing>,
		{
		}
	};
}

pub mod crab;
pub use crab::Executor as CrabExecutor;

pub mod darwinia;
pub use darwinia::Executor as DarwiniaExecutor;

pub mod dvm;

// --- darwinia-network ---
use darwinia_common_primitives::OpaqueBlock as Block;

type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FullClient<RuntimeApi, Executor> =
	sc_service::TFullClient<Block, RuntimeApi, sc_executor::NativeElseWasmExecutor<Executor>>;
type FullGrandpaBlockImport<RuntimeApi, Executor> = sc_finality_grandpa::GrandpaBlockImport<
	FullBackend,
	Block,
	FullClient<RuntimeApi, Executor>,
	FullSelectChain,
>;

type ServiceResult<T> = Result<T, sc_service::Error>;
type RpcServiceResult = Result<darwinia_rpc::RpcExtension, sc_service::Error>;

/// Can be called for a `Configuration` to check if it is a configuration for the `Crab` network.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Crab` network.
	fn is_crab(&self) -> bool;

	/// Returns true if this configuration is for a development network.
	fn is_dev(&self) -> bool;
}
impl IdentifyVariant for Box<dyn sc_service::ChainSpec> {
	fn is_crab(&self) -> bool {
		self.id().starts_with("crab")
	}

	fn is_dev(&self) -> bool {
		self.id().ends_with("dev")
	}
}

// If we're using prometheus, use a registry with a prefix of `darwinia`.
fn set_prometheus_registry(config: &mut sc_service::Configuration) -> ServiceResult<()> {
	// --- paritytech ---
	use sc_service::config::PrometheusConfig;
	use substrate_prometheus_endpoint::Registry;

	if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
		*registry = Registry::new_custom(Some("darwinia".into()), None)?;
	}

	Ok(())
}
