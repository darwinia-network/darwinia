use crate::factory_impl::FactoryState;
use crate::{load_spec, service, ChainSpec};
use log::info;
use node_transaction_factory::RuntimeAdapter;
pub use sc_cli::VersionInfo;
use sc_cli::{display_role, parse_and_prepare, GetSharedParams, ParseAndPrepare};
use sc_cli::{error, ImportParams, IntoExit, NoCustom, SharedParams};
use sc_service::{AbstractService, Configuration, Roles as ServiceRoles};
use structopt::StructOpt;
use tokio::prelude::Future;
use tokio::runtime::{Builder as RuntimeBuilder, Runtime};

/// Custom subcommands.
#[derive(Clone, Debug, StructOpt)]
pub enum CustomSubcommands {
	/// The custom factory subcommmand for manufacturing transactions.
	#[structopt(
		name = "factory",
		about = "Manufactures num transactions from Alice to random accounts. \
		Only supported for development or local testnet."
	)]
	Factory(FactoryCmd),
}

impl GetSharedParams for CustomSubcommands {
	fn shared_params(&self) -> Option<&SharedParams> {
		match self {
			CustomSubcommands::Factory(cmd) => Some(&cmd.shared_params),
		}
	}
}

/// The `factory` command used to generate transactions.
/// Please note: this command currently only works on an empty database!
#[derive(Debug, StructOpt, Clone)]
pub struct FactoryCmd {
	/// How often to repeat. This option only has an effect in mode `MasterToNToM`.
	#[structopt(long = "rounds", default_value = "1")]
	pub rounds: u64,

	/// MasterToN: Manufacture `num` transactions from the master account
	///            to `num` randomly created accounts, one each.
	///
	/// MasterTo1: Manufacture `num` transactions from the master account
	///            to exactly one other randomly created account.
	///
	/// MasterToNToM: Manufacture `num` transactions from the master account
	///               to `num` randomly created accounts.
	///               From each of these randomly created accounts manufacture
	///               a transaction to another randomly created account.
	///               Repeat this `rounds` times. If `rounds` = 1 the behavior
	///               is the same as `MasterToN`.{n}
	///               A -> B, A -> C, A -> D, ... x `num`{n}
	///               B -> E, C -> F, D -> G, ...{n}
	///               ... x `rounds`
	///
	/// These three modes control manufacturing.
	#[structopt(long = "mode", default_value = "MasterToN")]
	pub mode: node_transaction_factory::Mode,

	/// Number of transactions to generate. In mode `MasterNToNToM` this is
	/// the number of transactions per round.
	#[structopt(long = "num", default_value = "8")]
	pub num: u64,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub shared_params: SharedParams,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub import_params: ImportParams,
}

/// Parse command line arguments into service configuration.
pub fn run<I, T, E>(args: I, exit: E, version: sc_cli::VersionInfo) -> error::Result<()>
where
	I: IntoIterator<Item = T>,
	T: Into<std::ffi::OsString> + Clone,
	E: IntoExit,
{
	type Config<A, B> = Configuration<(), A, B>;

	match parse_and_prepare::<CustomSubcommands, NoCustom, _>(&version, "darwinia-node", args) {
		ParseAndPrepare::Run(cmd) => cmd.run(
			load_spec,
			exit,
			|exit, _cli_args, _custom_args, config: Config<_, _>| {
				info!("{}", version.name);
				info!("  version {}", config.full_version());
				info!("  _____                      _       _       ");
				info!(" |  __ \\                    (_)     (_)      ");
				info!(" | |  | | __ _ _ ____      ___ _ __  _  __ _ ");
				info!(" | |  | |/ _` | '__\\ \\ /\\ / / | '_ \\| |/ _` |");
				info!(" | |__| | (_| | |   \\ V  V /| | | | | | (_| |");
				info!(" |_____/ \\__,_|_|    \\_/\\_/ |_|_| |_|_|\\__,_|");
				info!("  by Darwinia Network, 2017-2019");
				info!("Chain specification: {}", config.chain_spec.name());
				info!("Node name: {}", config.name);
				info!("Roles: {}", display_role(&config));
				let runtime = RuntimeBuilder::new()
					.name_prefix("main-tokio-")
					.build()
					.map_err(|e| format!("{:?}", e))?;
				match config.roles {
					ServiceRoles::LIGHT => run_until_exit(runtime, service::new_light(config)?, exit),
					_ => run_until_exit(runtime, service::new_full(config)?, exit),
				}
			},
		),
		ParseAndPrepare::BuildSpec(cmd) => cmd.run::<NoCustom, _, _, _>(load_spec),
		ParseAndPrepare::ExportBlocks(cmd) => {
			cmd.run_with_builder(|config: Config<_, _>| Ok(new_full_start!(config).0), load_spec, exit)
		}
		ParseAndPrepare::ImportBlocks(cmd) => {
			cmd.run_with_builder(|config: Config<_, _>| Ok(new_full_start!(config).0), load_spec, exit)
		}
		ParseAndPrepare::CheckBlock(cmd) => {
			cmd.run_with_builder(|config: Config<_, _>| Ok(new_full_start!(config).0), load_spec, exit)
		}
		ParseAndPrepare::PurgeChain(cmd) => cmd.run(load_spec),
		ParseAndPrepare::RevertChain(cmd) => {
			cmd.run_with_builder(|config: Config<_, _>| Ok(new_full_start!(config).0), load_spec)
		}
		ParseAndPrepare::CustomCommand(CustomSubcommands::Factory(cli_args)) => {
			let mut config: Config<_, _> =
				sc_cli::create_config_with_db_path(load_spec, &cli_args.shared_params, &version)?;

			sc_cli::fill_import_params(&mut config, &cli_args.import_params, ServiceRoles::FULL)?;

			match ChainSpec::from(config.chain_spec.id()) {
				Some(ref c) if c == &ChainSpec::Development || c == &ChainSpec::LocalTestnet => {}
				_ => panic!("Factory is only supported for development and local testnet."),
			}

			let factory_state = FactoryState::new(cli_args.mode.clone(), cli_args.num, cli_args.rounds);

			let service_builder = new_full_start!(config).0;
			node_transaction_factory::factory::<FactoryState<_>, _, _, _, _, _>(
				factory_state,
				service_builder.client(),
				service_builder
					.select_chain()
					.expect("The select_chain is always initialized by new_full_start!; QED"),
			)
			.map_err(|e| format!("Error in transaction factory: {}", e))?;

			Ok(())
		}
	}
}

fn run_until_exit<T, E>(mut runtime: Runtime, service: T, e: E) -> error::Result<()>
where
	T: AbstractService,
	E: IntoExit,
{
	use futures::{channel::oneshot, compat::Future01CompatExt, future::select, FutureExt, TryFutureExt};

	let (exit_send, exit) = oneshot::channel();

	let informant = sc_cli::informant::build(&service);

	let future = select(informant, exit).map(|_| Ok(())).compat();

	runtime.executor().spawn(future);

	// we eagerly drop the service so that the internal exit future is fired,
	// but we need to keep holding a reference to the global telemetry guard
	let _telemetry = service.telemetry();

	let service_res = {
		let exit = e.into_exit();
		let service = service.map_err(|err| error::Error::Service(err)).compat();
		let select = select(service, exit).map(|_| Ok(())).compat();
		runtime.block_on(select)
	};

	let _ = exit_send.send(());

	// TODO [andre]: timeout this future #1318
	let _ = runtime.shutdown_on_idle().wait();

	service_res
}
