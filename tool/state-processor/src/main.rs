// Processor components.
mod processor;
use processor::*;
mod util;
use util::*;

// Runtime configurations.
mod adjust;
use adjust::*;
mod configuration;
use configuration::*;
mod type_registry;
use type_registry::*;

// Runtime pallets.
mod balances;
mod evm;
mod indices;
mod proxy;
mod staking;
mod sudo;
mod system;
mod vesting;

#[cfg(test)]
mod tests;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = anyhow::Result<T>;

fn main() -> Result<()> {
	std::env::set_var("RUST_LOG", "state_processor");
	pretty_env_logger::init();

	// <Processor<Darwinia>>::new()?.process()?;
	// <Processor<Crab>>::new()?.process()?;
	<Processor<Pangolin>>::new()?.test().process()?;

	Ok(())
}
