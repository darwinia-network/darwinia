#![cfg_attr(not(feature = "std"), no_std)]
#![feature(test)]

extern crate rstd;
#[cfg(all(feature = "std", test))]
extern crate test;
mod common;
mod merkle_mountain_range;
mod merkle_proof;

#[allow(unused)]
#[cfg(all(feature = "std", test))]
mod tests;

pub use common::*;
pub use merkle_mountain_range::MerkleMountainRange;
pub use merkle_proof::MerkleProof;
