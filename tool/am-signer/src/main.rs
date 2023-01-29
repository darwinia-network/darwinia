// crates.io
use clap::{Parser, ValueEnum};
// substrate
use sp_core::{sr25519::Pair, Pair as _};

#[derive(Parser)]
#[command(rename_all = "kebab")]
struct Cli {
	#[arg(value_name = "PRIVATE_KEY")]
	from: String,
	#[arg(value_name = "ADDRESS")]
	to: String,
	#[arg(required = true, value_enum, long, short, value_name = "NETWORK")]
	network: Network,
}
#[derive(Clone, ValueEnum)]
enum Network {
	Darwinia,
	Crab,
	Pangoro,
	Pangolin,
}
impl Network {
	fn as_bytes(&self) -> &'static [u8] {
		match self {
			Self::Darwinia => b"Darwinia2",
			Self::Crab => b"Crab2",
			Self::Pangoro => b"Pangoro2",
			Self::Pangolin => b"Pangolin2",
		}
	}
}

fn main() {
	let Cli { from, to, network } = Cli::parse();
	let from = array_bytes::hex2array(from).expect("invalid private key");
	let from = Pair::from_seed(&from);
	let network = network.as_bytes();
	let msg = [
		b"<Bytes>I authorize the migration to ",
		to.as_bytes(),
		b", an unused address on ",
		network,
		b". Sign this message to authorize using the Substrate key associated with the account on ",
		&network[..network.len() - 1],
		b" that you wish to migrate.</Bytes>",
	]
	.concat();
	let sig = from.sign(&msg);

	println!("{}", array_bytes::bytes2hex("0x", sig));
}
