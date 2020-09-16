// --- std ---
use std::path::PathBuf;
// --- crates ---
use log::info;
// --- substrate ---
use sc_cli::{Role, RunCmd, RuntimeVersion, SubstrateCli};
use sp_core::crypto::Ss58AddressFormat;
// --- darwinia ---
use crate::cli::{Cli, Subcommand};
use darwinia_cli::{Configuration, DarwiniaCli};
use darwinia_service::{crab_runtime, darwinia_runtime, IdentifyVariant};

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

	fn native_runtime_version(
		spec: &Box<dyn darwinia_service::ChainSpec>,
	) -> &'static RuntimeVersion {
		if spec.is_crab() {
			&darwinia_service::crab_runtime::VERSION
		} else if spec.is_darwinia() {
			&darwinia_service::darwinia_runtime::VERSION
		} else {
			&darwinia_service::darwinia_runtime::VERSION
		}
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
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
			"crab-dev" => Box::new(darwinia_service::chain_spec::crab_development_config()),
			"crab-local" => Box::new(darwinia_service::chain_spec::crab_local_testnet_config()),
			"crab-genesis" => Box::new(darwinia_service::chain_spec::crab_build_spec_config()),
			"crab" => Box::new(darwinia_service::chain_spec::crab_config()?),
			// "darwinia-dev" | "dev" => {
			// Box::new(darwinia_service::chain_spec::darwinia_development_config())
			// }
			// "darwinia-local" => {
			// Box::new(darwinia_service::chain_spec::darwinia_local_testnet_config())
			// }
			"darwinia-genesis" => {
				Box::new(darwinia_service::chain_spec::darwinia_build_spec_config())
			}
			"darwinia" => Box::new(darwinia_service::chain_spec::darwinia_config()?),
			path if self.run.force_crab => Box::new(
				darwinia_service::CrabChainSpec::from_json_file(std::path::PathBuf::from(path))?,
			),
			path => Box::new(darwinia_service::DarwiniaChainSpec::from_json_file(
				std::path::PathBuf::from(path),
			)?),
		})
	}
}

impl DarwiniaCli for Cli {
	fn conf(&self) -> &Option<PathBuf> {
		&self.conf
	}

	fn base(&self) -> &RunCmd {
		&self.run.base
	}

	fn mut_base(&mut self) -> &mut RunCmd {
		&mut self.run.base
	}
}

fn get_exec_name() -> Option<String> {
	std::env::current_exe()
		.ok()
		.and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
		.and_then(|s| s.into_string().ok())
}

/// Parses Darwinia specific CLI arguments and run the service.
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	fn set_default_ss58_version(spec: &Box<dyn darwinia_service::ChainSpec>) {
		let ss58_version = if spec.is_crab() {
			Ss58AddressFormat::SubstrateAccount
		} else if spec.is_darwinia() {
			Ss58AddressFormat::DarwiniaAccount
		} else {
			Ss58AddressFormat::DarwiniaAccount
		};

		sp_core::crypto::set_default_ss58_version(ss58_version);
	};

	match &cli.subcommand {
		None => {
			let runtime = Configuration::create_runner(cli)?;
			let chain_spec = &runtime.config().chain_spec;

			set_default_ss58_version(chain_spec);

			info!("  _____                      _       _       ");
			info!(" |  __ \\                    (_)     (_)      ");
			info!(" | |  | | __ _ _ ____      ___ _ __  _  __ _ ");
			info!(" | |  | |/ _` | '__\\ \\ /\\ / / | '_ \\| |/ _` |");
			info!(" | |__| | (_| | |   \\ V  V /| | | | | | (_| |");
			info!(" |_____/ \\__,_|_|    \\_/\\_/ |_|_| |_|_|\\__,_|");

			if chain_spec.is_crab() {
				runtime.run_node_until_exit(|config| match config.role {
					Role::Light => darwinia_service::crab_new_light(config),
					_ => darwinia_service::crab_new_full(config).map(|(components, _)| components),
				})
			} else if chain_spec.is_darwinia() {
				runtime.run_node_until_exit(|config| match config.role {
					Role::Light => darwinia_service::darwinia_new_light(config),
					_ => darwinia_service::darwinia_new_full(config)
						.map(|(components, _)| components),
				})
			} else {
				unreachable!()
			}
		}
		Some(Subcommand::Base(subcommand)) => {
			let runtime = cli.create_runner(subcommand)?;
			let chain_spec = &runtime.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab() {
				runtime.run_subcommand(subcommand, |config| {
					darwinia_service::new_chain_ops::<
						crab_runtime::RuntimeApi,
						darwinia_service::CrabExecutor,
					>(config)
				})
			} else if chain_spec.is_darwinia() {
				runtime.run_subcommand(subcommand, |config| {
					darwinia_service::new_chain_ops::<
						darwinia_runtime::RuntimeApi,
						darwinia_service::DarwiniaExecutor,
					>(config)
				})
			} else {
				runtime.run_subcommand(subcommand, |config| {
					darwinia_service::new_chain_ops::<
						darwinia_runtime::RuntimeApi,
						darwinia_service::DarwiniaExecutor,
					>(config)
				})
			}
		}
		Some(Subcommand::Key(cmd)) => cmd.run(),
		Some(Subcommand::Sign(cmd)) => cmd.run(),
		Some(Subcommand::Verify(cmd)) => cmd.run(),
		Some(Subcommand::Vanity(cmd)) => cmd.run(),
	}
}
