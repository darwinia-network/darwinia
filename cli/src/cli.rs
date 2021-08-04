// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! Darwinia CLI library.

// --- crates ---
use structopt::StructOpt;
// --- substrate ---
use sc_cli::{KeySubcommand, SignCmd, VanityCmd, VerifyCmd};

#[allow(missing_docs)]
#[derive(Debug, StructOpt)]
pub struct Cli {
	#[allow(missing_docs)]
	#[structopt(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub run: RunCmd,

	/// Load the boot configuration json file from <PATH>. Command line input will be overwritten by this.
	#[structopt(long = "conf", value_name = "PATH")]
	pub conf: Option<std::path::PathBuf>,
}

#[allow(missing_docs)]
#[derive(Debug, StructOpt)]
pub struct RunCmd {
	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub base: sc_cli::RunCmd,

	/// Force using Crab native runtime.
	#[structopt(long = "force-crab")]
	pub force_crab: bool,

	/// Disable the authority discovery module on validator or sentry nodes.
	///
	/// Enabled by default on validator and sentry nodes. Always disabled on non
	/// validator or sentry nodes.
	///
	/// When enabled:
	///
	/// (1) As a validator node: Make oneself discoverable by publishing either
	///     ones own network addresses, or the ones of ones sentry nodes
	///     (configured via the `sentry-nodes` flag).
	///
	/// (2) As a validator or sentry node: Discover addresses of validators or
	///     addresses of their sentry nodes and maintain a permanent connection
	///     to a subset.
	#[structopt(long = "disable-authority-discovery")]
	pub authority_discovery_disabled: bool,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub dynamic_fee_parameters: DynamicFeeParameters,
}

#[allow(missing_docs)]
#[derive(Debug, StructOpt)]
pub enum Subcommand {
	/// Build a chain specification.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Remove the whole chain.
	PurgeChain(sc_cli::PurgeChainCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// Key management cli utilities
	Key(KeySubcommand),

	/// Verify a signature for a message, provided on STDIN, with a given (public or secret) key.
	Verify(VerifyCmd),

	/// Generate a seed that provides a vanity address.
	Vanity(VanityCmd),

	/// Sign a message, with a given (secret) key.
	Sign(SignCmd),

	/// Try some experimental command on the runtime. This includes migration and runtime-upgrade
	/// testing.
	#[cfg(feature = "try-runtime")]
	TryRuntime(try_runtime_cli::TryRuntimeCmd),

	/// The custom benchmark subcommmand benchmarking runtime pallets.
	#[cfg(feature = "runtime-benchmarks")]
	#[structopt(name = "benchmark", about = "Benchmark runtime pallets.")]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),
}

#[derive(Debug, StructOpt)]
pub struct DynamicFeeParameters {
	/// Maximum number of logs in a query.
	#[structopt(long, default_value = "10000")]
	pub max_past_logs: u32,

	/// The dynamic-fee pallet target gas price set by block author
	#[structopt(long, default_value = "1000000000")]
	pub target_gas_price: u64,
}
