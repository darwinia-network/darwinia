//! Predefined chains.

/// The chain specification (this should eventually be replaced by a more general JSON-based chain
/// specification).
#[derive(Clone, Debug)]
pub enum ChainSpec {
	/// Whatever the current Crab runtime is, with just Alice as an auth.
	CrabDevelopment,
	/// Whatever the current Crab runtime is, with simple Alice/Bob auths.
	CrabLocalTestnet,
	/// Whatever the current Crab runtime is with the "global testnet" defaults.
	CrabStagingTestnet,
	/// The Crab network.
	Crab,
}

impl Default for ChainSpec {
	fn default() -> Self {
		ChainSpec::Crab
	}
}

/// Get a chain config from a spec setting.
impl ChainSpec {
	pub(crate) fn load(self) -> Result<Box<dyn darwinia_service::ChainSpec>, String> {
		// --- darwinia ---
		use darwinia_service::chain_spec::{
			crab_config, crab_development_config, crab_local_testnet_config,
			crab_staging_testnet_config,
		};
		Ok(match self {
			ChainSpec::CrabDevelopment => Box::new(crab_development_config()),
			ChainSpec::CrabLocalTestnet => Box::new(crab_local_testnet_config()),
			ChainSpec::CrabStagingTestnet => Box::new(crab_staging_testnet_config()),
			ChainSpec::Crab => Box::new(crab_config()?),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"crab-dev" => Some(ChainSpec::CrabDevelopment),
			"crab-local" => Some(ChainSpec::CrabLocalTestnet),
			"crab-staging" => Some(ChainSpec::CrabStagingTestnet),
			"crab" => Some(ChainSpec::Crab),
			"" => Some(ChainSpec::default()),
			_ => None,
		}
	}
}

/// Load the `ChainSpec` for the given `id`.
pub fn load_spec(id: &str) -> Result<Box<dyn darwinia_service::ChainSpec>, String> {
	Ok(match ChainSpec::from(id) {
		Some(spec) => spec.load()?,
		None => Box::new(darwinia_service::CrabChainSpec::from_json_file(
			std::path::PathBuf::from(id),
		)?),
	})
}
