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

// --- std ---
use std::path::PathBuf;
// --- paritytech ---
use sc_cli::{Role, RuntimeVersion, SubstrateCli};
use sc_service::ChainSpec;
#[cfg(feature = "try-runtime")]
use sc_service::TaskManager;
use sp_core::crypto::Ss58AddressFormat;
// --- darwinia ---
use crate::cli::{Cli, Subcommand};
use darwinia_service::{
	chain_spec,
	service::{
		crab::{self, crab_runtime, CrabExecutor},
		darwinia::{self, darwinia_runtime, DarwiniaExecutor},
		IdentifyVariant,
	},
	CrabChainSpec, DarwiniaChainSpec,
};

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
			"crab" => Box::new(chain_spec::crab_config()?),
			"crab-dev" => Box::new(chain_spec::crab_development_config()),
			"crab-genesis" => Box::new(chain_spec::crab_build_spec_config()),
			"darwinia" => Box::new(chain_spec::darwinia_config()?),
			"darwinia-dev" | "dev" => Box::new(chain_spec::darwinia_development_config()),
			"darwinia-genesis" => Box::new(chain_spec::darwinia_build_spec_config()),
			path => {
				let path = PathBuf::from(path);
				let chain_spec = Box::new(DarwiniaChainSpec::from_json_file(path.clone())?)
					as Box<dyn ChainSpec>;

				if self.run.force_crab || chain_spec.is_crab() {
					Box::new(CrabChainSpec::from_json_file(path)?)
				} else {
					chain_spec
				}
			}
		})
	}
}

fn get_exec_name() -> Option<String> {
	std::env::current_exe()
		.ok()
		.and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
		.and_then(|s| s.into_string().ok())
}

fn set_default_ss58_version(spec: &Box<dyn ChainSpec>) {
	let ss58_version = if spec.is_crab() {
		Ss58AddressFormat::SubstrateAccount
	} else {
		Ss58AddressFormat::DarwiniaAccount
	};

	sp_core::crypto::set_default_ss58_version(ss58_version);
}

/// Parses Darwinia specific CLI arguments and run the service.
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();
	let max_past_logs = cli.run.dynamic_fee_parameters.max_past_logs;
	let target_gas_price = cli.run.dynamic_fee_parameters.target_gas_price;

	match &cli.subcommand {
		None => {
			let authority_discovery_disabled = cli.run.authority_discovery_disabled;
			let runner = cli
				.create_runner(&cli.run.base)
				.map_err(sc_cli::Error::from)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			log::info!("  _____                      _       _       ");
			log::info!(" |  __ \\                    (_)     (_)      ");
			log::info!(" | |  | | __ _ _ ____      ___ _ __  _  __ _ ");
			log::info!(" | |  | |/ _` | '__\\ \\ /\\ / / | '_ \\| |/ _` |");
			log::info!(" | |__| | (_| | |   \\ V  V /| | | | | | (_| |");
			log::info!(" |_____/ \\__,_|_|    \\_/\\_/ |_|_| |_|_|\\__,_|");

			if chain_spec.is_crab() {
				runner.run_node_until_exit(|config| async move {
					match config.role {
						Role::Light => {
							crab::crab_new_light(config).map(|(task_manager, _)| task_manager)
						}
						_ => crab::crab_new_full(
							config,
							authority_discovery_disabled,
							max_past_logs,
							target_gas_price,
						)
						.map(|(task_manager, _, _)| task_manager),
					}
					.map_err(sc_cli::Error::Service)
				})
			} else {
				runner.run_node_until_exit(|config| async move {
					match config.role {
						Role::Light => darwinia::darwinia_new_light(config)
							.map(|(task_manager, _)| task_manager),
						_ => darwinia::darwinia_new_full(config, authority_discovery_disabled)
							.map(|(task_manager, _, _)| task_manager),
					}
					.map_err(sc_cli::Error::Service)
				})
			}
		}
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.async_run(|mut config| {
					let (client, _, import_queue, task_manager) =
						crab::new_chain_ops::<crab_runtime::RuntimeApi, CrabExecutor>(
							&mut config,
							max_past_logs,
							target_gas_price,
						)?;

					Ok((cmd.run(client, import_queue), task_manager))
				})
			} else {
				runner.async_run(|mut config| {
					let (client, _, import_queue, task_manager) = darwinia::new_chain_ops::<
						darwinia_runtime::RuntimeApi,
						DarwiniaExecutor,
					>(&mut config)?;

					Ok((cmd.run(client, import_queue), task_manager))
				})
			}
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.async_run(|mut config| {
					let (client, _, _, task_manager) =
						crab::new_chain_ops::<crab_runtime::RuntimeApi, CrabExecutor>(
							&mut config,
							max_past_logs,
							target_gas_price,
						)?;

					Ok((cmd.run(client, config.database), task_manager))
				})
			} else {
				runner.async_run(|mut config| {
					let (client, _, _, task_manager) = darwinia::new_chain_ops::<
						darwinia_runtime::RuntimeApi,
						DarwiniaExecutor,
					>(&mut config)?;

					Ok((cmd.run(client, config.database), task_manager))
				})
			}
		}
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.async_run(|mut config| {
					let (client, _, _, task_manager) =
						crab::new_chain_ops::<crab_runtime::RuntimeApi, CrabExecutor>(
							&mut config,
							max_past_logs,
							target_gas_price,
						)?;

					Ok((cmd.run(client, config.chain_spec), task_manager))
				})
			} else {
				runner.async_run(|mut config| {
					let (client, _, _, task_manager) = darwinia::new_chain_ops::<
						darwinia_runtime::RuntimeApi,
						DarwiniaExecutor,
					>(&mut config)?;

					Ok((cmd.run(client, config.chain_spec), task_manager))
				})
			}
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.async_run(|mut config| {
					let (client, _, import_queue, task_manager) =
						crab::new_chain_ops::<crab_runtime::RuntimeApi, CrabExecutor>(
							&mut config,
							max_past_logs,
							target_gas_price,
						)?;

					Ok((cmd.run(client, import_queue), task_manager))
				})
			} else {
				runner.async_run(|mut config| {
					let (client, _, import_queue, task_manager) = darwinia::new_chain_ops::<
						darwinia_runtime::RuntimeApi,
						DarwiniaExecutor,
					>(&mut config)?;

					Ok((cmd.run(client, import_queue), task_manager))
				})
			}
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			if chain_spec.is_crab() {
				runner.sync_run(|config| {
					// <--- dvm ---
					// Remove dvm offchain db
					let dvm_database_config = sc_service::DatabaseConfig::RocksDb {
						path: darwinia_service::crab::dvm_database_dir(&config),
						cache_size: 0,
					};

					cmd.run(dvm_database_config)?;
					// --- dvm --->

					cmd.run(config.database)
				})
			} else {
				runner.sync_run(|config| cmd.run(config.database))
			}
		}
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runner.async_run(|mut config| {
					let (client, backend, _, task_manager) =
						crab::new_chain_ops::<crab_runtime::RuntimeApi, CrabExecutor>(
							&mut config,
							max_past_logs,
							target_gas_price,
						)?;

					Ok((cmd.run(client, backend), task_manager))
				})
			} else {
				runner.async_run(|mut config| {
					let (client, backend, _, task_manager) = darwinia::new_chain_ops::<
						darwinia_runtime::RuntimeApi,
						DarwiniaExecutor,
					>(&mut config)?;

					Ok((cmd.run(client, backend), task_manager))
				})
			}
		}
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::Sign(cmd)) => cmd.run(),
		Some(Subcommand::Verify(cmd)) => cmd.run(),
		Some(Subcommand::Vanity(cmd)) => cmd.run(),
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			if chain_spec.is_crab() {
				runner.async_run(|config| {
					let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
					// we don't need any of the components of new_partial, just a runtime, or a task
					// manager to do `async_run`.
					let task_manager = TaskManager::new(config.task_executor.clone(), registry)
						.map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;

					Ok((
						cmd.run::<crab_runtime::Block, CrabExecutor>(config),
						task_manager,
					))
				})
			} else {
				runner.async_run(|config| {
					let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
					// we don't need any of the components of new_partial, just a runtime, or a task
					// manager to do `async_run`.
					let task_manager = TaskManager::new(config.task_executor.clone(), registry)
						.map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;

					Ok((
						cmd.run::<darwinia_runtime::Block, DarwiniaExecutor>(config),
						task_manager,
					))
				})
			}
		}
		#[cfg(feature = "runtime-benchmarks")]
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			if chain_spec.is_crab() {
				runner.sync_run(|config| cmd.run::<crab_runtime::Block, CrabExecutor>(config))
			} else if chain_spec.is_darwinia() {
				runner
					.sync_run(|config| cmd.run::<darwinia_runtime::Block, DarwiniaExecutor>(config))
			} else {
				Err("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
					.into())
			}
		}
	}
}
