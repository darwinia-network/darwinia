// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use rstd::rc::Rc;

mod db;
mod error;
mod nibbles;
mod node;
mod proof;
mod tests;
pub mod trie;

pub use db::MemoryDB;
pub use error::TrieError;
pub use proof::Proof;
pub use trie::{MerklePatriciaTrie, Trie, TrieResult};

/// Generates a trie for a vector of key-value tuples
///
/// ```rust
/// extern crate merkle_patricia_trie as trie;
/// extern crate hex;
///
/// use trie::{Trie, build_trie};
/// use hex::FromHex;
///
/// fn main() {
/// 	let v = vec![
/// 		("doe", "reindeer"),
/// 		("dog", "puppy"),
/// 		("dogglesworth", "cat"),
/// 	];
///
/// 	let root:Vec<u8> = Vec::from_hex("8aad789dff2f538bca5d8ea56e8abe10f4c7ba3a5dea95fea4cd6e7c3a1168d3").unwrap();
/// 	assert_eq!(build_trie(v).unwrap().root().unwrap(), root);
/// }
/// ```
pub fn build_trie<I, A, B>(data: I) -> TrieResult<MerklePatriciaTrie>
where
	I: IntoIterator<Item = (A, B)>,
	A: AsRef<[u8]> + Ord,
	B: AsRef<[u8]>,
{
	let memdb = Rc::new(MemoryDB::new());
	let mut trie = MerklePatriciaTrie::new(memdb.clone());
	data.into_iter().for_each(|(key, value)| {
		// TODO  the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `core::ops::Try`)
		trie.insert(key.as_ref().to_vec(), value.as_ref().to_vec());
	});
	trie.root()?;
	Ok(trie)
}

/// Generates a trie for a vector of values
///
/// ```rust
/// extern crate merkle_patricia_trie as trie;
/// extern crate hex;
///
/// use trie::{Trie, build_order_trie};
/// use hex::FromHex;
///
/// fn main() {
/// 	let v = &["doe", "reindeer"];
/// 	let root:Vec<u8> = Vec::from_hex("e766d5d51b89dc39d981b41bda63248d7abce4f0225eefd023792a540bcffee3").unwrap();
/// 	assert_eq!(build_order_trie(v).unwrap().root().unwrap(), root);
/// }
/// ```
pub fn build_order_trie<I>(data: I) -> TrieResult<MerklePatriciaTrie>
where
	I: IntoIterator,
	I::Item: AsRef<[u8]>,
{
	build_trie(data.into_iter().enumerate().map(|(i, v)| (rlp::encode(&i), v)))
}
