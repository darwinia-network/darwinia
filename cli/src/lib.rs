//! Darwinia CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

// pub mod chain_spec;

#[cfg(feature = "browser")]
mod browser;
mod chain_spec;
#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
mod command;

#[cfg(feature = "cli")]
pub use command::run;
#[cfg(feature = "cli")]
pub use sc_cli::{Error, VersionInfo};
