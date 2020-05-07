// --- crates ---
use serde::Deserialize;

/// Service configuration.
#[derive(Deserialize)]
pub struct Configuration {
	/// Implementation name
	pub impl_name: String,
	/// Implementation version (see sc-cli to see an example of format)
	pub impl_version: String,
	// /// Node role.
	// pub role: Role,
	// /// How to spawn background tasks. Mandatory, otherwise creating a `Service` will error.
	// pub task_executor: Arc<dyn Fn(Pin<Box<dyn Future<Output = ()> + Send>>) + Send + Sync>,
	// /// Extrinsic pool configuration.
	// pub transaction_pool: TransactionPoolOptions,
	// /// Network configuration.
	// pub network: NetworkConfiguration,
	// /// Configuration for the keystore.
	// pub keystore: KeystoreConfig,
	// /// Configuration for the database.
	// pub database: DatabaseConfig,
	// /// Size of internal state cache in Bytes
	// pub state_cache_size: usize,
	// /// Size in percent of cache size dedicated to child tries
	// pub state_cache_child_ratio: Option<usize>,
	// /// Pruning settings.
	// pub pruning: PruningMode,
	// /// Chain configuration.
	// pub chain_spec: Box<dyn ChainSpec>,
	// /// Wasm execution method.
	// pub wasm_method: WasmExecutionMethod,
	// /// Execution strategies.
	// pub execution_strategies: ExecutionStrategies,
	// /// Whether potentially unsafe RPC is considered safe to be exposed.
	// pub unsafe_rpc_expose: bool,
	// /// RPC over HTTP binding address. `None` if disabled.
	// pub rpc_http: Option<SocketAddr>,
	// /// RPC over Websockets binding address. `None` if disabled.
	// pub rpc_ws: Option<SocketAddr>,
	// /// Maximum number of connections for WebSockets RPC server. `None` if default.
	// pub rpc_ws_max_connections: Option<usize>,
	// /// CORS settings for HTTP & WS servers. `None` if all origins are allowed.
	// pub rpc_cors: Option<Vec<String>>,
	// /// Prometheus endpoint configuration. `None` if disabled.
	// pub prometheus_config: Option<PrometheusConfig>,
	// /// Telemetry service URL. `None` if disabled.
	// pub telemetry_endpoints: Option<TelemetryEndpoints>,
	// /// External WASM transport for the telemetry. If `Some`, when connection to a telemetry
	// /// endpoint, this transport will be tried in priority before all others.
	// pub telemetry_external_transport: Option<ExtTransport>,
	// /// The default number of 64KB pages to allocate for Wasm execution
	// pub default_heap_pages: Option<u64>,
	// /// Should offchain workers be executed.
	// pub offchain_worker: OffchainWorkerConfig,
	// /// Enable authoring even when offline.
	// pub force_authoring: bool,
	// /// Disable GRANDPA when running in validator mode
	// pub disable_grandpa: bool,
	// /// Development key seed.
	// ///
	// /// When running in development mode, the seed will be used to generate authority keys by the keystore.
	// ///
	// /// Should only be set when `node` is running development mode.
	// pub dev_key_seed: Option<String>,
	// /// Tracing targets
	// pub tracing_targets: Option<String>,
	// /// Tracing receiver
	// pub tracing_receiver: sc_tracing::TracingReceiver,
	// /// The size of the instances cache.
	// ///
	// /// The default value is 8.
	// pub max_runtime_instances: usize,
	// /// Announce block automatically after they have been imported
	// pub announce_block: bool,
}

impl Configuration {
	pub fn load_config(path: &std::path::PathBuf) -> Self {
		serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap()
	}

	pub fn update_config(self, config: &mut sc_service::config::Configuration) {
		let Self {
			impl_name,
			impl_version,
		} = self;
	}
}
