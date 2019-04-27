//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
mod service;
mod cli;

pub use substrate_cli::{VersionInfo, IntoExit, error};

fn run() -> cli::error::Result<()> {
	let version = VersionInfo {
		name: "Evolution Land's Land Chain Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "land-chain",
		author: "Evolution Land <hello@evolution.land>",
		description: "Land Chain Node",
		support_url: "https://github.com/evolutionlandorg/land-chain/issues/new",
	};
	cli::run(::std::env::args(), cli::Exit, version)
}

error_chain::quick_main!(run);
