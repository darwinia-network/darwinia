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
	($api:path,$($extra_apis:path),*,) => {
		/// A set of APIs that darwinia-like runtimes must implement.
		pub trait RuntimeApiCollection:
			$api
			$(+ $extra_apis)*
		{
		}
		impl<Api> RuntimeApiCollection for Api
		where
			Api: $api
				$(+ $extra_apis)*
		{
		}
	};
}

// --- darwinia-network ---
use darwinia_primitives::{OpaqueBlock as Block, *};

impl_runtime_apis![
	sp_api::ApiExt<Block>,
	sp_api::Metadata<Block>,
	sp_block_builder::BlockBuilder<Block>,
	sp_session::SessionKeys<Block>,
	sp_consensus_babe::BabeApi<Block>,
	sp_finality_grandpa::GrandpaApi<Block>,
	sp_authority_discovery::AuthorityDiscoveryApi<Block>,
	sp_offchain::OffchainWorkerApi<Block>,
	sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>,
	frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>,
	pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>,
	fp_rpc::EthereumRuntimeRPCApi<Block>,
	fp_rpc::ConvertTransactionRuntimeApi<Block>,
	moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block>,
];
