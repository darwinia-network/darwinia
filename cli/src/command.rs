// --- third-party ---
use log::info;
// --- substrate ---
use sc_cli::{Result, SubstrateCli};
use sc_executor::NativeExecutionDispatch;
use sp_api::ConstructRuntimeApi;
use sp_runtime::traits::BlakeTwo256;
// --- darwinia ---
use crate::{
	chain_spec::load_spec,
	cli::{Cli, Subcommand},
};
use darwinia_service::{Block, RuntimeApiCollection, TFullClient};

impl SubstrateCli for Cli {
	fn impl_name() -> &'static str {
		"darwinia-network"
	}

	fn impl_version() -> &'static str {
		env!("DARWINIA_CLI_IMPL_VERSION")
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

	fn executable_name() -> &'static str {
		"darwinia"
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"crab-dev" | "dev" => Box::new(crab_development_config()),
			"crab-local" => Box::new(crab_local_testnet_config()),
			"crab-genesis" => Box::new(crab_genesis_builder_config()),
			"crab" | "" => Box::new(crab_config()?),
			path => Box::new(darwinia_service::CrabChainSpec::from_json_file(
				std::path::PathBuf::from(path),
			)?),
		})
	}
}

/// Parses Darwinia specific CLI arguments and run the service.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		None => {
			let runtime = cli.create_runner(&cli.run.base)?;
			let config = runtime.config();

			info!(
				"⛓  Native runtime: {}",
				darwinia_service::CrabExecutor::native_version().runtime_version
			);
			info!("  _____                      _       _       ");
			info!(" |  __ \\                    (_)     (_)      ");
			info!(" | |  | | __ _ _ ____      ___ _ __  _  __ _ ");
			info!(" | |  | |/ _` | '__\\ \\ /\\ / / | '_ \\| |/ _` |");
			info!(" | |__| | (_| | |   \\ V  V /| | | | | | (_| |");
			info!(" |_____/ \\__,_|_|    \\_/\\_/ |_|_| |_|_|\\__,_|");
			info!("  by Darwinia-Network, 2018-2020");

			run_node::<
				darwinia_service::crab_runtime::RuntimeApi,
				darwinia_service::CrabExecutor,
				darwinia_service::crab_runtime::UncheckedExtrinsic,
			>(runtime)
		}
		Some(Subcommand::Base(cmd)) => {
			let runtime = cli.create_runner(subcommand)?;

			runtime.run_subcommand(subcommand, |config| {
				service::new_chain_ops::<
					darwinia_service::crab_runtime::RuntimeApi,
					darwinia_service::CrabExecutor,
					darwinia_service::crab_runtime::UncheckedExtrinsic,
				>(config)
			})
		}
		Some(_) => {
			// TODO: benchmark
			unimplemented!()
		}
	}
}

fn run_node<R, D, E>(runtime: sc_cli::Runner<Cli>) -> sc_cli::Result<()>
where
	R: ConstructRuntimeApi<Block, darwinia_service::TFullClient<Block, R, D>>
		+ Send
		+ Sync
		+ 'static,
	<R as ConstructRuntimeApi<Block, darwinia_service::TFullClient<Block, R, D>>>::RuntimeApi:
		RuntimeApiCollection<
			E,
			StateBackend = sc_client_api::StateBackendFor<
				darwinia_service::TFullBackend<Block>,
				Block,
			>,
		>,
	<R as ConstructRuntimeApi<Block, darwinia_service::TLightClient<Block, R, D>>>::RuntimeApi:
		RuntimeApiCollection<
			E,
			StateBackend = sc_client_api::StateBackendFor<
				darwinia_service::TLightBackend<Block>,
				Block,
			>,
		>,
	E: darwinia_service::Codec + Send + Sync + 'static,
	D: darwinia_service::NativeExecutionDispatch + 'static,
	// Rust bug: https://github.com/rust-lang/rust/issues/24159
	<<R as ConstructRuntimeApi<Block, TFullClient<Block, R, D>>>::RuntimeApi as sp_api::ApiExt<
		Block,
	>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
	// Rust bug: https://github.com/rust-lang/rust/issues/43580
	R: ConstructRuntimeApi<Block, TLightClient<R, D>>,
{
	runtime.run_node(
		|config| darwinia_service::new_light::<R, D, E>(config),
		|config| darwinia_service::new_full::<R, D, E>(config),
	)
}

// We can't simply use `darwinia_service::TLightClient` due to a
// Rust bug: https://github.com/rust-lang/rust/issues/43580
type TLightClient<Runtime, Dispatch> = sc_client::Client<
	sc_client::light::backend::Backend<sc_client_db::light::LightStorage<Block>, BlakeTwo256>,
	sc_client::light::call_executor::GenesisCallExecutor<
		sc_client::light::backend::Backend<sc_client_db::light::LightStorage<Block>, BlakeTwo256>,
		sc_client::LocalCallExecutor<
			sc_client::light::backend::Backend<
				sc_client_db::light::LightStorage<Block>,
				BlakeTwo256,
			>,
			sc_executor::NativeExecutor<Dispatch>,
		>,
	>,
	Block,
	Runtime,
>;
