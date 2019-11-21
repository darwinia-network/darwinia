#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate impl_codec;
#[macro_use]
extern crate fixed_hash;
#[macro_use]
extern crate impl_rlp;

#[macro_use]
extern crate rlp_derive;

pub mod encoded;
pub mod error;
pub mod keccak;
pub mod pow;
pub mod receipt;
pub mod transaction;

pub use codec::{Decode, Encode};
pub use ethereum_types::{Address, BigEndianHash, Bloom, BloomInput, H160, H256, H64, U128, U256, U512};
pub use keccak_hash::keccak;
pub use rlp::{self, DecoderError, Encodable, Rlp, RlpStream};

pub type Bytes = Vec<u8>;
pub type BlockNumber = u64;
pub struct BestBLock {
	height: u64, // enough for ethereum poa network (kovan)
	hash: H256,
	total_difficulty: U256,
}

// TODO: later rewrite Bloom to impl Encode and Decode
//pub use ethbloom::{BloomRef, Input};
//
//// 3 according to yellowpaper
//const BLOOM_BITS: u32 = 3;
//const BLOOM_SIZE: usize = 256;
//
//construct_fixed_hash! {pub struct Bloom(BLOOM_SIZE);}
//impl_fixed_hash_rlp!(Bloom, BLOOM_SIZE);
//impl_fixed_hash_codec!(Bloom, BLOOM_SIZE);
//
//impl<'a> From<Input<'a>> for Bloom {
//	fn from(input: Input<'a>) -> Bloom {
//		let mut bloom = Bloom::default();
//		bloom.accrue(input);
//		bloom
//	}
//}
//
//impl Bloom {
//	pub fn is_empty(&self) -> bool {
//		self.0.iter().all(|x| *x == 0)
//	}
//
//	pub fn contains_input(&self, input: Input<'_>) -> bool {
//		let bloom: Bloom = input.into();
//		self.contains_bloom(&bloom)
//	}
//
//	pub fn contains_bloom<'a, B>(&self, bloom: B) -> bool
//	where
//		BloomRef<'a>: From<B>,
//	{
//		let bloom_ref: BloomRef = bloom.into();
//		// workaround for https://github.com/rust-lang/rust/issues/43644
//		self.contains_bloom_ref(bloom_ref)
//	}
//
//	fn contains_bloom_ref(&self, bloom: BloomRef) -> bool {
//		let self_ref: BloomRef = self.into();
//		self_ref.contains_bloom(bloom)
//	}
//
//	pub fn accrue(&mut self, input: Input<'_>) {
//		let p = BLOOM_BITS;
//
//		let m = self.0.len();
//		let bloom_bits = m * 8;
//		let mask = bloom_bits - 1;
//		let bloom_bytes = (log2(bloom_bits) + 7) / 8;
//
//		let hash: Hash = input.into();
//
//		// must be a power of 2
//		assert_eq!(m & (m - 1), 0);
//		// out of range
//		assert!(p * bloom_bytes <= hash.len() as u32);
//
//		let mut ptr = 0;
//
//		assert_eq!(BLOOM_BITS, 3);
//		unroll! {
//			for i in 0..3 {
//				let _ = i;
//				let mut index = 0 as usize;
//				for _ in 0..bloom_bytes {
//					index = (index << 8) | hash[ptr] as usize;
//					ptr += 1;
//				}
//				index &= mask;
//				self.0[m - 1 - index / 8] |= 1 << (index % 8);
//			}
//		}
//	}
//
//	pub fn accrue_bloom<'a, B>(&mut self, bloom: B)
//	where
//		BloomRef<'a>: From<B>,
//	{
//		let bloom_ref: BloomRef = bloom.into();
//		assert_eq!(self.0.len(), BLOOM_SIZE);
//		assert_eq!(bloom_ref.0.len(), BLOOM_SIZE);
//		for i in 0..BLOOM_SIZE {
//			self.0[i] |= bloom_ref.0[i];
//		}
//	}
//
//	pub fn data(&self) -> &[u8; BLOOM_SIZE] {
//		&self.0
//	}
//}
