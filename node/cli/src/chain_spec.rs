
use service;

/// The chain specification option.
#[derive(Clone, Debug)]
pub enum Alternative {
	/// Whatever the current runtime is, with just Alice as an auth.
	Development,
	/// Whatever the current runtime is, with simple Alice/Bob auths.
	LocalTestnet,
}

impl Default for Alternative {
	fn default() -> Self {
		Alternative::Development
	}
}

/// Get a chain config from a spec setting.
impl Alternative {
	pub(crate) fn load(self) -> Result<service::chain_spec::ChainSpec, String> {
		Ok(match self {
			Alternative::Development => service::chain_spec::development_config(),
			Alternative::LocalTestnet => service::chain_spec::local_testnet_config(),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(Alternative::Development),
			"local" => Some(Alternative::LocalTestnet),
			"" => Some(Alternative::default()),
			_ => None,
		}
	}
}