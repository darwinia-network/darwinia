//! Darwinia CLI library.

// --- crates ---
use structopt::StructOpt;

#[allow(missing_docs)]
#[derive(Debug, StructOpt)]
pub enum Subcommand {
	#[allow(missing_docs)]
	#[structopt(flatten)]
	Base(sc_cli::Subcommand),
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
}

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
