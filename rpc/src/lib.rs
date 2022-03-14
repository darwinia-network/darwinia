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

// #![warn(missing_docs)]

pub mod crab;
pub mod darwinia;

pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};

// --- std ---
use std::{error::Error, str::FromStr, sync::Arc};
// --- darwinia-network ---
use darwinia_common_primitives::{BlockNumber, Hash, OpaqueBlock as Block};
use dc_rpc::{CacheRequester as TraceFilterCacheRequester, DebugRequester};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;
/// RPC result.
pub type RpcResult = Result<RpcExtension, Box<dyn Error + Send + Sync>>;

/// Extra dependencies for BABE.
pub struct BabeDeps {
	/// BABE protocol config.
	pub babe_config: sc_consensus_babe::Config,
	/// BABE pending epoch changes.
	pub shared_epoch_changes:
		sc_consensus_epochs::SharedEpochChanges<Block, sc_consensus_babe::Epoch>,
	/// The keystore that manages the keys of the node.
	pub keystore: sp_keystore::SyncCryptoStorePtr,
}

/// Dependencies for GRANDPA
pub struct GrandpaDeps<B> {
	/// Voting round info.
	pub shared_voter_state: sc_finality_grandpa::SharedVoterState,
	/// Authority set info.
	pub shared_authority_set: sc_finality_grandpa::SharedAuthoritySet<Hash, BlockNumber>,
	/// Receives notifications about justification events from Grandpa.
	pub justification_stream: sc_finality_grandpa::GrandpaJustificationStream<Block>,
	/// Executor to drive the subscription manager in the Grandpa RPC handler.
	pub subscription_executor: sc_rpc::SubscriptionTaskExecutor,
	/// Finality proof provider.
	pub finality_provider: Arc<sc_finality_grandpa::FinalityProofProvider<B, Block>>,
}

#[derive(Clone)]
pub struct RpcRequesters {
	pub debug: Option<DebugRequester>,
	pub trace: Option<TraceFilterCacheRequester>,
}

/// Ethereum RPC configurations.
#[derive(Clone, Debug, PartialEq)]
pub struct EthRpcConfig {
	pub ethapi: Vec<EthApiCmd>,
	pub ethapi_max_permits: u32,
	pub ethapi_trace_max_count: u32,
	pub ethapi_trace_cache_duration: u64,
	pub eth_log_block_cache: usize,
	pub max_past_logs: u32,
}

#[derive(Clone)]
pub struct EthRpcRequesters {
	pub debug: Option<dc_rpc::DebugRequester>,
	pub trace: Option<dc_rpc::CacheRequester>,
}

/// Ethereum RPC Commands.
#[derive(Clone, Debug, PartialEq)]
pub enum EthApiCmd {
	Debug,
	Trace,
}
impl FromStr for EthApiCmd {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"debug" => Self::Debug,
			"trace" => Self::Trace,
			_ => {
				return Err(format!(
					"`{}` is not recognized as a supported Ethereum Api",
					s
				))
			}
		})
	}
}
