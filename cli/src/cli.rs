//! Polkadot CLI library.

// --- third-party ---
use structopt::StructOpt;

#[allow(missing_docs)]
#[derive(Debug, StructOpt, Clone)]
pub enum Subcommand {
	#[allow(missing_docs)]
	#[structopt(flatten)]
	Base(sc_cli::Subcommand),
	// TODO: benchmark
	// /// The custom benchmark subcommmand benchmarking runtime pallets.
	// #[structopt(name = "benchmark", about = "Benchmark runtime pallets.")]
	// Benchmark(frame_benchmarking_cli::BenchmarkCmd),
}

#[allow(missing_docs)]
#[derive(Debug, StructOpt, Clone)]
pub struct RunCmd {
	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub base: sc_cli::RunCmd,
}

#[allow(missing_docs)]
#[derive(Debug, StructOpt, Clone)]
#[structopt(settings = &[
	structopt::clap::AppSettings::GlobalVersion,
	structopt::clap::AppSettings::ArgsNegateSubcommands,
	structopt::clap::AppSettings::SubcommandsNegateReqs,
])]
pub struct Cli {
	#[allow(missing_docs)]
	#[structopt(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub run: RunCmd,
}
