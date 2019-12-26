#![cfg_attr(not(feature = "std"), no_std)]
#![feature(test)]

#[cfg(all(feature = "std", test))]
extern crate test;

mod common;
mod merkle_proof;
mod mmr;

#[allow(unused)]
#[cfg(all(feature = "std", test))]
mod tests;

pub use common::*;
pub use merkle_proof::MerkleProof;
pub use mmr::MerkleMountainRange;
