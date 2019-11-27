#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
pub extern crate rlp_derive;

use rstd::vec;
use rstd::vec::Vec;

pub mod encoded;
pub mod error;
//pub mod keccak;
pub mod pow;
pub mod receipt;
//pub mod transaction;

pub use codec::{Decode, Encode};
pub use ethbloom::{Bloom, Input as BloomInput};
pub use ethereum_types::BigEndianHash;
pub use fixed_hash::construct_fixed_hash;
pub use impl_codec::impl_fixed_hash_codec;
pub use impl_rlp::impl_fixed_hash_rlp;
pub use primitive_types::{H160, H256, U128, U256, U512};

pub type Bytes = Vec<u8>;
pub type Address = H160;
pub type BlockNumber = u64;

#[derive(Default, Clone, Copy, Eq, PartialEq, Encode, Decode)]
pub struct BestBlock {
	height: u64, // enough for ethereum poa network (kovan)
	hash: H256,
	total_difficulty: U256,
}

construct_fixed_hash! {pub struct H64(8);}
impl_fixed_hash_rlp!(H64, 8);
impl_fixed_hash_codec!(H64, 8);
