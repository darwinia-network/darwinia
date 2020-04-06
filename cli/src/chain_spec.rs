//! Predefined chains.

/// The chain specification (this should eventually be replaced by a more general JSON-based chain
/// specification).
#[derive(Clone, Debug)]
pub enum ChainSpec {
	/// Whatever the current Crab runtime is, with just Alice as an auth.
	CrabDevelopment,
	/// Whatever the current Crab runtime is, with simple Alice/Bob auths.
	CrabLocalTestnet,
	/// The Crab network genesis builder.
	CrabGenesisBuilder,
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
			crab_config, crab_development_config, crab_genesis_builder_config,
			crab_local_testnet_config,
		};
		Ok(match self {
			ChainSpec::CrabDevelopment => Box::new(crab_development_config()),
			ChainSpec::CrabLocalTestnet => Box::new(crab_local_testnet_config()),
			ChainSpec::CrabGenesisBuilder => Box::new(crab_genesis_builder_config()),
			ChainSpec::Crab => Box::new(crab_config()?),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"crab-dev" | "dev" => Some(ChainSpec::CrabDevelopment),
			"crab-local" => Some(ChainSpec::CrabLocalTestnet),
			"crab-genesis" => Some(ChainSpec::CrabGenesisBuilder),
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
