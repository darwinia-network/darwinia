// crates.io
use clap::{Parser, ValueEnum};
// substrate
use sp_core::{ed25519::Pair as Ep, sr25519::Pair as Sp, Pair as _};

#[derive(Parser)]
#[command(rename_all = "kebab")]
struct Cli {
	#[arg(value_name = "PRIVATE_KEY")]
	from: String,
	#[arg(value_name = "ADDRESS")]
	to: String,
	#[arg(
		required = true,
		value_enum,
		long,
		short,
		value_name = "SCHEME",
		default_value = "sr25519"
	)]
	scheme: Scheme,
	#[arg(required = true, value_enum, long, short, value_name = "NETWORK")]
	network: Network,
}
#[allow(non_camel_case_types)]
#[derive(Clone, ValueEnum)]
enum Scheme {
	sr25519,
	ed25519,
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

enum Pair {
	S(Box<Sp>),
	E(Box<Ep>),
}
impl Pair {
	fn sign(&self, msg: &[u8]) -> [u8; 64] {
		match self {
			Self::S(p) => p.sign(msg).0,
			Self::E(p) => p.sign(msg).0,
		}
	}
}

fn main() {
	let Cli { from, to, scheme, network } = Cli::parse();
	let from = array_bytes::hex2array(from).expect("invalid private key");
	let from = match scheme {
		Scheme::sr25519 => Pair::S(Box::new(Sp::from_seed(&from))),
		Scheme::ed25519 => Pair::E(Box::new(Ep::from_seed(&from))),
	};
	let network = network.as_bytes();
	let msg = [
		b"<Bytes>I authorize the migration to ",
		to.to_lowercase().as_bytes(),
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
