//! Darwinia Node CLI

#![warn(missing_docs)]

fn main() -> sc_cli::Result<()> {
	let version = sc_cli::VersionInfo {
		name: "Darwinia Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "darwinia",
		author: "Darwinia Network <hello@darwinia.network>",
		description: r#" _____                      _       _       
|  __ \\                    (_)     (_)      
| |  | | __ _ _ ____      ___ _ __  _  __ _ 
| |  | |/ _` | '__\\ \\ /\\ / / | '_ \\| |/ _` |
| |__| | (_| | |   \\ V  V /| | | | | | (_| |
|_____/ \\__,_|_|    \\_/\\_/ |_|_| |_|_|\\__,_|
            Darwinia Network 2018-2020          "#,
		support_url: "https://github.com/darwinia-network/darwinia/issues/new",
		copyright_start_year: 2018,
	};

	node_cli::run(std::env::args(), version)
}
