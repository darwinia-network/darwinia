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

//! Darwinia CLI library.

// --- crates.io ---
use structopt::StructOpt;
// --- paritytech ---
#[cfg(feature = "runtime-benchmarks")]
use frame_benchmarking_cli::BenchmarkCmd;
use sc_cli::*;
#[cfg(feature = "try-runtime")]
use try_runtime_cli::TryRuntimeCmd;
// --- darwinia-network ---
use darwinia_rpc::EthRpcConfig;

#[allow(missing_docs)]
#[derive(Debug, StructOpt)]
pub struct Cli {
	#[allow(missing_docs)]
	#[structopt(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub run: Run,
}

#[allow(missing_docs)]
#[derive(Debug, StructOpt)]
pub enum Subcommand {
	/// Build a chain specification.
	BuildSpec(BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(ExportStateCmd),

	/// Import blocks.
	ImportBlocks(ImportBlocksCmd),

	/// Remove the whole chain.
	PurgeChain(PurgeChainCmd),

	/// Revert the chain to a previous state.
	Revert(RevertCmd),

	/// Key management cli utilities
	Key(KeySubcommand),

	/// Verify a signature for a message, provided on STDIN, with a given (public or secret) key.
	Verify(VerifyCmd),

	/// Generate a seed that provides a vanity address.
	Vanity(VanityCmd),

	/// Sign a message, with a given (secret) key.
	Sign(SignCmd),

	/// Try some experimental command on the runtime. This includes migration and runtime-upgrade
	/// testing.
	#[cfg(feature = "try-runtime")]
	TryRuntime(TryRuntimeCmd),

	/// The custom benchmark subcommand benchmarking runtime pallets.
	#[cfg(feature = "runtime-benchmarks")]
	#[structopt(name = "benchmark", about = "Benchmark runtime pallets.")]
	Benchmark(BenchmarkCmd),
}

#[allow(missing_docs)]
#[derive(Debug, StructOpt)]
pub struct Run {
	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub base: RunCmd,

	/// Force using Crab native runtime.
	#[structopt(long = "force-crab")]
	pub force_crab: bool,

	/// Disable the authority discovery module on validator or sentry nodes.
	///
	/// Enabled by default on validator and sentry nodes. Always disabled on non
	/// validator or sentry nodes.
	///
	/// When enabled:
	///
	/// (1) As a validator node: Make oneself discoverable by publishing either
	///     ones own network addresses, or the ones of ones sentry nodes
	///     (configured via the `sentry-nodes` flag).
	///
	/// (2) As a validator or sentry node: Discover addresses of validators or
	///     addresses of their sentry nodes and maintain a permanent connection
	///     to a subset.
	#[structopt(long = "disable-authority-discovery")]
	pub authority_discovery_disabled: bool,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub dvm_args: DvmArgs,
}

#[derive(Debug, StructOpt)]
pub struct DvmArgs {
	/// Enable EVM tracing module on a non-authority node.
	#[structopt(long, conflicts_with = "validator", require_delimiter = true)]
	pub ethapi_debug_targets: Vec<String>,

	/// Number of concurrent tracing tasks. Meant to be shared by both "debug" and "trace" modules.
	#[structopt(long, default_value = "10")]
	pub ethapi_max_permits: u32,

	/// Maximum number of trace entries a single request of `trace_filter` is allowed to return.
	/// A request asking for more or an unbounded one going over this limit will both return an
	/// error.
	#[structopt(long, default_value = "500")]
	pub ethapi_trace_max_count: u32,

	/// Duration (in seconds) after which the cache of `trace_filter` for a given block will be
	/// discarded.
	#[structopt(long, default_value = "300")]
	pub ethapi_trace_cache_duration: u64,

	/// Size of the LRU cache for block data and their transaction statuses.
	#[structopt(long, default_value = "3000")]
	pub eth_log_block_cache: usize,

	/// Maximum number of logs in a query.
	#[structopt(long, default_value = "10000")]
	pub max_past_logs: u32,

	/// Maximum fee history cache size.
	#[structopt(long, default_value = "2048")]
	pub fee_history_limit: u64,
}
impl DvmArgs {
	pub fn build_eth_rpc_config(&self) -> EthRpcConfig {
		EthRpcConfig {
			ethapi_debug_targets: self.ethapi_debug_targets.clone(),
			ethapi_max_permits: self.ethapi_max_permits,
			ethapi_trace_max_count: self.ethapi_trace_max_count,
			ethapi_trace_cache_duration: self.ethapi_trace_cache_duration,
			eth_log_block_cache: self.eth_log_block_cache,
			max_past_logs: self.max_past_logs,
			fee_history_limit: self.fee_history_limit,
		}
	}
}
