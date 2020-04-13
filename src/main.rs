//! Darwinia CLI

#![warn(missing_docs)]

// --- darwinia ---
use darwinia_cli::VersionInfo;

fn main() -> Result<(), darwinia_cli::Error> {
	let version = VersionInfo {
		name: "Darwinia Network",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "darwinia",
		author: "Darwinia Network <hello@darwinia.network>",
		description: "Darwinia node implementation in Rust",
		support_url: "https://github.com/darwinia-network/darwinia/issues/new",
		copyright_start_year: 2018,
	};

	darwinia_cli::run(version)
}
