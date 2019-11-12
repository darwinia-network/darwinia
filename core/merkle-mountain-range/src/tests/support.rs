pub struct DebugHasher;

pub trait Digest {
	fn new() -> Self;

	fn chain<B: AsRef<[u8]>>(self, data: B) -> Self
	where
		Self: Sized;

	fn result(self) -> Vec<u8>;

	fn digest(data: &[u8]) -> Vec<u8>;
}

impl Specify for DebugHasher {
	fn new() -> Self {
		DebugHasher
	}
}

impl<D: Specify> Digest for D {
	fn new() -> Self {
		<D as Specify>::new()
	}

	fn chain<B: AsRef<[u8]>>(self, data: B) -> Self {
		self
	}

	fn result(self) -> Vec<u8> {
		unimplemented!()
	}

	fn digest(data: &[u8]) -> Vec<u8> {
		unimplemented!()
	}
}

pub trait Specify {
	fn new() -> Self;
}
