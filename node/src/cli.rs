// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
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

/// The version of the node.
///
/// This is the version that is used for versioning this node binary.
/// By default the `minor` version is bumped in every release. `Major` or `patch` releases are only
/// expected in very rare cases.
///
/// The worker binaries associated to the node binary should ensure that they are using the same
/// version as the main node that started them.
pub const NODE_VERSION: &str = "6.7.0";

/// Sub-commands supported by the collator.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Build a chain specification.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// Remove the whole chain.
	PurgeChain(cumulus_client_cli::PurgeChainCmd),

	/// Export the genesis state of the parachain.
	ExportGenesisHead(cumulus_client_cli::ExportGenesisHeadCommand),

	/// Export the genesis wasm of the parachain.
	ExportGenesisWasm(cumulus_client_cli::ExportGenesisWasmCommand),

	/// Sub-commands concerned with benchmarking.
	/// The pallet benchmarking moved to the `pallet` sub-command.
	#[cfg(feature = "runtime-benchmarks")]
	#[command(subcommand)]
	Benchmark(Box<frame_benchmarking_cli::BenchmarkCmd>),

	/// Errors since the binary was not build with `--features runtime-benchmarks`.
	#[cfg(not(feature = "runtime-benchmarks"))]
	Benchmark,
}

#[derive(Debug, clap::Parser)]
#[command(
	propagate_version = true,
	args_conflicts_with_subcommands = true,
	subcommand_negates_reqs = true
)]
pub struct Cli {
	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[command(flatten)]
	pub run: cumulus_client_cli::RunCmd,

	/// Disable automatic hardware benchmarks.
	///
	/// By default these benchmarks are automatically ran at startup and measure
	/// the CPU speed, the memory bandwidth and the disk speed.
	///
	/// The results are then printed out in the logs, and also sent as part of
	/// telemetry, if telemetry is enabled.
	#[arg(long)]
	pub no_hardware_benchmarks: bool,

	#[clap(flatten)]
	pub storage_monitor: sc_storage_monitor::StorageMonitorParams,

	/// Relay chain arguments
	#[arg(raw = true)]
	pub relay_chain_args: Vec<String>,

	#[command(flatten)]
	pub eth_args: EthArgs,
}

#[derive(Debug)]
pub struct RelayChainCli {
	/// The actual relay chain cli object.
	pub base: polkadot_cli::RunCmd,

	/// Optional chain id that should be passed to the relay chain.
	pub chain_id: Option<String>,

	/// The base path that should be used by the relay chain.
	pub base_path: Option<std::path::PathBuf>,
}
impl RelayChainCli {
	/// Parse the relay chain CLI parameters using the para chain `Configuration`.
	pub fn new<'a>(
		para_config: &sc_service::Configuration,
		relay_chain_args: impl Iterator<Item = &'a String>,
	) -> Self {
		let extension = crate::chain_spec::Extensions::try_get(&*para_config.chain_spec);
		let chain_id = extension.map(|e| e.relay_chain.clone());
		let base_path = para_config.base_path.path().join("polkadot");

		Self {
			base_path: Some(base_path),
			chain_id,
			base: clap::Parser::parse_from(relay_chain_args),
		}
	}
}

/// Available frontier backend types.
#[derive(Copy, Clone, Debug, Default, clap::ValueEnum)]
pub enum FrontierBackendType {
	/// Either RocksDb or ParityDb as per inherited from the global backend settings.
	#[default]
	KeyValue,
	/// Sql database with custom log indexing.
	Sql,
}

#[derive(Debug, clap::Parser)]
pub struct EthArgs {
	/// Enable EVM tracing functionalities.
	#[arg(long, value_delimiter = ',')]
	pub tracing_api: Vec<TracingApi>,

	/// Number of concurrent tracing tasks. Meant to be shared by both "debug" and "trace" modules.
	#[arg(long, default_value = "10")]
	pub tracing_max_permits: u32,

	/// Maximum number of trace entries a single request of `trace_filter` is allowed to return.
	/// A request asking for more or an unbounded one going over this limit will both return an
	/// error.
	#[arg(long, default_value = "500")]
	pub tracing_max_count: u32,

	// Duration (in seconds) after which the cache of `trace_filter` for a given block will be
	/// discarded.
	#[arg(long, default_value = "300")]
	pub tracing_cache_duration: u64,

	/// Size in bytes of data a raw tracing request is allowed to use.
	/// Bound the size of memory, stack and storage data.
	#[arg(long, default_value = "20000000")]
	pub tracing_raw_max_memory_usage: usize,

	/// Size in bytes of the LRU cache for block data.
	#[arg(long, default_value = "300000000")]
	pub eth_log_block_cache: usize,

	/// Size of the LRU cache for block data and their transaction statuses.
	#[arg(long, default_value = "300000000")]
	pub eth_statuses_cache: usize,

	/// Maximum number of logs in a query.
	#[arg(long, default_value = "10000")]
	pub max_past_logs: u32,

	/// Maximum fee history cache size.
	#[arg(long, default_value = "2048")]
	pub fee_history_limit: u64,

	/// Sets the frontier backend type (KeyValue or Sql)
	#[arg(long, value_enum, ignore_case = true, default_value_t = FrontierBackendType::default())]
	pub frontier_backend_type: FrontierBackendType,

	// Sets the SQL backend's pool size.
	#[arg(long, default_value = "100")]
	pub frontier_sql_backend_pool_size: u32,

	/// Sets the SQL backend's query timeout in number of VM ops.
	#[arg(long, default_value = "10000000")]
	pub frontier_sql_backend_num_ops_timeout: u32,

	/// Sets the SQL backend's auxiliary thread limit.
	#[arg(long, default_value = "4")]
	pub frontier_sql_backend_thread_count: u32,

	/// Sets the SQL backend's query timeout in number of VM ops.
	/// Default value is 200MB.
	#[arg(long, default_value = "209715200")]
	pub frontier_sql_backend_cache_size: u64,
}
impl EthArgs {
	pub fn build_eth_rpc_config(&self) -> EthRpcConfig {
		EthRpcConfig {
			tracing_api: self.tracing_api.clone(),
			tracing_max_permits: self.tracing_max_permits,
			tracing_max_count: self.tracing_max_permits,
			tracing_cache_duration: self.tracing_cache_duration,
			tracing_raw_max_memory_usage: self.tracing_raw_max_memory_usage,
			eth_statuses_cache: self.eth_statuses_cache,
			eth_log_block_cache: self.eth_log_block_cache,
			max_past_logs: self.max_past_logs,
			fee_history_limit: self.fee_history_limit,
			frontier_backend_type: self.frontier_backend_type,
			frontier_sql_backend_pool_size: self.frontier_sql_backend_pool_size,
			frontier_sql_backend_num_ops_timeout: self.frontier_sql_backend_num_ops_timeout,
			frontier_sql_backend_thread_count: self.frontier_sql_backend_thread_count,
			frontier_sql_backend_cache_size: self.frontier_sql_backend_cache_size,
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum TracingApi {
	Debug,
	Trace,
}
impl std::str::FromStr for TracingApi {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"debug" => Self::Debug,
			"trace" => Self::Trace,
			_ => return Err(format!("`{}` is not recognized as a supported Ethereum Api", s)),
		})
	}
}

#[derive(Clone, Debug, Default)]
pub struct EthRpcConfig {
	pub tracing_api: Vec<TracingApi>,
	pub tracing_max_permits: u32,
	pub tracing_max_count: u32,
	pub tracing_cache_duration: u64,
	pub tracing_raw_max_memory_usage: usize,
	pub eth_log_block_cache: usize,
	pub eth_statuses_cache: usize,
	pub fee_history_limit: u64,
	pub max_past_logs: u32,
	pub frontier_backend_type: FrontierBackendType,
	pub frontier_sql_backend_pool_size: u32,
	pub frontier_sql_backend_num_ops_timeout: u32,
	pub frontier_sql_backend_thread_count: u32,
	pub frontier_sql_backend_cache_size: u64,
}
