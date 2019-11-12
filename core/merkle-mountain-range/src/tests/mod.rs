pub mod support;

use std::time::Instant;

use blake2::{Blake2b, Digest};
use test::Bencher;

use crate::*;
// pub use support::{Digest, *};

type Hasher = Blake2b;
// type Hasher = DebugHasher;

fn mmr_with_count(count: usize) -> MerkleMountainRange<Hasher> {
	let mut mmr = MerkleMountainRange::<Hasher>::new(vec![]);
	for i in 0..count {
		let hash = usize_to_hash(i);
		mmr.append(&hash);
	}

	mmr
}

fn usize_to_hash(x: usize) -> Hash {
	Hasher::digest(&x.to_le_bytes()).to_vec()
}

#[test]
fn t1() {
	let mmr = mmr_with_count(6);
	let a = chain_two_hash::<Hasher, _>(&mmr[0], &mmr[1]);
	let b = chain_two_hash::<Hasher, _>(&a, &mmr[5]);
	let c = chain_two_hash::<Hasher, _>(&mmr[7], &mmr[8]);
	let d = chain_two_hash::<Hasher, _>(&b, &c);
	assert_eq!(mmr.root().unwrap(), d);
}

#[test]
fn t2() {
	let mmr = mmr_with_count(6);
	let root = mmr.root().unwrap();
	let index = 0;
	let hash = usize_to_hash(index);
	let proof = mmr.to_merkle_proof(index).unwrap();
	assert!(proof.verify::<Hasher, _>(root, hash, index));
}

#[bench]
fn b1(b: &mut Bencher) {
	let mmr = mmr_with_count(10_000_000);
	let index = 23_333;
	let mmr_index = leaf_index(index);
	let root = mmr.root().unwrap();
	let hash = usize_to_hash(index);
	let proof = mmr.to_merkle_proof(mmr_index).unwrap();

	b.iter(|| assert!(proof.verify::<Hasher, _>(root.clone(), hash.clone(), mmr_index)));
}

#[test]
fn b2() {
	let mmr = mmr_with_count(100_000_000);
	let index = 233_333;
	let mmr_index = leaf_index(index);
	let root = mmr.root().unwrap();
	let hash = usize_to_hash(index);

	let start = Instant::now();
	let proof = mmr.to_merkle_proof(mmr_index).unwrap();
	proof.verify::<Hasher, _>(root, hash, mmr_index);
	let elapsed = start.elapsed();
	println!("{}", elapsed.as_nanos());
}
