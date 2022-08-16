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

// --- std ---
use std::{env, path::PathBuf};
// --- paritytech ---
use sc_cli::{Error as CliError, Result as CliResult, RuntimeVersion, SubstrateCli};
use sc_service::{ChainSpec, DatabaseSource};
#[cfg(feature = "try-runtime")]
use sc_service::{Error as ServiceError, TaskManager};
use sp_core::crypto::{self, Ss58AddressFormatRegistry};
// --- darwinia-network ---
use crate::cli::*;
use darwinia_node_service::*;
#[cfg(any(feature = "try-runtime", feature = "runtime-benchmarks"))]
use darwinia_primitives::OpaqueBlock as Block;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Darwinia".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn executable_name() -> String {
		"darwinia".into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
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

	fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		if spec.is_crab() {
			&crab_runtime::VERSION
		} else {
			&darwinia_runtime::VERSION
		}
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn ChainSpec>, String> {
		let id = if id.is_empty() {
			let n = get_exec_name().unwrap_or_default();
			["darwinia", "crab"]
				.iter()
				.cloned()
				.find(|&chain| n.starts_with(chain))
				.unwrap_or("darwinia")
		} else {
			id
		};

		Ok(match id.to_lowercase().as_ref() {
			"crab" => Box::new(crab_chain_spec::config()?),
			"crab-dev" => Box::new(crab_chain_spec::development_config()),
			"crab-genesis" => Box::new(crab_chain_spec::genesis_config()),
			"darwinia" => Box::new(darwinia_chain_spec::config()?),
			"darwinia-dev" | "dev" => Box::new(darwinia_chain_spec::development_config()),
			"darwinia-genesis" => Box::new(darwinia_chain_spec::genesis_config()),
			_ => {
				let path = PathBuf::from(id);
				let chain_spec = Box::new(DarwiniaChainSpec::from_json_file(path.clone())?)
					as Box<dyn ChainSpec>;

				if self.run.force_crab || chain_spec.is_crab() {
					Box::new(CrabChainSpec::from_json_file(path)?)
				} else {
					chain_spec
				}
			},
		})
	}
}

/// Parses Darwinia specific CLI arguments and run the service.
pub fn run() -> CliResult<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		None => {
			validate_trace_environment(&cli)?;

			let runner = cli.create_runner(&cli.run.base).map_err(CliError::from)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			log::info!("  _____                      _       _       ");
			log::info!(" |  __ \\                    (_)     (_)      ");
			log::info!(" | |  | | __ _ _ ____      ___ _ __  _  __ _ ");
			log::info!(" | |  | |/ _` | '__\\ \\ /\\ / / | '_ \\| |/ _` |");
			log::info!(" | |__| | (_| | |   \\ V  V /| | | | | | (_| |");
			log::info!(" |_____/ \\__,_|_|    \\_/\\_/ |_|_| |_|_|\\__,_|");

			let authority_discovery_disabled = cli.run.authority_discovery_disabled;
			let eth_rpc_config = cli.run.dvm_args.build_eth_rpc_config();

			if chain_spec.is_crab() {
				runner.run_node_until_exit(|config| async move {
					crab_service::new_full(config, authority_discovery_disabled, eth_rpc_config)
						.map(|(task_manager, _, _)| task_manager)
						.map_err(CliError::from)
				})
			} else {
				runner.run_node_until_exit(|config| async move {
					darwinia_service::new_full(config, authority_discovery_disabled, eth_rpc_config)
						.map(|(task_manager, _, _)| task_manager)
						.map_err(CliError::from)
				})
			}
		},
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.async_run(|mut config| {
					let (client, _, import_queue, task_manager) =
						darwinia_node_service::new_chain_ops::<CrabRuntimeApi, CrabExecutor>(
							&mut config,
						)?;

					Ok((cmd.run(client, import_queue), task_manager))
				})
			} else {
				runner.async_run(|mut config| {
					let (client, _, import_queue, task_manager) =
						darwinia_node_service::new_chain_ops::<DarwiniaRuntimeApi, DarwiniaExecutor>(
							&mut config,
						)?;

					Ok((cmd.run(client, import_queue), task_manager))
				})
			}
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.async_run(|mut config| {
					let (client, _, _, task_manager) = darwinia_node_service::new_chain_ops::<
						CrabRuntimeApi,
						CrabExecutor,
					>(&mut config)?;

					Ok((cmd.run(client, config.database), task_manager))
				})
			} else {
				runner.async_run(|mut config| {
					let (client, _, _, task_manager) = darwinia_node_service::new_chain_ops::<
						DarwiniaRuntimeApi,
						DarwiniaExecutor,
					>(&mut config)?;

					Ok((cmd.run(client, config.database), task_manager))
				})
			}
		},
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.async_run(|mut config| {
					let (client, _, _, task_manager) = darwinia_node_service::new_chain_ops::<
						CrabRuntimeApi,
						CrabExecutor,
					>(&mut config)?;

					Ok((cmd.run(client, config.chain_spec), task_manager))
				})
			} else {
				runner.async_run(|mut config| {
					let (client, _, _, task_manager) = darwinia_node_service::new_chain_ops::<
						DarwiniaRuntimeApi,
						DarwiniaExecutor,
					>(&mut config)?;

					Ok((cmd.run(client, config.chain_spec), task_manager))
				})
			}
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.async_run(|mut config| {
					let (client, _, import_queue, task_manager) =
						darwinia_node_service::new_chain_ops::<CrabRuntimeApi, CrabExecutor>(
							&mut config,
						)?;

					Ok((cmd.run(client, import_queue), task_manager))
				})
			} else {
				runner.async_run(|mut config| {
					let (client, _, import_queue, task_manager) =
						darwinia_node_service::new_chain_ops::<DarwiniaRuntimeApi, DarwiniaExecutor>(
							&mut config,
						)?;

					Ok((cmd.run(client, import_queue), task_manager))
				})
			}
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.sync_run(|config| {
					// Remove dvm offchain db
					let dvm_database_config = DatabaseSource::RocksDb {
						path: darwinia_node_service::dvm::db_path(&config),
						cache_size: 0,
					};

					cmd.run(dvm_database_config)?;
					cmd.run(config.database)
				})
			} else {
				runner.sync_run(|config| cmd.run(config.database))
			}
		},
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.async_run(|mut config| {
					let (client, backend, _, task_manager) = darwinia_node_service::new_chain_ops::<
						CrabRuntimeApi,
						CrabExecutor,
					>(&mut config)?;

					Ok((cmd.run(client, backend), task_manager))
				})
			} else {
				runner.async_run(|mut config| {
					let (client, backend, _, task_manager) = darwinia_node_service::new_chain_ops::<
						DarwiniaRuntimeApi,
						DarwiniaExecutor,
					>(&mut config)?;

					Ok((cmd.run(client, backend), task_manager))
				})
			}
		},
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::Sign(cmd)) => cmd.run(),
		Some(Subcommand::Verify(cmd)) => cmd.run(),
		Some(Subcommand::Vanity(cmd)) => cmd.run(),
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.async_run(|config| {
					let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
					// we don't need any of the components of new_partial, just a runtime, or a task
					// manager to do `async_run`.
					let task_manager = TaskManager::new(config.tokio_handle.clone(), registry)
						.map_err(|e| CliError::from(ServiceError::Prometheus(e)))?;

					Ok((cmd.run::<Block, CrabExecutor>(config), task_manager))
				})
			} else {
				runner.async_run(|config| {
					let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
					// we don't need any of the components of new_partial, just a runtime, or a task
					// manager to do `async_run`.
					let task_manager = TaskManager::new(config.tokio_handle.clone(), registry)
						.map_err(|e| CliError::from(ServiceError::Prometheus(e)))?;

					Ok((cmd.run::<Block, DarwiniaExecutor>(config), task_manager))
				})
			}
		},
		#[cfg(feature = "runtime-benchmarks")]
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.sync_run(|config| cmd.run::<Block, CrabExecutor>(config))
			} else if chain_spec.is_darwinia() {
				runner.sync_run(|config| cmd.run::<Block, DarwiniaExecutor>(config))
			} else {
				Err("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
					.into())
			}
		},
	}
}

fn get_exec_name() -> Option<String> {
	env::current_exe().ok()?.file_name().map(|name| name.to_string_lossy().into_owned())
}

fn set_default_ss58_version(spec: &Box<dyn ChainSpec>) {
	let ss58_version = if spec.is_crab() {
		Ss58AddressFormatRegistry::SubstrateAccount
	} else {
		Ss58AddressFormatRegistry::DarwiniaAccount
	};

	crypto::set_default_ss58_version(ss58_version.into());
}

fn validate_trace_environment(cli: &Cli) -> CliResult<()> {
	if cli
		.run
		.dvm_args
		.ethapi_debug_targets
		.iter()
		.any(|target| matches!(target.as_str(), "debug" | "trace"))
		&& cli.run.base.import_params.wasm_runtime_overrides.is_none()
	{
		Err("`debug` or `trace` namespaces requires `--wasm-runtime-overrides /path/to/overrides`."
			.into())
	} else {
		Ok(())
	}
}
