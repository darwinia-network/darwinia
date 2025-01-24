// std
use std::{
	fs::File,
	io::{Read, Write},
	path::PathBuf,
};
// crates.io
use array_bytes::{Dehexify, Hexify};
use clap::{Parser, ValueEnum};
use serde::Serialize;
// polkadot-sdk
use sp_core::{ed25519::Pair as Ep, sr25519::Pair as Sp, Pair as _};

#[derive(Parser)]
#[command(rename_all = "kebab")]
struct Cli {
	/// 1.0 networks private key.
	#[arg(long, value_name = "PRIVATE_KEY", conflicts_with = "file", requires = "to")]
	from: Option<String>,
	/// 2.0 networks public key.
	#[arg(long, value_name = "ADDRESS", conflicts_with = "file", requires = "from")]
	to: Option<String>,
	/// The path to the migration list file.
	///
	/// The format is:
	/// ```
	/// form1:to1
	/// from2:to2
	/// ```
	#[arg(long, value_name = "PATH", conflicts_with_all = &["from", "to"])]
	file: Option<PathBuf>,
	#[arg(value_enum, long, short, value_name = "SCHEME", default_value = "sr25519")]
	/// Key scheme.
	scheme: Scheme,
	/// Network name.
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
}
impl Network {
	fn as_bytes(&self) -> &'static [u8] {
		match self {
			Self::Darwinia => b"Darwinia2",
			Self::Crab => b"Crab2",
		}
	}
}

enum Pair {
	S(Box<Sp>),
	E(Box<Ep>),
}
impl Pair {
	fn public_key(&self) -> [u8; 32] {
		match self {
			Self::S(p) => p.public().0,
			Self::E(p) => p.public().0,
		}
	}

	fn sign(&self, msg: &[u8]) -> [u8; 64] {
		match self {
			Self::S(p) => p.sign(msg).0,
			Self::E(p) => p.sign(msg).0,
		}
	}
}

#[derive(Serialize)]
struct MigrationParameters {
	from: String,
	to: String,
	signature: String,
}

fn main() {
	let Cli { from, to, file, scheme, network } = Cli::parse();

	if let Some(from) = from {
		if let Some(to) = to {
			let message = message_of(&to, &network);
			let (from, signature) = sign_message(&from, &scheme, &message);

			println!("{from},{signature}");
		}
	} else if let Some(p) = file {
		let mut f = File::open(&p).expect("file not found");
		let mut s = String::new();

		f.read_to_string(&mut s).expect("file read error");

		let output = s
			.lines()
			.filter_map(|l| {
				let l = l.trim();

				if l.is_empty() {
					None
				} else {
					let (from, to) = l.split_once(':').expect("valid format");
					let message = message_of(to, &network);
					let (from, signature) = sign_message(from, &scheme, &message);

					Some(MigrationParameters { from, to: to.into(), signature })
				}
			})
			.collect::<Vec<_>>();

		let mut f_new = File::create(p.with_file_name("am-signer").with_extension("json"))
			.expect("file create error");

		f_new
			.write_all(&serde_json::to_vec_pretty(&output).expect("json serialize error"))
			.expect("file write error");
		f_new.flush().expect("file flush error");
	}
}

fn message_of(to: &str, network: &Network) -> Vec<u8> {
	let network = network.as_bytes();

	[
		b"<Bytes>I authorize the migration to ",
		to.to_lowercase().as_bytes(),
		b", an unused address on ",
		network,
		b". Sign this message to authorize using the Substrate key associated with the account on ",
		&network[..network.len() - 1],
		b" that you wish to migrate.</Bytes>",
	]
	.concat()
}

fn sign_message(from: &str, scheme: &Scheme, message: &[u8]) -> (String, String) {
	let from = <[u8; 32]>::dehexify(from).expect("invalid private key");
	let from = match scheme {
		Scheme::sr25519 => Pair::S(Box::new(Sp::from_seed(&from))),
		Scheme::ed25519 => Pair::E(Box::new(Ep::from_seed(&from))),
	};
	let signature = from.sign(message);

	(from.public_key().hexify_prefixed(), signature.hexify_prefixed())
}
