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

// std
use std::{env, net::SocketAddr, path::PathBuf};
// cumulus
use cumulus_primitives_core::ParaId;
// darwinia
use crate::{
	chain_spec::*,
	cli::{Cli, FrontierBackendType, RelayChainCli, Subcommand, NODE_VERSION},
	service::{self, *},
};
// substrate
use sc_cli::{
	CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams, NetworkParams,
	Result, SharedParams, SubstrateCli,
};
use sc_service::{
	config::{BasePath, PrometheusConfig},
	ChainSpec as ChainSpecT, DatabaseSource,
};
use sp_core::crypto::{self, Ss58AddressFormatRegistry};
use sp_runtime::traits::AccountIdConversion;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Darwinia".into()
	}

	fn impl_version() -> String {
		let commit_hash = env!("SUBSTRATE_CLI_COMMIT_HASH");

		format!("{NODE_VERSION}-{commit_hash}")
	}

	fn description() -> String {
		format!(
			"Darwinia\n\nThe command-line arguments provided first will be \
			passed to the parachain node, while the arguments provided after -- will be passed \
			to the relay chain node.\n\n\
			{} <parachain-args> -- <relay-chain-args>",
			Self::executable_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/darwinia-network/darwinia/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2018
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn ChainSpecT>, String> {
		load_spec(id)
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Darwinia".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		format!(
			"Darwinia\n\nThe command-line arguments provided first will be \
			passed to the parachain node, while the arguments provided after -- will be passed \
			to the relay chain node.\n\n\
			{} <parachain-args> -- <relay-chain-args>",
			Self::executable_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/darwinia-network/darwinia/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2018
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn ChainSpecT>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}
}
impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_listen_port() -> u16 {
		9945
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}
impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self.shared_params().base_path()?.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_addr(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_addr(default_listen_port)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpecT>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self, is_dev: bool) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool(is_dev)
	}

	fn trie_cache_maximum_size(&self) -> Result<Option<usize>> {
		self.base.base.trie_cache_maximum_size()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_max_connections(&self) -> Result<u32> {
		self.base.base.rpc_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}

	fn telemetry_endpoints(
		&self,
		chain_spec: &Box<dyn ChainSpecT>,
	) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}

	fn node_name(&self) -> Result<String> {
		self.base.base.node_name()
	}
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	#[cfg(feature = "runtime-benchmarks")]
	/// Creates partial components for the runtimes that are supported by the benchmarks.
	macro_rules! construct_benchmark_partials {
		($config:expr, $cli:ident, |$partials:ident| $code:expr) => {{
			#[cfg(feature = "crab-native")]
			if $config.chain_spec.is_crab() {
				let $partials = service::new_partial::<CrabRuntimeApi>(
					&$config,
					&$cli.eth_args.build_eth_rpc_config(),
				)?;

				return $code;
			}

			#[cfg(feature = "darwinia-native")]
			if $config.chain_spec.is_darwinia() {
				let $partials = service::new_partial::<DarwiniaRuntimeApi>(
					&$config,
					&$cli.eth_args.build_eth_rpc_config(),
				)?;

				return $code;
			}

			#[cfg(feature = "koi-native")]
			if $config.chain_spec.is_koi() {
				let $partials = service::new_partial::<KoiRuntimeApi>(
					&$config,
					&$cli.eth_args.build_eth_rpc_config(),
				)?;

				return $code;
			}

			panic!("No feature(crab-native, darwinia-native, koi-native) is enabled!");
		}};
	}

	macro_rules! construct_async_run {
		(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
			let runner = $cli.create_runner($cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			#[cfg(feature = "crab-native")]
			if chain_spec.is_crab() {
				return runner.async_run(|$config| {
					let $components = service::new_partial::<CrabRuntimeApi>(
						&$config,
						&$cli.eth_args.build_eth_rpc_config()
					)?;
					let task_manager = $components.task_manager;

					{ $( $code )* }.map(|v| (v, task_manager))
				});
			}

			#[cfg(feature = "darwinia-native")]
			if chain_spec.is_darwinia() {
				return runner.async_run(|$config| {
					let $components = service::new_partial::<DarwiniaRuntimeApi>(
						&$config,
						&$cli.eth_args.build_eth_rpc_config()
					)?;
					let task_manager = $components.task_manager;

					{ $( $code )* }.map(|v| (v, task_manager))
				});
			}

			#[cfg(feature = "koi-native")]
			if chain_spec.is_koi() {
				return runner.async_run(|$config| {
					let $components = service::new_partial::<KoiRuntimeApi>(
						&$config,
						&$cli.eth_args.build_eth_rpc_config()
					)?;
					let task_manager = $components.task_manager;

					{ $( $code )* }.map(|v| (v, task_manager))
				});
			}

			panic!("No feature(crab-native, darwinia-native, koi-native) is enabled!");
		}}
	}

	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.database))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.chain_spec))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::Revert(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.backend, None))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);
			runner.sync_run(|config| {
				// Remove Frontier off-chain db
				let db_config_dir = frontier::db_config_dir(&config);
				match cli.eth_args.frontier_backend_type {
					FrontierBackendType::KeyValue => {
						let frontier_database_config = match config.database {
							DatabaseSource::RocksDb { .. } => DatabaseSource::RocksDb {
								path: fc_db::kv::frontier_database_dir(&db_config_dir, "db"),
								cache_size: 0,
							},
							DatabaseSource::ParityDb { .. } => DatabaseSource::ParityDb {
								path: fc_db::kv::frontier_database_dir(&db_config_dir, "paritydb"),
							},
							_ => {
								return Err(format!(
									"Cannot purge `{:?}` database",
									config.database
								)
								.into())
							}
						};
						cmd.base.run(frontier_database_config)?;
					}
					FrontierBackendType::Sql => {
						let db_path = db_config_dir.join("sql");
						match std::fs::remove_dir_all(&db_path) {
							Ok(_) => {
								println!("{:?} removed.", &db_path);
							}
							Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => {
								eprintln!("{:?} did not exist.", &db_path);
							}
							Err(err) => {
								return Err(format!(
									"Cannot purge `{:?}` database: {:?}",
									db_path, err,
								)
								.into())
							}
						};
					}
				};

				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
				);
				let polkadot_config = SubstrateCli::create_configuration(
					&polkadot_cli,
					&polkadot_cli,
					config.tokio_handle.clone(),
				)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		},
		Some(Subcommand::ExportGenesisHead(cmd)) =>
			construct_async_run!(|components, cli, cmd, config| {
				Ok(async move { cmd.run(&*config.chain_spec, &*components.client) })
			}),
		Some(Subcommand::ExportGenesisWasm(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);
			runner.sync_run(|_config| {
				let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
				cmd.run(&*spec)
			})
		},
		#[cfg(feature = "runtime-benchmarks")]
		Some(Subcommand::Benchmark(cmd)) => {
			// darwinia
			use dc_primitives::Block;
			// substrate
			use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};

			let runner = cli.create_runner(&**cmd)?;

			set_default_ss58_version(&runner.config().chain_spec);

			match &**cmd {
				BenchmarkCmd::Pallet(cmd) =>
					runner.sync_run(|config| cmd.run::<Block, ()>(config)),
				BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, cli, |partials| {
						let db = partials.backend.expose_db();
						let storage = partials.backend.expose_storage();

						cmd.run(config, partials.client.clone(), db, storage)
					})
				}),
				BenchmarkCmd::Overhead(_) => Err("Unsupported benchmarking command".into()),
				BenchmarkCmd::Extrinsic(_) => Err("Unsupported benchmarking command".into()),
				BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, cli, |partials| cmd.run(partials.client))
				}),
				BenchmarkCmd::Machine(cmd) =>
					runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())),
			}
		},
		#[cfg(not(feature = "runtime-benchmarks"))]
		Some(Subcommand::Benchmark) => Err(
			"Benchmarking was not enabled when building the node. You can enable it with `--features runtime-benchmarks`.".into()
		),
		Some(Subcommand::TryRuntime) => Err("The `try-runtime` subcommand has been migrated to a standalone CLI (https://github.com/paritytech/try-runtime-cli). It is no longer being maintained here and will be removed entirely some time after January 2024. Please remove this subcommand from your runtime and use the standalone CLI.".into()),
		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;
			let collator_options = cli.run.collator_options();

			runner.run_node_until_exit(|config| async move {
				let chain_spec = &config.chain_spec;
				let hwbench = (!cli.no_hardware_benchmarks).then_some(
					config.database.path().map(|database_path| {
						let _ = std::fs::create_dir_all(database_path);
						sc_sysinfo::gather_hwbench(Some(database_path))
					})).flatten();

				set_default_ss58_version(chain_spec);

				let para_id = Extensions::try_get(&*config.chain_spec)
					.map(|e| e.para_id)
					.ok_or("Could not find parachain ID in chain-spec.")?;
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
				);
				let id = ParaId::from(para_id);
				let parachain_account =
					AccountIdConversion::<polkadot_primitives::AccountId>::into_account_truncating(&id);
				let tokio_handle = config.tokio_handle.clone();
				let eth_rpc_config = cli.eth_args.build_eth_rpc_config();

				log::info!("Parachain id: {:?}", id);
				log::info!("Parachain Account: {}", parachain_account);
				log::info!(
					"Is collating: {}",
					if config.role.is_authority() { "yes" } else { "no" }
				);

				if chain_spec.is_dev() {
					#[cfg(feature = "crab-native")]
					if chain_spec.is_crab() {
						return service::start_dev_node::<CrabRuntimeApi>(
							config,
							&eth_rpc_config,
						)
						.map_err(Into::into);
					}

					#[cfg(feature = "darwinia-native")]
					if chain_spec.is_darwinia() {
						return service::start_dev_node::<DarwiniaRuntimeApi>(
							config,
							&eth_rpc_config,
						)
						.map_err(Into::into)
					}

					#[cfg(feature = "koi-native")]
					if chain_spec.is_koi() {
						return service::start_dev_node::<KoiRuntimeApi>(
							config,
							&eth_rpc_config,
						)
						.map_err(Into::into)
					}
				}

				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				#[cfg(feature = "crab-native")]
				if chain_spec.is_crab() {
					return service::start_parachain_node::<CrabRuntimeApi>(
						config,
						polkadot_config,
						collator_options,
						id,
						hwbench,
						&eth_rpc_config,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into);
				}

				#[cfg(feature = "darwinia-native")]
				if chain_spec.is_darwinia() {
					return service::start_parachain_node::<DarwiniaRuntimeApi>(
						config,
						polkadot_config,
						collator_options,
						id,
						hwbench,
						&eth_rpc_config,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into);
				}

				#[cfg(feature = "koi-native")]
				if chain_spec.is_koi() {
					return service::start_parachain_node::<KoiRuntimeApi>(
						config,
						polkadot_config,
						collator_options,
						id,
						hwbench,
						&eth_rpc_config,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into);
				}

				panic!("No feature(crab-native, darwinia-native, koi-native) is enabled!");
			})
		},
	}
}

fn load_spec(id: &str) -> std::result::Result<Box<dyn ChainSpecT>, String> {
	let id = if id.is_empty() {
		let n = get_exec_name().unwrap_or_default();
		["darwinia", "crab", "koi"]
			.iter()
			.cloned()
			.find(|&chain| n.starts_with(chain))
			.unwrap_or("darwinia")
	} else {
		id
	};
	let chain_spec = match id.to_lowercase().as_str() {
		#[cfg(feature = "crab-native")]
		"crab" => Box::new(crab_chain_spec::config()),
		#[cfg(feature = "crab-native")]
		"crab-genesis" => Box::new(crab_chain_spec::genesis_config()),
		#[cfg(feature = "crab-native")]
		"crab-dev" => Box::new(crab_chain_spec::development_config()),
		#[cfg(feature = "darwinia-native")]
		"darwinia" => Box::new(darwinia_chain_spec::config()),
		#[cfg(feature = "darwinia-native")]
		"darwinia-genesis" => Box::new(darwinia_chain_spec::genesis_config()),
		#[cfg(feature = "darwinia-native")]
		"darwinia-dev" => Box::new(darwinia_chain_spec::development_config()),
		#[cfg(feature = "koi-native")]
		"koi" => Box::new(koi_chain_spec::config()),
		#[cfg(feature = "koi-native")]
		"koi-genesis" => Box::new(koi_chain_spec::genesis_config()),
		#[cfg(feature = "koi-native")]
		"koi-dev" => Box::new(koi_chain_spec::development_config()),
		_ => {
			let path = PathBuf::from(id);
			let chain_spec = Box::new(ChainSpec::from_json_file(path.clone())?);

			chain_spec
		},
	};

	Ok(chain_spec)
}

fn get_exec_name() -> Option<String> {
	env::current_exe()
		.ok()
		.and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
		.and_then(|s| s.into_string().ok())
}

fn set_default_ss58_version(chain_spec: &dyn IdentifyVariant) {
	let ss58_version = if chain_spec.is_crab() || chain_spec.is_koi() {
		Ss58AddressFormatRegistry::SubstrateAccount
	} else {
		Ss58AddressFormatRegistry::DarwiniaAccount
	}
	.into();

	crypto::set_default_ss58_version(ss58_version);
}
