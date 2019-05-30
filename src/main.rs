//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
mod service;
mod cli;

pub use substrate_cli::{VersionInfo, IntoExit, error};

fn run() -> cli::error::Result<()> {
	let version = VersionInfo {
		name: "Darwinia AppChain",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "darwinia-appchain",
		author: "Evolution Land <hello@evolution.land>",
		description: "Application chain in Darwinia network based on substrate framework",
		support_url: "https://github.com/evolutionlandorg/darwinia-appchain/issues/new",
	};
	cli::run(::std::env::args(), cli::Exit, version)
}

error_chain::quick_main!(run);
