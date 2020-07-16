// --- std ---
use std::{io::Read, path::PathBuf};
// --- crates ---
use serde::Deserialize;
// --- substrate ---
use sc_cli::{CliConfiguration, SubstrateCli};
// --- darwinia ---
use crate::cli::Cli;

/// Service configuration.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
	/// Enable validator mode.
	///
	/// The node will be started with the authority role and actively
	/// participate in any consensus task that it can (e.g. depending on
	/// availability of local keys).
	validator: Option<bool>,

	/// Enable sentry mode.
	///
	/// The node will be started with the authority role and participate in
	/// consensus tasks as an "observer", it will never actively participate
	/// regardless of whether it could (e.g. keys are available locally). This
	/// mode is useful as a secure proxy for validators (which would run
	/// detached from the network), since we want this node to participate in
	/// the full consensus protocols in order to have all needed consensus data
	/// available to relay to private nodes.
	// #[structopt(
	// long = "sentry",
	// conflicts_with_all = &[ "validator", "light" ],
	// parse(try_from_str)
	// )]
	sentry: Option<Vec<sc_service::config::MultiaddrWithPeerId>>,

	/// Disable GRANDPA voter when running in validator mode, otherwise disable the GRANDPA observer.
	no_grandpa: Option<bool>,

	/// Experimental: Run in light client mode.
	// #[structopt(long = "light", conflicts_with = "sentry")]
	light: Option<bool>,

	/// Listen to all RPC interfaces.
	///
	/// Default is local. Note: not all RPC methods are safe to be exposed publicly. Use an RPC proxy
	/// server to filter out dangerous methods. More details: https://github.com/paritytech/substrate/wiki/Public-RPC.
	/// Use `--unsafe-rpc-external` to suppress the warning if you understand the risks.
	rpc_external: Option<bool>,

	/// Listen to all RPC interfaces.
	///
	/// Same as `--rpc-external`.
	unsafe_rpc_external: Option<bool>,

	/// RPC methods to expose.
	///
	/// - `Unsafe`: Exposes every RPC method.
	/// - `Safe`: Exposes only a safe subset of RPC methods, denying unsafe RPC methods.
	/// - `Auto`: Acts as `Safe` if RPC is served externally, e.g. when `--{rpc,ws}-external` is passed,
	///   otherwise acts as `Unsafe`.
	rpc_methods: Option<RpcMethods>,

	/// Listen to all Websocket interfaces.
	///
	/// Default is local. Note: not all RPC methods are safe to be exposed publicly. Use an RPC proxy
	/// server to filter out dangerous methods. More details: https://github.com/paritytech/substrate/wiki/Public-RPC.
	/// Use `--unsafe-ws-external` to suppress the warning if you understand the risks.
	ws_external: Option<bool>,

	/// Listen to all Websocket interfaces.
	///
	/// Same as `--ws-external` but doesn't warn you about it.
	unsafe_ws_external: Option<bool>,

	/// Listen to all Prometheus data source interfaces.
	///
	/// Default is local.
	prometheus_external: Option<bool>,

	/// Specify HTTP RPC server TCP port.
	rpc_port: Option<u16>,

	/// Specify WebSockets RPC server TCP port.
	ws_port: Option<u16>,

	/// Maximum number of WS RPC server connections.
	ws_max_connections: Option<usize>,

	/// Specify browser Origins allowed to access the HTTP & WS RPC servers.
	///
	/// A comma-separated list of origins (protocol://domain or special `null`
	/// value). Value of `all` will disable origin validation. Default is to
	/// allow localhost and https://polkadot.js.org origins. When running in
	/// --dev mode the default is to allow all origins.
	rpc_cors: Option<String>,

	/// Specify Prometheus data source server TCP Port.
	prometheus_port: Option<u16>,

	/// Do not expose a Prometheus metric endpoint.
	///
	/// Prometheus metric endpoint is enabled by default.
	no_prometheus: Option<bool>,

	/// The human-readable name for this node.
	///
	/// The node name will be reported to the telemetry server, if enabled.
	name: Option<String>,

	/// Disable connecting to the Substrate telemetry server.
	///
	/// Telemetry is on by default on global chains.
	no_telemetry: Option<bool>,

	/// The URL of the telemetry server to connect to.
	///
	/// This flag can be passed multiple times as a means to specify multiple
	/// telemetry endpoints. Verbosity levels range from 0-9, with 0 denoting
	/// the least verbosity.
	/// Expected format is 'URL VERBOSITY', e.g. `--telemetry-url 'wss://foo/bar 0'`.
	#[serde(rename = "telemetry-url")]
	telemetry_endpoints: Option<Vec<String>>,

	#[allow(missing_docs)]
	#[serde(flatten)]
	offchain_worker_config: OffchainWorkerConfig,

	#[allow(missing_docs)]
	#[serde(flatten)]
	shared_config: SharedConfig,

	#[allow(missing_docs)]
	#[serde(flatten)]
	import_config: ImportConfig,
	//
	// #[allow(missing_docs)]
	// #[serde(flatten)]
	// network_config: NetworkConfig,
	//
	// #[allow(missing_docs)]
	// #[structopt(flatten)]
	// pub pool_config: TransactionPoolParams,
	//
	/// Shortcut for `--name Alice --validator` with session keys for `Alice` added to keystore.
	// #[structopt(long, conflicts_with_all = &["bob", "charlie", "dave", "eve", "ferdie", "one", "two"])]
	alice: Option<bool>,

	/// Shortcut for `--name Bob --validator` with session keys for `Bob` added to keystore.
	// #[structopt(long, conflicts_with_all = &["alice", "charlie", "dave", "eve", "ferdie", "one", "two"])]
	bob: Option<bool>,

	/// Shortcut for `--name Charlie --validator` with session keys for `Charlie` added to keystore.
	// #[structopt(long, conflicts_with_all = &["alice", "bob", "dave", "eve", "ferdie", "one", "two"])]
	charlie: Option<bool>,

	/// Shortcut for `--name Dave --validator` with session keys for `Dave` added to keystore.
	// #[structopt(long, conflicts_with_all = &["alice", "bob", "charlie", "eve", "ferdie", "one", "two"])]
	dave: Option<bool>,

	/// Shortcut for `--name Eve --validator` with session keys for `Eve` added to keystore.
	// #[structopt(long, conflicts_with_all = &["alice", "bob", "charlie", "dave", "ferdie", "one", "two"])]
	eve: Option<bool>,

	/// Shortcut for `--name Ferdie --validator` with session keys for `Ferdie` added to keystore.
	// #[structopt(long, conflicts_with_all = &["alice", "bob", "charlie", "dave", "eve", "one", "two"])]
	ferdie: Option<bool>,

	/// Shortcut for `--name One --validator` with session keys for `One` added to keystore.
	// #[structopt(long, conflicts_with_all = &["alice", "bob", "charlie", "dave", "eve", "ferdie", "two"])]
	one: Option<bool>,

	/// Shortcut for `--name Two --validator` with session keys for `Two` added to keystore.
	// #[structopt(long, conflicts_with_all = &["alice", "bob", "charlie", "dave", "eve", "ferdie", "one"])]
	two: Option<bool>,

	/// Enable authoring even when offline.
	force_authoring: Option<bool>,
	//
	// #[allow(missing_docs)]
	// #[structopt(flatten)]
	// pub keystore_params: KeystoreParams,
	//
	/// The size of the instances cache for each runtime.
	///
	/// The default value is 8 and the values higher than 256 are ignored.
	max_runtime_instances: Option<usize>,

	/// Specify a list of sentry node public addresses.
	///
	/// Can't be used with --public-addr as the sentry node would take precedence over the public address
	/// specified there.
	// #[structopt(
	// long = "sentry-nodes",
	// value_name = "ADDR",
	// conflicts_with_all = &[ "sentry", "public-addr" ]
	// )]
	sentry_nodes: Option<Vec<sc_service::config::MultiaddrWithPeerId>>,
}
impl Configuration {
	pub fn create_runner_from_cli(cli: Cli) -> sc_cli::Result<sc_cli::Runner<Cli>> {
		if let Some(path) = &cli.conf {
			Ok(Self::load_config(path).update_config(cli)?)
		} else {
			Ok(cli.create_runner(&cli.run.base)?)
		}
	}

	fn load_config(path: &PathBuf) -> Self {
		let mut s = String::new();
		let mut f = std::fs::File::open(path).unwrap();
		f.read_to_string(&mut s).unwrap();

		toml::from_str(&s).unwrap()
	}

	fn update_config(self, mut cli: Cli) -> sc_cli::Result<sc_cli::Runner<Cli>> {
		macro_rules! quick_if_let {
			($($source:tt).+, $($target:tt).+, $field:ident) => {
				if let Some($field) = $($target).+.$field.clone() {
					$($source).+.$field = $field.into();
				}
			};
			($($source:tt).+, $($target:tt).+, Some($field:ident)) => {
				if $($target).+.$field.is_some() {
					$($source).+.$field = $($target).+.$field.clone().into();
				}
			};
			// ($($source:tt).+, $($target:ident$(($($params:expr),*))?).+, $field:ident) => {
			// 	if let Some($field) = $($target$(($($params),*))?).+ {
			// 		$($source).+.$field = $field.into();
			// 	}
			// };
		}

		{
			let cmd = &mut cli.run.base;

			quick_if_let!(cmd, self, validator);
			quick_if_let!(cmd, self, sentry);
			quick_if_let!(cmd, self, no_grandpa);
			quick_if_let!(cmd, self, light);
			quick_if_let!(cmd, self, rpc_external);
			quick_if_let!(cmd, self, unsafe_rpc_external);
			quick_if_let!(cmd, self, ws_external);
			quick_if_let!(cmd, self, unsafe_ws_external);
			quick_if_let!(cmd, self, prometheus_external);
			quick_if_let!(cmd, self, Some(rpc_port));
			quick_if_let!(cmd, self, Some(ws_port));
			quick_if_let!(cmd, self, Some(ws_max_connections));
			quick_if_let!(cmd, self, Some(prometheus_port));
			quick_if_let!(cmd, self, no_prometheus);
			quick_if_let!(cmd, self, Some(name));
			quick_if_let!(cmd, self, no_telemetry);
			if let Some(telemetry_endpoints) = &self.telemetry_endpoints {
				for telemetry_endpoint in telemetry_endpoints {
					cmd.telemetry_endpoints
						.push(parse_telemetry_endpoints(telemetry_endpoint));
				}
			}

			quick_if_let!(cmd.shared_params, self.shared_config, Some(chain));
			quick_if_let!(cmd.shared_params, self.shared_config, dev);
			quick_if_let!(cmd.shared_params, self.shared_config, Some(base_path));
			quick_if_let!(cmd.shared_params, self.shared_config, log);

			quick_if_let!(
				cmd.import_params.pruning_params,
				self.import_config.pruning_config,
				Some(pruning)
			);
			quick_if_let!(cmd.import_params, self.import_config, unsafe_pruning);
			quick_if_let!(cmd.import_params, self.import_config, state_cache_size);
			quick_if_let!(cmd.import_params, self.import_config, Some(tracing_targets));

			// cmd.network_params.bootnodes = self.network_config.bootnodes;
			// cmd.network_params.reserved_nodes = self.network_config.reserved_nodes;
			// cmd.network_params.reserved_only = self.network_config.reserved_only;
			// cmd.network_params.public_addr = self.network_config.public_addr;
			// cmd.network_params.listen_addr = self.network_config.listen_addr;
			// cmd.network_params.port = self.network_config.port;
			// cmd.network_params.no_private_ipv4 = self.network_config.no_private_ipv4;
			// cmd.network_params.out_peers = self.network_config.out_peers;
			// cmd.network_params.in_peers = self.network_config.in_peers;
			// cmd.network_params.no_mdns = self.network_config.no_mdns;
			// cmd.network_params.max_parallel_downloads = self.network_config.max_parallel_downloads;
			// cmd.network_params.no_yamux_flow_control = self.network_config.no_yamux_flow_control;
			// cmd.network_params.discover_local = self.network_config.discover_local;
			// cmd.network_params.legacy_network_protocol =
			// 	self.network_config.legacy_network_protocol;
			//
			quick_if_let!(cmd, self, alice);
			quick_if_let!(cmd, self, bob);
			quick_if_let!(cmd, self, charlie);
			quick_if_let!(cmd, self, dave);
			quick_if_let!(cmd, self, eve);
			quick_if_let!(cmd, self, ferdie);
			quick_if_let!(cmd, self, one);
			quick_if_let!(cmd, self, two);
			quick_if_let!(cmd, self, force_authoring);
			quick_if_let!(cmd, self, Some(max_runtime_instances));
			quick_if_let!(cmd, self, sentry_nodes);
		}

		let mut runner = cli.create_runner(&cli.run.base)?;
		{
			let config = runner.config_mut();
			let is_dev = cli.run.base.is_dev().unwrap();
			let role = config.role.clone();
			// let base_path = self
			// 	.base_path()
			// 	.unwrap()
			// 	.unwrap_or_else(|| {
			// 		app_dirs::get_app_root(
			// 			AppDataType::UserData,
			// 			&AppInfo {
			// 				name: cli.executable_name(),
			// 				author: cli.author(),
			// 			},
			// 		)
			// 		.expect("app directories exist on all supported platforms; qed")
			// 	})
			// 	.join("chains")
			// 	.join(chain_spec.id());

			// config.rpc_methods = self.rpc_methods.into();
			//

			config.rpc_cors = self
				.rpc_cors
				.map(|s| parse_cors(&s).into())
				.unwrap_or_else(|| {
					if is_dev {
						None
					} else {
						Some(vec![
							"http://localhost:*".into(),
							"http://127.0.0.1:*".into(),
							"https://localhost:*".into(),
							"https://127.0.0.1:*".into(),
							"https://polkadot.js.org".into(),
						])
					}
				});

			config.offchain_worker = self
				.offchain_worker_config
				.offchain_worker(&config.offchain_worker, &role);
			//
			// config.database = match self.import_config.database_config {
			// 	DatabaseConfig::RocksDb => sc_service::config::DatabaseConfig::RocksDb {
			// 		path: base_path.join("db"),
			// 		cache_size: self
			// 			.import_config
			// 			.database_config
			// 			.database_cache_size
			// 			.unwrap_or(128),
			// 	},
			// 	DatabaseConfig::SubDb => sc_service::config::DatabaseConfig::SubDb {
			// 		path: base_path.join("subdb"),
			// 	},
			// 	DatabaseConfig::ParityDb => sc_service::config::DatabaseConfig::ParityDb {
			// 		path: base_path.join("paritydb"),
			// 	},
			// };
			//
			quick_if_let!(config, self.import_config, wasm_method);
			config.execution_strategies = self
				.import_config
				.execution_strategies(&config.execution_strategies, is_dev);
			quick_if_let!(config, self.import_config, tracing_receiver);
		}

		Ok(runner)
	}
}

/// Available RPC methods.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
enum RpcMethods {
	/// Expose every RPC method only when RPC is listening on `localhost`,
	/// otherwise serve only safe RPC methods.
	Auto,
	/// Allow only a safe subset of RPC methods.
	Safe,
	/// Expose every RPC method (even potentially unsafe ones).
	Unsafe,
}
impl Into<sc_service::config::RpcMethods> for RpcMethods {
	fn into(self) -> sc_service::config::RpcMethods {
		match self {
			RpcMethods::Auto => sc_service::config::RpcMethods::Auto,
			RpcMethods::Safe => sc_service::config::RpcMethods::Safe,
			RpcMethods::Unsafe => sc_service::config::RpcMethods::Unsafe,
		}
	}
}

/// CORS setting
///
/// The type is introduced to overcome `Option<Option<T>>`
/// handling of `structopt`.
enum Cors {
	/// All hosts allowed.
	All,
	/// Only hosts on the list are allowed.
	List(Vec<String>),
}
impl From<Cors> for Option<Vec<String>> {
	fn from(cors: Cors) -> Self {
		match cors {
			Cors::All => None,
			Cors::List(list) => Some(list),
		}
	}
}

/// Configuration of the database of the client.
#[derive(Deserialize)]
struct OffchainWorkerConfig {
	/// Should execute offchain workers on every block.
	///
	/// By default it's only enabled for nodes that are authoring new blocks.
	#[serde(rename = "offchain-worker")]
	enabled: Option<OffchainWorkerEnabled>,
	/// allow writes from the runtime to the offchain worker database.
	/// Enable Offchain Indexing API, which allows block import to write to Offchain DB.
	///
	/// Enables a runtime to write directly to a offchain workers
	/// DB during block import.
	#[serde(rename = "enable-offchain-indexing")]
	indexing_enabled: Option<bool>,
}
impl OffchainWorkerConfig {
	/// Load spec to `Configuration` from `OffchainWorkerParams` and spec factory.
	fn offchain_worker(
		&self,
		origin_config: &sc_service::config::OffchainWorkerConfig,
		role: &sc_service::Role,
	) -> sc_service::config::OffchainWorkerConfig {
		let enabled = if let Some(enabled) = &self.enabled {
			match (enabled, role) {
				(OffchainWorkerEnabled::WhenValidating, sc_service::Role::Authority { .. }) => true,
				(OffchainWorkerEnabled::Always, _) => true,
				(OffchainWorkerEnabled::Never, _) => false,
				(OffchainWorkerEnabled::WhenValidating, _) => false,
			}
		} else {
			origin_config.enabled
		};
		let indexing_enabled = if let Some(indexing_enabled) = self.indexing_enabled {
			indexing_enabled
		} else {
			origin_config.indexing_enabled
		};

		sc_service::config::OffchainWorkerConfig {
			enabled,
			indexing_enabled: enabled && indexing_enabled,
		}
	}
}
/// Whether off-chain workers are enabled.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum OffchainWorkerEnabled {
	Always,
	Never,
	WhenValidating,
}

/// Shared parameters used by all `CoreParams`.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SharedConfig {
	/// Specify the chain specification (one of dev, local, or staging).
	chain: Option<String>,

	/// Specify the development chain.
	dev: Option<bool>,

	/// Specify custom base path.
	base_path: Option<PathBuf>,

	/// Sets a custom logging filter. Syntax is <target>=<level>, e.g. -lsync=debug.
	///
	/// Log levels (least to most verbose) are error, warn, info, debug, and trace.
	/// By default, all targets log `info`. The global log level can be set with -l<level>.
	log: Option<Vec<String>>,
}

/// Parameters for block import.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ImportConfig {
	#[allow(missing_docs)]
	#[serde(flatten)]
	pruning_config: PruningConfig,

	#[allow(missing_docs)]
	#[serde(flatten)]
	database_config: DatabaseConfig,

	/// Force start with unsafe pruning settings.
	///
	/// When running as a validator it is highly recommended to disable state
	/// pruning (i.e. 'archive') which is the default. The node will refuse to
	/// start as a validator if pruning is enabled unless this option is set.
	unsafe_pruning: Option<bool>,

	/// Method for executing Wasm runtime code.
	wasm_method: Option<WasmExecutionMethod>,

	#[allow(missing_docs)]
	#[serde(flatten)]
	execution_strategies: ExecutionStrategiesConfig,

	/// Specify the state cache size.
	state_cache_size: Option<usize>,

	/// Comma separated list of targets for tracing.
	tracing_targets: Option<String>,

	/// Receiver to process tracing messages.
	tracing_receiver: Option<TracingReceiver>,
}
impl ImportConfig {
	fn execution_strategies(
		&self,
		origin_execution_strategies: &sc_client_api::execution_extensions::ExecutionStrategies,
		is_dev: bool,
	) -> sc_client_api::execution_extensions::ExecutionStrategies {
		/// Default value for the `--execution-syncing` parameter.
		const DEFAULT_EXECUTION_SYNCING: ExecutionStrategy = ExecutionStrategy::NativeElseWasm;
		/// Default value for the `--execution-import-block` parameter.
		const DEFAULT_EXECUTION_IMPORT_BLOCK: ExecutionStrategy = ExecutionStrategy::NativeElseWasm;
		/// Default value for the `--execution-block-construction` parameter.
		const DEFAULT_EXECUTION_BLOCK_CONSTRUCTION: ExecutionStrategy = ExecutionStrategy::Wasm;
		/// Default value for the `--execution-offchain-worker` parameter.
		const DEFAULT_EXECUTION_OFFCHAIN_WORKER: ExecutionStrategy = ExecutionStrategy::Native;
		/// Default value for the `--execution-other` parameter.
		const DEFAULT_EXECUTION_OTHER: ExecutionStrategy = ExecutionStrategy::Native;

		let exec = &self.execution_strategies;
		let exec_all_or = |strat: ExecutionStrategy, default: ExecutionStrategy| {
			exec.execution
				.clone()
				.unwrap_or(if strat == default && is_dev {
					ExecutionStrategy::Native
				} else {
					strat
				})
				.into()
		};

		sc_client_api::execution_extensions::ExecutionStrategies {
			syncing: if let Some(syncing) = exec.execution_syncing.clone() {
				exec_all_or(syncing, DEFAULT_EXECUTION_SYNCING)
			} else {
				origin_execution_strategies.syncing
			},
			importing: if let Some(importing) = exec.execution_import_block.clone() {
				exec_all_or(importing, DEFAULT_EXECUTION_IMPORT_BLOCK)
			} else {
				origin_execution_strategies.importing
			},
			block_construction: if let Some(block_construction) =
				exec.execution_block_construction.clone()
			{
				exec_all_or(block_construction, DEFAULT_EXECUTION_BLOCK_CONSTRUCTION)
			} else {
				origin_execution_strategies.block_construction
			},
			offchain_worker: if let Some(offchain_worker) = exec.execution_offchain_worker.clone() {
				exec_all_or(offchain_worker, DEFAULT_EXECUTION_OFFCHAIN_WORKER)
			} else {
				origin_execution_strategies.offchain_worker
			},
			other: if let Some(other) = exec.execution_other.clone() {
				exec_all_or(other, DEFAULT_EXECUTION_OTHER)
			} else {
				origin_execution_strategies.other
			},
		}
	}
}
/// Parameters to define the pruning mode
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct PruningConfig {
	/// Specify the state pruning mode, a number of blocks to keep or 'archive'.
	///
	/// Default is to keep all block states if the node is running as a
	/// validator (i.e. 'archive'), otherwise state is only kept for the last
	/// 256 blocks.
	pruning: Option<String>,
}
/// Parameters for block import.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct DatabaseConfig {
	#[serde(rename = "db")]
	database: Option<Database>,

	/// Limit the memory the database cache can use.
	#[serde(rename = "db-cache")]
	database_cache_size: Option<usize>,
}
/// Database backend
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
enum Database {
	// Facebooks RocksDB
	RocksDb,
	// Subdb. https://github.com/paritytech/subdb/
	SubDb,
	// ParityDb. https://github.com/paritytech/parity-db/
	ParityDb,
}
/// How to execute Wasm runtime code
#[derive(Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
enum WasmExecutionMethod {
	// Uses an interpreter.
	Interpreted,
	// Uses a compiled runtime.
	Compiled,
}
impl Into<sc_service::config::WasmExecutionMethod> for WasmExecutionMethod {
	fn into(self) -> sc_service::config::WasmExecutionMethod {
		match self {
			WasmExecutionMethod::Interpreted => {
				sc_service::config::WasmExecutionMethod::Interpreted
			}
			#[cfg(feature = "wasmtime")]
			WasmExecutionMethod::Compiled => sc_service::config::WasmExecutionMethod::Compiled,
			#[cfg(not(feature = "wasmtime"))]
			WasmExecutionMethod::Compiled => panic!(
				"Substrate must be compiled with \"wasmtime\" feature for compiled Wasm execution"
			),
		}
	}
}
/// Execution strategies parameters.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ExecutionStrategiesConfig {
	/// The means of execution used when calling into the runtime while syncing blocks.
	// #[structopt(
	// long = "execution-syncing",
	// value_name = "STRATEGY",
	// possible_values = &ExecutionStrategy::variants(),
	// case_insensitive = true,
	// default_value = DEFAULT_EXECUTION_SYNCING.as_str(),
	// )]
	execution_syncing: Option<ExecutionStrategy>,

	/// The means of execution used when calling into the runtime while importing blocks.
	// #[structopt(
	// long = "execution-import-block",
	// value_name = "STRATEGY",
	// possible_values = &ExecutionStrategy::variants(),
	// case_insensitive = true,
	// default_value = DEFAULT_EXECUTION_IMPORT_BLOCK.as_str(),
	// )]
	execution_import_block: Option<ExecutionStrategy>,

	/// The means of execution used when calling into the runtime while constructing blocks.
	// #[structopt(
	// long = "execution-block-construction",
	// value_name = "STRATEGY",
	// possible_values = &ExecutionStrategy::variants(),
	// case_insensitive = true,
	// default_value = DEFAULT_EXECUTION_BLOCK_CONSTRUCTION.as_str(),
	// )]
	execution_block_construction: Option<ExecutionStrategy>,

	/// The means of execution used when calling into the runtime while using an off-chain worker.
	// #[structopt(
	// long = "execution-offchain-worker",
	// value_name = "STRATEGY",
	// possible_values = &ExecutionStrategy::variants(),
	// case_insensitive = true,
	// default_value = DEFAULT_EXECUTION_OFFCHAIN_WORKER.as_str(),
	// )]
	execution_offchain_worker: Option<ExecutionStrategy>,

	/// The means of execution used when calling into the runtime while not syncing, importing or constructing blocks.
	// #[structopt(
	// long = "execution-other",
	// value_name = "STRATEGY",
	// possible_values = &ExecutionStrategy::variants(),
	// case_insensitive = true,
	// default_value = DEFAULT_EXECUTION_OTHER.as_str(),
	// )]
	execution_other: Option<ExecutionStrategy>,

	/// The execution strategy that should be used by all execution contexts.
	// #[structopt(
	// long = "execution",
	// value_name = "STRATEGY",
	// possible_values = &ExecutionStrategy::variants(),
	// case_insensitive = true,
	// conflicts_with_all = &[
	// "execution-other",
	// "execution-offchain-worker",
	// "execution-block-construction",
	// "execution-import-block",
	// "execution-syncing",
	// ]
	// )]
	execution: Option<ExecutionStrategy>,
}
/// How to execute blocks
#[derive(Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
enum ExecutionStrategy {
	// Execute with native build (if available, WebAssembly otherwise).
	Native,
	// Only execute with the WebAssembly build.
	Wasm,
	// Execute with both native (where available) and WebAssembly builds.
	Both,
	// Execute with the native build if possible; if it fails, then execute with WebAssembly.
	NativeElseWasm,
}
impl Into<sc_client_api::ExecutionStrategy> for ExecutionStrategy {
	fn into(self) -> sc_client_api::ExecutionStrategy {
		match self {
			ExecutionStrategy::Native => sc_client_api::ExecutionStrategy::NativeWhenPossible,
			ExecutionStrategy::Wasm => sc_client_api::ExecutionStrategy::AlwaysWasm,
			ExecutionStrategy::Both => sc_client_api::ExecutionStrategy::Both,
			ExecutionStrategy::NativeElseWasm => sc_client_api::ExecutionStrategy::NativeElseWasm,
		}
	}
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
enum TracingReceiver {
	Log,
	Telemetry,
}
impl Into<sc_tracing::TracingReceiver> for TracingReceiver {
	fn into(self) -> sc_tracing::TracingReceiver {
		match self {
			TracingReceiver::Log => sc_tracing::TracingReceiver::Log,
			TracingReceiver::Telemetry => sc_tracing::TracingReceiver::Telemetry,
		}
	}
}

/// Parameters used to create the network configuration.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct NetworkConfig {
	/// Specify a list of bootnodes.
	bootnodes: Vec<sc_service::config::MultiaddrWithPeerId>,

	/// Specify a list of reserved node addresses.
	reserved_nodes: Vec<sc_service::config::MultiaddrWithPeerId>,

	/// Whether to only allow connections to/from reserved nodes.
	///
	/// If you are a validator your node might still connect to other validator
	/// nodes regardless of whether they are defined as reserved nodes.
	reserved_only: bool,

	/// The public address that other nodes will use to connect to it.
	/// This can be used if there's a proxy in front of this node.
	public_addr: Vec<sc_service::config::Multiaddr>,

	/// Listen on this multiaddress.
	listen_addr: Vec<sc_service::config::Multiaddr>,

	/// Specify p2p protocol TCP port.
	port: Option<u16>,

	/// Forbid connecting to private IPv4 addresses (as specified in
	/// [RFC1918](https://tools.ietf.org/html/rfc1918)), unless the address was passed with
	/// `--reserved-nodes` or `--bootnodes`.
	no_private_ipv4: bool,

	/// Specify the number of outgoing connections we're trying to maintain.
	out_peers: u32,

	/// Specify the maximum number of incoming connections we're accepting.
	in_peers: u32,

	/// Disable mDNS discovery.
	///
	/// By default, the network will use mDNS to discover other nodes on the
	/// local network. This disables it. Automatically implied when using --dev.
	no_mdns: bool,

	/// Maximum number of peers from which to ask for the same blocks in parallel.
	///
	/// This allows downloading announced blocks from multiple peers. Decrease to save
	/// traffic and risk increased latency.
	max_parallel_downloads: u32,

	#[allow(missing_docs)]
	#[serde(flatten)]
	node_key_config: NodeKeyConfig,

	/// Disable the yamux flow control. This option will be removed in the future once there is
	/// enough confidence that this feature is properly working.
	no_yamux_flow_control: bool,

	/// Enable peer discovery on local networks.
	///
	/// By default this option is true for `--dev` and false otherwise.
	discover_local: bool,

	/// Use the legacy "pre-mainnet-launch" networking protocol. Enable if things seem broken.
	/// This option will be removed in the future.
	legacy_network_protocol: bool,
}
/// Parameters used to create the `NodeKeyConfig`, which determines the keypair
/// used for libp2p networking.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct NodeKeyConfig {
	/// The secret key to use for libp2p networking.
	///
	/// The value is a string that is parsed according to the choice of
	/// `--node-key-type` as follows:
	///
	///   `ed25519`:
	///   The value is parsed as a hex-encoded Ed25519 32 byte secret key,
	///   i.e. 64 hex characters.
	///
	/// The value of this option takes precedence over `--node-key-file`.
	///
	/// WARNING: Secrets provided as command-line arguments are easily exposed.
	/// Use of this option should be limited to development and testing. To use
	/// an externally managed secret key, use `--node-key-file` instead.
	node_key: Option<String>,

	/// The type of secret key to use for libp2p networking.
	///
	/// The secret key of the node is obtained as follows:
	///
	///   * If the `--node-key` option is given, the value is parsed as a secret key
	///     according to the type. See the documentation for `--node-key`.
	///
	///   * If the `--node-key-file` option is given, the secret key is read from the
	///     specified file. See the documentation for `--node-key-file`.
	///
	///   * Otherwise, the secret key is read from a file with a predetermined,
	///     type-specific name from the chain-specific network config directory
	///     inside the base directory specified by `--base-dir`. If this file does
	///     not exist, it is created with a newly generated secret key of the
	///     chosen type.
	///
	/// The node's secret key determines the corresponding public key and hence the
	/// node's peer ID in the context of libp2p.
	node_key_type: NodeKeyType,

	/// The file from which to read the node's secret key to use for libp2p networking.
	///
	/// The contents of the file are parsed according to the choice of `--node-key-type`
	/// as follows:
	///
	///   `ed25519`:
	///   The file must contain an unencoded 32 byte Ed25519 secret key.
	///
	/// If the file does not exist, it is created with a newly generated secret key of
	/// the chosen type.
	pub node_key_file: Option<PathBuf>,
}
// impl NodeKeyConfig {
// 	/// Create a `NodeKeyConfig` from the given `NodeKeyParams` in the context
// 	/// of an optional network config storage directory.
// 	fn node_key(&self, net_config_dir: &PathBuf) -> sc_service::config::NodeKeyConfig {
// 		/// The file name of the node's Ed25519 secret key inside the chain-specific
// 		/// network config directory, if neither `--node-key` nor `--node-key-file`
// 		/// is specified in combination with `--node-key-type=ed25519`.
// 		const NODE_KEY_ED25519_FILE: &str = "secret_ed25519";
//
// 		match self.node_key_type {
// 			NodeKeyType::Ed25519 => {
// 				let secret = if let Some(node_key) = self.node_key.as_ref() {
// 					parse_ed25519_secret(node_key)?
// 				} else {
// 					let path = self
// 						.node_key_file
// 						.clone()
// 						.unwrap_or_else(|| net_config_dir.join(NODE_KEY_ED25519_FILE));
//
// 					sc_network::config::Secret::File(path)
// 				};
//
// 				NodeKeyConfig::Ed25519(secret)
// 			}
// 		}
// 	}
// }
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
enum NodeKeyType {
	Ed25519,
}

fn parse_cors(s: &str) -> Cors {
	let mut is_all = false;
	let mut origins = Vec::new();
	for part in s.split(',') {
		match part {
			"All" | "*" => {
				is_all = true;
				break;
			}
			other => origins.push(other.to_owned()),
		}
	}

	if is_all {
		Cors::All
	} else {
		Cors::List(origins)
	}
}

fn parse_telemetry_endpoints(s: &str) -> (String, u8) {
	let pos = s.find(' ').unwrap();
	let url = s[..pos].to_string();
	let verbosity = s[pos + 1..].parse().unwrap();

	(url, verbosity)
}
