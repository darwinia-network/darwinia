use std::{
	env,
	fs::{self, File},
	io::{self, Read},
	path::Path,
};

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

	//	let srml_dir = fs::read_dir(
	//		env::args()
	//			.skip(1)
	//			.next()
	//			.ok_or(io::Error::new(
	//				io::ErrorKind::NotFound,
	//				"please specify the `srml` path",
	//			))?
	//			.as_str(),
	//	)?
	//	.into_iter()
	//	.filter_map(|entry| if let Ok(entry) = entry { Some(entry) } else { None })
	//	.find(|entry| {
	//		let path = entry.path();
	//		println!("{:?}", path);
	//		path.is_dir() && path.file_name().unwrap().to_str().unwrap() == "srml"
	//	})
	//	.ok_or(io::Error::new(io::ErrorKind::NotFound, "folder `srml` not found"))?;

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
		let mut s = String::new();
		file.read_to_string(&mut s)?;

		Ok(Self {
			structs: Storage::parse_structs(s),
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
