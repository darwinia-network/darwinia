//! Darwinia CLI

#![warn(missing_docs)]

use cli::VersionInfo;

fn main() -> Result<(), cli::Error> {
	let version = VersionInfo {
		name: "Darwinia-Network Darwinia",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "darwinia",
		author: "Darwinia Network <hello@darwinia.network>",
		description: "Darwinia Relay-chain Client Node",
		support_url: "https://github.com/darwinia-network/darwinia//issues/new",
		copyright_start_year: 2018,
	};

	cli::run(version)
}
