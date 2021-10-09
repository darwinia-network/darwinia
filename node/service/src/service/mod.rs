// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
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
			$(+ $extra_apis),*
		where
			<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
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
				$(+ $extra_apis),*,
			<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
		{
		}
	};
}

pub mod crab;
pub use crab::CrabExecutor;
pub use crab_runtime;

pub mod darwinia;
pub use darwinia::DarwiniaExecutor;
pub use darwinia_runtime;

// --- std ---
use std::sync::Arc;
// --- crates.io ---
use codec::Codec;
// --- paritytech ---
use sc_consensus::LongestChain;
use sc_finality_grandpa::GrandpaBlockImport;
use sc_keystore::LocalKeystore;
use sc_service::{
	config::PrometheusConfig, ChainSpec, Configuration, Error as ServiceError, TFullBackend,
	TFullClient, TLightBackendWithHash, TLightClientWithBackend,
};
use sp_runtime::traits::BlakeTwo256;
use substrate_prometheus_endpoint::Registry;
// --- darwinia-network ---
use darwinia_primitives::OpaqueBlock as Block;

type FullBackend = TFullBackend<Block>;
type FullSelectChain = LongestChain<FullBackend, Block>;
type FullClient<RuntimeApi, Executor> = TFullClient<Block, RuntimeApi, Executor>;
type FullGrandpaBlockImport<RuntimeApi, Executor> =
	GrandpaBlockImport<FullBackend, Block, FullClient<RuntimeApi, Executor>, FullSelectChain>;
type LightBackend = TLightBackendWithHash<Block, BlakeTwo256>;
type LightClient<RuntimeApi, Executor> =
	TLightClientWithBackend<Block, RuntimeApi, Executor, LightBackend>;

pub trait RuntimeExtrinsic: 'static + Send + Sync + Codec {}
impl<E> RuntimeExtrinsic for E where E: 'static + Send + Sync + Codec {}

/// Can be called for a `Configuration` to check if it is a configuration for the `Crab` network.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Crab` network.
	fn is_crab(&self) -> bool;

	/// Returns true if this configuration is for a development network.
	fn is_dev(&self) -> bool;
}
impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_crab(&self) -> bool {
		self.id().starts_with("crab")
	}

	fn is_dev(&self) -> bool {
		self.id().ends_with("dev")
	}
}

// If we're using prometheus, use a registry with a prefix of `darwinia`.
fn set_prometheus_registry(config: &mut Configuration) -> Result<(), ServiceError> {
	if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
		*registry = Registry::new_custom(Some("darwinia".into()), None)?;
	}

	Ok(())
}

fn remote_keystore(_url: &String) -> Result<Arc<LocalKeystore>, &'static str> {
	// FIXME: here would the concrete keystore be built,
	//        must return a concrete type (NOT `LocalKeystore`) that
	//        implements `CryptoStore` and `SyncCryptoStore`
	Err("Remote Keystore not supported.")
}
