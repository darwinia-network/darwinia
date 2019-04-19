
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

extern crate evo_service as service;

use substrate_cli as cli;
use exit_future;

#[macro_use]
extern crate log;

mod chain_spec;

use std::ops::Deref;
use chain_spec::Alternative;
use futures::Future;
use tokio::runtime::Runtime;


pub use service::{ServiceFactory, Factory};

pub use cli::{VersionInfo, IntoExit, NoCustom};
pub use cli::error;
pub use tokio::runtime::TaskExecutor;

fn load_spec(id: &str) -> Result<Option<service::ChainSpec>, String> {
	Ok(match Alternative::from(id) {
		Some(spec) => Some(spec.load()?),
		None => None,
	})
}

/// Parse command line arguments into service configuration.
pub fn run<I, T, E>(args: I, exit: E, version: cli::VersionInfo) -> error::Result<()> where
	I: IntoIterator<Item = T>,
	T: Into<std::ffi::OsString> + Clone,
	E: IntoExit,
{
	cli::parse_and_execute::<service::Factory, NoCustom, NoCustom, _, _, _, _, _>(
		load_spec, &version, "evo-node", args, exit,
		|exit, _custom_args, config| {
			info!("{}", version.name);
			info!("  version {}", config.full_version());
			info!("  by {}, 2018-2019", version.author);
			info!("Chain specification: {}", config.chain_spec.name());
			info!("Node name: {}", config.name);
			info!("Roles: {:?}", config.roles);
			let runtime = Runtime::new().map_err(|e| format!("{:?}", e))?;
			let executor = runtime.executor();
			match config.roles {
				service::Roles::LIGHT =>
					run_until_exit(
						runtime,
						Factory::new_light(config, executor).map_err(|e| format!("{:?}", e))?,
						exit
					),
				_ => run_until_exit(
						runtime,
						Factory::new_full(config, executor).map_err(|e| format!("{:?}", e))?,
						exit
					),
			}.map_err(|e| format!("{:?}", e))
		}
	).map_err(Into::into).map(|_| ())
}

fn run_until_exit<T, C, E>(
	mut runtime: Runtime,
	service: T,
	e: E,
) -> error::Result<()>
	where
	    T: Deref<Target=service::Service<C>>,
		C: service::Components,
		E: IntoExit,
{
	let (exit_send, exit) = exit_future::signal();

	let executor = runtime.executor();
	cli::informant::start(&service, exit.clone(), executor.clone());

	let _ = runtime.block_on(e.into_exit());
	exit_send.fire();

	// we eagerly drop the service so that the internal exit future is fired,
	// but we need to keep holding a reference to the global telemetry guard
	let _telemetry = service.telemetry();
	drop(service);

	// TODO [andre]: timeout this future #1318
	let _ = runtime.shutdown_on_idle().wait();

	Ok(())
}