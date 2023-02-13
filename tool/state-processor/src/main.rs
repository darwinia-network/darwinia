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
mod identity;
mod parachain_system;
mod staking;
mod system;
mod vesting;

#[cfg(test)]
mod tests;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = anyhow::Result<T>;

fn main() -> Result<()> {
	std::env::set_var("RUST_LOG", "state_processor");
	pretty_env_logger::init();

	// <Processor<Pangolin>>::new()?.process().save()?;
	// <Processor<Pangolin>>::new()?.test().process().save()?;
	// <Processor<Pangoro>>::new()?.process().save()?;
	// <Processor<Pangoro>>::new()?.test().process().save()?;
	<Processor<Crab>>::new()?.process().save()?;
	// <Processor<Crab>>::new()?.test().process().save()?;
	// <Processor<Darwinia>>::new()?.process().save()?;
	// <Processor<Darwinia>>::new()?.test().process().save()?;

	Ok(())
}
