// std
use std::{fs, path::PathBuf};
// crates.io
use clap::Parser;
use regex::Regex;

fn main() {
	let Cli { paths } = Cli::parse();

	paths.into_iter().for_each(|p| {
		println!("Checking: {}", p.display());

		if !p.is_dir() {
			panic!("invalid path");
		}

		let (on_chain_ref_time, on_chain_proof_size) = (375_000_000_000, 3_932_160);
		let (mut ref_time, mut proof_size) = (0, 0);

		fs::read_dir(p).unwrap().for_each(|e| {
			let e = e.unwrap();
			let p = e.path();
			let t = fs::read_to_string(p).unwrap();
			let (ref_time_, proof_size_) = R::new().captures_max(&t);

			ref_time = ref_time.max(ref_time_);
			proof_size = proof_size.max(proof_size_);
		});

		println!("Max `ref_time`         : {ref_time}");
		println!("Max on-chain `ref_time`: {on_chain_ref_time}");
		println!("Max `proof_size`         : {proof_size}");
		println!("Max on-chain `proof_size`: {on_chain_proof_size}");

		if ref_time >= on_chain_ref_time || proof_size >= on_chain_proof_size {
			panic!("exceeded on-chain limit");
		}
	});
}

#[derive(Parser)]
#[command(rename_all = "kebab")]
struct Cli {
	/// Path(s) to the weights folder.
	#[arg(value_name = "PATH", num_args = 1.., value_delimiter = ',')]
	paths: Vec<PathBuf>,
}

struct R(Regex);
impl R {
	fn new() -> Self {
		R(Regex::new(r"from_parts\((\d+(?:_\d+)*),\s*(\d+(?:_\d+)*)\)").unwrap())
	}

	fn captures_max(&self, text: &str) -> (u128, u128) {
		let (mut ref_time, mut proof_size) = (0, 0);

		self.0.captures_iter(text).for_each(|c| {
			ref_time = ref_time
				.max(c[1].chars().filter(|c| c != &'_').collect::<String>().parse().unwrap());
			proof_size = proof_size
				.max(c[2].chars().filter(|c| c != &'_').collect::<String>().parse().unwrap());
		});

		(ref_time, proof_size)
	}
}

#[test]
fn regex_should_work() {
	assert_eq!(R::new().captures_max(include_str!("t.rs")), (36_514_055_000, 0));
}
