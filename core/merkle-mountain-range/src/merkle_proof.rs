use blake2::Digest;
use rstd::vec::Vec;

use crate::*;

#[derive(Clone, Debug)]
pub struct MerkleProof {
	pub mmr_size: usize,
	//
	// λ cargo bench b1
	//     Finished bench [optimized] target(s) in 0.00s
	//      Running target/release/deps/mmr-0c4d672df8c18022
	//
	// running 1 test
	// test tests::b1 ... bench:      42,015 ns/iter (+/- 23)
	//
	// test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 2 filtered out
	pub path: Vec<Hash>,
	//
	// λ cargo bench b1
	//     Finished bench [optimized] target(s) in 0.00s
	//      Running target/release/deps/mmr-0c4d672df8c18022
	//
	// running 1 test
	// test tests::b1 ... bench:      42,299 ns/iter (+/- 37)
	//
	// test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 2 filtered out
	// pub path: VecDeque<Hash>,
}

impl MerkleProof {
	pub fn verify<D, H>(&self, root: H, hash: H, index: usize) -> bool
	where
		D: Digest,
		H: AsRef<[u8]>,
	{
		self.clone().verify_consume::<D, _>(root, hash, index)
	}

	fn verify_consume<D, H>(&mut self, root: H, hash: H, index: usize) -> bool
	where
		D: Digest,
		H: AsRef<[u8]>,
	{
		let root = root.as_ref();
		let hash = hash.as_ref();
		let peak_indexes = peak_indexes(self.mmr_size);

		if self.path.is_empty() {
			return root == hash;
		}

		let sibling = self.path.remove(0);
		// let sibling = self.path.pop_front().unwrap();
		let sibling = sibling.as_ref();
		let (parent_index, sibling_index) = family(index);

		match peak_indexes.binary_search(&index) {
			Ok(x) => {
				let parent = if x == peak_indexes.len() - 1 {
					chain_two_hash::<D, _>(sibling, hash)
				} else {
					chain_two_hash::<D, _>(hash, sibling)
				};
				self.verify::<D, _>(root, &parent, parent_index)
			}
			_ if parent_index > self.mmr_size => {
				self.verify::<D, _>(root, &chain_two_hash::<D, _>(sibling, hash), parent_index)
			}
			_ => {
				let parent = if is_left_sibling(sibling_index) {
					chain_two_hash::<D, _>(sibling, hash)
				} else {
					chain_two_hash::<D, _>(hash, sibling)
				};
				self.verify::<D, _>(root, &parent, parent_index)
			}
		}
	}
}
