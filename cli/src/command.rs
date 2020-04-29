// --- crates ---
use log::info;
// --- substrate ---
use sc_cli::SubstrateCli;
use sc_executor::NativeExecutionDispatch;
// --- darwinia ---
use crate::cli::{Cli, Subcommand};
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
		Ok(match id {
			"crab-dev" | "dev" => Box::new(darwinia_service::chain_spec::crab_development_config()),
			"crab-local" => Box::new(darwinia_service::chain_spec::crab_local_testnet_config()),
			"crab-genesis" => Box::new(darwinia_service::chain_spec::crab_build_spec_config()),
			"crab" | "" => Box::new(darwinia_service::chain_spec::crab_config()?),
			path => Box::new(darwinia_service::CrabChainSpec::from_json_file(
				std::path::PathBuf::from(path),
			)?),
		})
	}
}

/// Parses Darwinia specific CLI arguments and run the service.
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	if let Some(path) = &cli.conf {
		if path.is_file() {
			// TODO: load boot conf from file
		}
	}

	match &cli.subcommand {
		None => {
			let runtime = cli.create_runner(&cli.run.base)?;
			let config = runtime.config();

			info!("  _____                      _       _       ");
			info!(" |  __ \\                    (_)     (_)      ");
			info!(" | |  | | __ _ _ ____      ___ _ __  _  __ _ ");
			info!(" | |  | |/ _` | '__\\ \\ /\\ / / | '_ \\| |/ _` |");
			info!(" | |__| | (_| | |   \\ V  V /| | | | | | (_| |");
			info!(" |_____/ \\__,_|_|    \\_/\\_/ |_|_| |_|_|\\__,_|");

			if config.chain_spec.is_crab() {
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

			if runtime.config().chain_spec.is_crab() {
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
		} // TODO: benchmark
		  // Some(Subcommand::Benchmark(cmd)) => {
		  // 	cmd.init(&version)?;
		  // 	cmd.update_config(&mut config, |id| load_spec(id), &version)?;
		  // 	cmd.run::<darwinia_service::crab_runtime::Block, darwinia_service::CrabExecutor>(config)
		  // }
	}
}
