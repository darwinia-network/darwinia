// --- std ---
use std::path::PathBuf;
// --- crates ---
use log::info;
// --- substrate ---
use sc_cli::{RunCmd, SubstrateCli};
use sc_executor::NativeExecutionDispatch;
// --- darwinia ---
use crate::cli::{Cli, Subcommand};
use darwinia_cli::{Configuration, DarwiniaCli};
use darwinia_service::{crab_runtime, IdentifyVariant};

impl SubstrateCli for Cli {
	fn impl_name() -> &'static str {
		"Crab"
	}

	fn impl_version() -> &'static str {
		env!("SUBSTRATE_CLI_IMPL_VERSION")
	}

	fn executable_name() -> &'static str {
		"darwinia"
	}

	fn description() -> &'static str {
		env!("CARGO_PKG_DESCRIPTION")
	}

	fn author() -> &'static str {
		env!("CARGO_PKG_AUTHORS")
	}

	fn support_url() -> &'static str {
		"https://github.com/darwinia-network/darwinia/issues/new"
	}

	fn copyright_start_year() -> i32 {
		2018
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		let id = if id.is_empty() {
			let n = get_exec_name().unwrap_or_default();
			["crab"]
				.iter()
				.cloned()
				.find(|&chain| n.starts_with(chain))
				.unwrap_or("crab")
		} else {
			id
		};

		Ok(match id.to_lowercase().as_ref() {
			"crab-dev" | "dev" => Box::new(darwinia_service::chain_spec::crab_development_config()),
			"crab-local" => Box::new(darwinia_service::chain_spec::crab_local_testnet_config()),
			"crab-genesis" => Box::new(darwinia_service::chain_spec::crab_build_spec_config()),
			"crab" => Box::new(darwinia_service::chain_spec::crab_config()?),
			path => Box::new(darwinia_service::CrabChainSpec::from_json_file(
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
		// --- substrate ---
		use sp_core::crypto::Ss58AddressFormat;

		let ss58_version = if spec.is_crab() {
			Ss58AddressFormat::SubstrateAccount
		} else {
			Ss58AddressFormat::DarwiniaAccount
		};

		sp_core::crypto::set_default_ss58_version(ss58_version);
	};

	match &cli.subcommand {
		None => {
			let runtime = Configuration::create_runner_from_cli(cli)?;
			let chain_spec = &runtime.config().chain_spec;

			set_default_ss58_version(chain_spec);

			info!("  _____                      _       _       ");
			info!(" |  __ \\                    (_)     (_)      ");
			info!(" | |  | | __ _ _ ____      ___ _ __  _  __ _ ");
			info!(" | |  | |/ _` | '__\\ \\ /\\ / / | '_ \\| |/ _` |");
			info!(" | |__| | (_| | |   \\ V  V /| | | | | | (_| |");
			info!(" |_____/ \\__,_|_|    \\_/\\_/ |_|_| |_|_|\\__,_|");

			if chain_spec.is_crab() {
				runtime.run_node(
					|config| darwinia_service::crab_new_light(config),
					|config| darwinia_service::crab_new_full(config),
					darwinia_service::CrabExecutor::native_version().runtime_version,
				)
			} else {
				runtime.run_node(
					|config| darwinia_service::crab_new_light(config),
					|config| darwinia_service::crab_new_full(config),
					darwinia_service::CrabExecutor::native_version().runtime_version,
				)
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
						crab_runtime::UncheckedExtrinsic,
					>(config)
				})
			} else {
				runtime.run_subcommand(subcommand, |config| {
					darwinia_service::new_chain_ops::<
						crab_runtime::RuntimeApi,
						darwinia_service::CrabExecutor,
						crab_runtime::UncheckedExtrinsic,
					>(config)
				})
			}
		}
	}
}
