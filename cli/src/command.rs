// --- third-party ---
use log::info;
// --- substrate ---
use sc_cli::VersionInfo;
use sc_executor::NativeExecutionDispatch;
use sp_api::ConstructRuntimeApi;
use sp_runtime::traits::BlakeTwo256;
// --- darwinia ---
use crate::{
	chain_spec::load_spec,
	cli::{Cli, Subcommand},
};
use darwinia_service::{Block, RuntimeApiCollection, TFullClient};

/// Parses polkadot specific CLI arguments and run the service.
pub fn run(version: VersionInfo) -> sc_cli::Result<()> {
	let opt = sc_cli::from_args::<Cli>(&version);

	let mut config = darwinia_service::Configuration::from_version(&version);
	config.impl_name = "darwinia-network-darwinia";

	match opt.subcommand {
		None => {
			opt.run.base.init(&version)?;
			opt.run
				.base
				.update_config(&mut config, |id| load_spec(id), &version)?;

			info!("{}", version.name);
			info!("  version {}", config.full_version());
			info!("  _____                      _       _       ");
			info!(" |  __ \\                    (_)     (_)      ");
			info!(" | |  | | __ _ _ ____      ___ _ __  _  __ _ ");
			info!(" | |  | |/ _` | '__\\ \\ /\\ / / | '_ \\| |/ _` |");
			info!(" | |__| | (_| | |   \\ V  V /| | | | | | (_| |");
			info!(" |_____/ \\__,_|_|    \\_/\\_/ |_|_| |_|_|\\__,_|");
			info!("  by {}, 2018-2020", version.author);
			info!(
				"📋 Chain specification: {}",
				config.expect_chain_spec().name()
			);
			info!("🏷 Node name: {}", config.name);
			info!("👤 Roles: {}", config.display_role());

			info!(
				"⛓ Native runtime: {}",
				darwinia_service::CrabExecutor::native_version().runtime_version
			);

			run_service_until_exit::<
				darwinia_service::crab_runtime::RuntimeApi,
				darwinia_service::CrabExecutor,
				darwinia_service::crab_runtime::UncheckedExtrinsic,
			>(config)
		}
		Some(Subcommand::Base(cmd)) => {
			cmd.init(&version)?;
			cmd.update_config(&mut config, |id| load_spec(id), &version)?;
			cmd.run(
				config,
				darwinia_service::new_chain_ops::<
					darwinia_service::crab_runtime::RuntimeApi,
					darwinia_service::CrabExecutor,
					darwinia_service::crab_runtime::UncheckedExtrinsic,
				>,
			)
		} // TODO: benchmark
		  // Some(Subcommand::Benchmark(cmd)) => {
		  // 	cmd.init(&version)?;
		  // 	cmd.update_config(&mut config, |id| load_spec(id), &version)?;
		  // 	cmd.run::<darwinia_service::crab_runtime::Block, darwinia_service::CrabExecutor>(config)
		  // }
	}
}

fn run_service_until_exit<R, D, E>(config: darwinia_service::Configuration) -> sc_cli::Result<()>
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
	match config.roles {
		darwinia_service::Roles::LIGHT => sc_cli::run_service_until_exit(config, |config| {
			darwinia_service::new_light::<R, D, E>(config)
		}),
		_ => sc_cli::run_service_until_exit(config, |config| {
			darwinia_service::new_full::<R, D, E>(config)
		}),
	}
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
