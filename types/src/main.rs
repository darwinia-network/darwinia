use std::{
	env,
	fs::{self, File},
	io::{self, Read},
	path::Path,
};

use regex::Regex;

mod native_types {
	const ARRAY: &[&str] = &["Bytes", "H160", "H256", "U128", "U256", "U512"];

	const NUMERIC: &[&str] = &[
		"i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "i128", "u128", "i268", "u256",
	];

	const SUBSTRATE: &[&str] = &["Address", "LockIdentifier"];

	const WRAPPER: &[&str] = &[r"Compact<.+?>", r"Option<.+?>", r"Vec<.+?>", r"[.+?;.+?]"];
}

fn main() -> io::Result<()> {
	let srml_path = env::args()
		.skip(1)
		.next()
		.ok_or(io::Error::new(
			io::ErrorKind::NotFound,
			"please specify the `srml` path",
		))?
		.to_string();
	let srml_path = Path::new(&srml_path);

	if !srml_path.is_dir() {
		return Ok(());
	}

	let modules = srml_path
		.read_dir()?
		.into_iter()
		.filter_map(|entry| {
			if let Ok(entry) = entry {
				if entry.path().is_dir() {
					return Some(Module::new(entry.path().to_str().unwrap()));
				}
			}

			None
		})
		.collect::<Vec<_>>();

	println!("{:?}", modules);

	Ok(())
}

#[derive(Debug, Default)]
struct Module {
	path: String,
	storage: Storage,
}

impl Module {
	fn new<S: AsRef<str>>(path: S) -> Self {
		Module {
			path: path.as_ref().to_string(),
			..Default::default()
		}
	}

	fn read(&mut self) -> io::Result<()> {
		self.storage = Path::new(&self.path)
			.read_dir()?
			.into_iter()
			.filter_map(|entry| {
				if let Ok(entry) = entry {
					if let Some(path) = entry.path().to_str() {
						if path.ends_with(".rs") {
							return Storage::from_file(path).ok();
						}
					}
				}

				None
			})
			.fold(Storage::default(), |acc, storage| acc.merge(storage));

		Ok(())
	}
}

#[derive(Debug, Default)]
struct Storage {
	structs: Vec<String>,
}

impl Storage {
	fn from_file<S: AsRef<Path>>(path: S) -> io::Result<Self> {
		let mut file = File::open(path)?;
		let mut text = String::new();
		file.read_to_string(&mut text)?;

		Ok(Self {
			structs: Storage::parse_structs(text),
		})
	}

	fn merge(mut self, mut storage: Self) -> Self {
		self.structs.append(&mut storage.structs);
		self
	}

	fn parse_structs(text: String) -> Vec<String> {
		let mut structs = vec![];

		structs
	}
}
