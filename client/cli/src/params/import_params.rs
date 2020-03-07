use sc_service::{config::DatabaseConfig, Configuration, RuntimeGenesis};
use serde::Deserialize;
use structopt::StructOpt;

use crate::arg_enums::{
	ExecutionStrategy, TracingReceiver, WasmExecutionMethod, DEFAULT_EXECUTION_BLOCK_CONSTRUCTION,
	DEFAULT_EXECUTION_IMPORT_BLOCK, DEFAULT_EXECUTION_OFFCHAIN_WORKER, DEFAULT_EXECUTION_OTHER,
	DEFAULT_EXECUTION_SYNCING,
};
use crate::error;
use crate::params::PruningParams;

/// Parameters for block import.
#[derive(Clone, Debug, Deserialize, StructOpt)]
#[serde(default, rename_all = "kebab-case")]
pub struct ImportParams {
	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub pruning_params: PruningParams,

	/// Force start with unsafe pruning settings.
	///
	/// When running as a validator it is highly recommended to disable state
	/// pruning (i.e. 'archive') which is the default. The node will refuse to
	/// start as a validator if pruning is enabled unless this option is set.
	#[structopt(long = "unsafe-pruning")]
	pub unsafe_pruning: bool,

	/// Method for executing Wasm runtime code.
	#[structopt(
		long = "wasm-execution",
		value_name = "METHOD",
		possible_values = &WasmExecutionMethod::enabled_variants(),
		case_insensitive = true,
		default_value = "Interpreted"
	)]
	pub wasm_method: WasmExecutionMethod,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub execution_strategies: ExecutionStrategies,

	/// Limit the memory the database cache can use.
	#[serde(rename = "db-cache")]
	#[structopt(long = "db-cache", value_name = "MiB", default_value = "1024")]
	pub database_cache_size: u32,

	/// Specify the state cache size.
	#[structopt(long = "state-cache-size", value_name = "Bytes", default_value = "67108864")]
	pub state_cache_size: usize,

	/// Comma separated list of targets for tracing
	#[structopt(long = "tracing-targets", value_name = "TARGETS")]
	pub tracing_targets: Option<String>,

	/// Receiver to process tracing messages
	#[structopt(
		long = "tracing-receiver",
		value_name = "RECEIVER",
		possible_values = &TracingReceiver::variants(),
		case_insensitive = true,
		default_value = "Log"
	)]
	pub tracing_receiver: TracingReceiver,
}

impl ImportParams {
	/// Put block import CLI params into `config` object.
	pub fn update_config<G, E>(
		&self,
		mut config: &mut Configuration<G, E>,
		role: sc_service::Roles,
		is_dev: bool,
	) -> error::Result<()>
	where
		G: RuntimeGenesis,
	{
		use sc_client_api::execution_extensions::ExecutionStrategies;

		if let Some(DatabaseConfig::Path { ref mut cache_size, .. }) = config.database {
			*cache_size = Some(self.database_cache_size);
		}

		config.state_cache_size = self.state_cache_size;

		self.pruning_params
			.update_config(&mut config, role, self.unsafe_pruning)?;

		config.wasm_method = self.wasm_method.into();

		let exec = &self.execution_strategies;
		let exec_all_or = |strat: ExecutionStrategy, default: ExecutionStrategy| {
			exec.execution
				.unwrap_or(if strat == default && is_dev {
					ExecutionStrategy::Native
				} else {
					strat
				})
				.into()
		};

		config.execution_strategies = ExecutionStrategies {
			syncing: exec_all_or(exec.execution_syncing, DEFAULT_EXECUTION_SYNCING),
			importing: exec_all_or(exec.execution_import_block, DEFAULT_EXECUTION_IMPORT_BLOCK),
			block_construction: exec_all_or(exec.execution_block_construction, DEFAULT_EXECUTION_BLOCK_CONSTRUCTION),
			offchain_worker: exec_all_or(exec.execution_offchain_worker, DEFAULT_EXECUTION_OFFCHAIN_WORKER),
			other: exec_all_or(exec.execution_other, DEFAULT_EXECUTION_OTHER),
		};

		Ok(())
	}
}

impl Default for ImportParams {
	fn default() -> Self {
		Self {
			pruning_params: Default::default(),
			unsafe_pruning: false,
			wasm_method: WasmExecutionMethod::Interpreted,
			execution_strategies: Default::default(),
			database_cache_size: 1024,
			state_cache_size: 67108864,
			tracing_targets: None,
			tracing_receiver: TracingReceiver::Log,
		}
	}
}

/// Execution strategies parameters.
#[derive(Clone, Debug, Deserialize, StructOpt)]
pub struct ExecutionStrategies {
	/// The means of execution used when calling into the runtime while syncing blocks.
	#[structopt(
		long = "execution-syncing",
		value_name = "STRATEGY",
		possible_values = &ExecutionStrategy::variants(),
		case_insensitive = true,
		default_value = DEFAULT_EXECUTION_SYNCING.as_str(),
	)]
	pub execution_syncing: ExecutionStrategy,

	/// The means of execution used when calling into the runtime while importing blocks.
	#[structopt(
		long = "execution-import-block",
		value_name = "STRATEGY",
		possible_values = &ExecutionStrategy::variants(),
		case_insensitive = true,
		default_value = DEFAULT_EXECUTION_IMPORT_BLOCK.as_str(),
	)]
	pub execution_import_block: ExecutionStrategy,

	/// The means of execution used when calling into the runtime while constructing blocks.
	#[structopt(
		long = "execution-block-construction",
		value_name = "STRATEGY",
		possible_values = &ExecutionStrategy::variants(),
		case_insensitive = true,
		default_value = DEFAULT_EXECUTION_BLOCK_CONSTRUCTION.as_str(),
	)]
	pub execution_block_construction: ExecutionStrategy,

	/// The means of execution used when calling into the runtime while using an off-chain worker.
	#[structopt(
		long = "execution-offchain-worker",
		value_name = "STRATEGY",
		possible_values = &ExecutionStrategy::variants(),
		case_insensitive = true,
		default_value = DEFAULT_EXECUTION_OFFCHAIN_WORKER.as_str(),
	)]
	pub execution_offchain_worker: ExecutionStrategy,

	/// The means of execution used when calling into the runtime while not syncing, importing or constructing blocks.
	#[structopt(
		long = "execution-other",
		value_name = "STRATEGY",
		possible_values = &ExecutionStrategy::variants(),
		case_insensitive = true,
		default_value = DEFAULT_EXECUTION_OTHER.as_str(),
	)]
	pub execution_other: ExecutionStrategy,

	/// The execution strategy that should be used by all execution contexts.
	#[structopt(
		long = "execution",
		value_name = "STRATEGY",
		possible_values = &ExecutionStrategy::variants(),
		case_insensitive = true,
		conflicts_with_all = &[
			"execution-other",
			"execution-offchain-worker",
			"execution-block-construction",
			"execution-import-block",
			"execution-syncing",
		]
	)]
	pub execution: Option<ExecutionStrategy>,
}

impl Default for ExecutionStrategies {
	fn default() -> Self {
		Self {
			execution_syncing: ExecutionStrategy::NativeElseWasm,
			execution_import_block: ExecutionStrategy::NativeElseWasm,
			execution_block_construction: ExecutionStrategy::Wasm,
			execution_offchain_worker: ExecutionStrategy::Native,
			execution_other: ExecutionStrategy::Native,
			execution: None,
		}
	}
}
