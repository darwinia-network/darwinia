use core::{marker::PhantomData, ops::Index};

use blake2::Digest;
use codec::{Decode, Encode};

use rstd::{borrow::ToOwned, vec::Vec};

use crate::*;

#[derive(Clone, Debug, Default, Encode, Decode)]
pub struct MerkleMountainRange<D> {
	hashes: Vec<Hash>,
	_hasher: PhantomData<D>,
}

impl<D: Digest> MerkleMountainRange<D> {
	pub fn new(hashes: Vec<Hash>) -> Self {
		Self {
			hashes,
			_hasher: PhantomData,
		}
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.hashes.len()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.hashes.is_empty()
	}

	#[inline]
	pub fn get(&self, index: usize) -> Option<&Hash> {
		self.hashes.get(index)
	}

	#[inline]
	pub fn push(&mut self, hash: Hash) -> usize {
		self.hashes.push(hash);
		self.len() - 1
	}

	pub fn append<H: AsRef<[u8]>>(&mut self, hash: H) -> Option<usize> {
		let hash = hash.as_ref();

		if self.is_empty() {
			return Some(self.push(hash.to_owned()));
		}

		let mut index = self.len();
		let (peak_map, height) = peak_map_height(index);

		if height != 0 {
			return None;
		}

		self.push(hash.to_owned());

		let mut peak = 1;
		while (peak_map & peak) != 0 {
			let new_hash = chain_two_hash::<D, _>(&self[index + 1 - 2 * peak], &self[self.len() - 1]);
			self.push(new_hash);

			peak *= 2;
			index += 1;
		}

		Some(index)
	}

	pub fn root(&self) -> Option<Hash> {
		if self.is_empty() {
			None
		} else {
			// TODO: bagging strategy
			// Some(
			// 	peak_indexes(self.len())
			// 		.into_iter()
			// 		.fold(D::new(), |hasher, peak_index| {
			// 			hasher.chain(&self[peak_index])
			// 		})
			// 		.result()
			// 		.to_vec(),
			// )

			let mut hash = None;
			for peak_index in peak_indexes(self.len()).into_iter().rev() {
				hash = match hash {
					None => Some(self[peak_index].to_owned()),
					Some(right_peak) => Some(chain_two_hash::<D, _>(&self[peak_index], &right_peak)),
				}
			}

			hash
		}
	}

	pub fn to_merkle_proof(&self, index: usize) -> Option<MerkleProof> {
		if !is_leaf(index) {
			return None;
		}

		let family_branch = family_branch(index, self.len());
		let peak_index = if let Some((current, _)) = family_branch.last() {
			*current
		} else {
			index
		};
		let mut path: Vec<_> = family_branch
			.into_iter()
			.map(|(_, sibling)| self.get(sibling).unwrap().to_owned())
			.collect();
		path.append(&mut self.peak_path(peak_index));

		Some(MerkleProof {
			mmr_size: self.len(),
			path,
		})
	}

	pub fn peak_path(&self, peak_index: usize) -> Vec<Hash> {
		let mut peaks: Vec<_> = peak_indexes(self.len())
			.into_iter()
			.filter(|peak_index_| *peak_index_ < peak_index)
			.map(|peak_index| self[peak_index].to_owned())
			.collect();
		if let Some(peak) = self.bag_the_rhs(peak_index) {
			peaks.push(peak);
		}
		peaks.reverse();

		peaks
	}

	pub fn bag_the_rhs(&self, peak_index: usize) -> Option<Hash> {
		let peak_indexes: Vec<_> = peak_indexes(self.len())
			.into_iter()
			.filter(|peak_index_| *peak_index_ > peak_index)
			.collect();
		let mut hash = None;
		for peak_index in peak_indexes.into_iter().rev() {
			hash = match hash {
				None => Some(self[peak_index].to_owned()),
				Some(right_peak) => Some(chain_two_hash::<D, _>(&self[peak_index], &right_peak)),
			}
		}

		hash
	}
}

impl<D: Digest> Index<usize> for MerkleMountainRange<D> {
	type Output = Hash;

	fn index(&self, index: usize) -> &Self::Output {
		&self.hashes[index]
	}
}
