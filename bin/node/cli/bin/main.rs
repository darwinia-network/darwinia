//! Darwinia Node CLI

#![warn(missing_docs)]

fn main() -> sc_cli::Result<()> {
	let version = sc_cli::VersionInfo {
		name: "Darwinia Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "darwinia",
		author: "Darwinia Network <hello@darwinia.network>",
		description: "Darwinia node implementation in Rust",
		support_url: "https://github.com/darwinia-network/darwinia/issues/new",
		copyright_start_year: 2018,
	};

	node_cli::run(std::env::args(), version)
}
