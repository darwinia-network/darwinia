#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

use rlp::DecoderError;
use sp_std::{borrow::ToOwned, fmt};

#[derive(Debug)]
pub enum TrieError {
	DB(String),
	Decoder(DecoderError),
	InvalidData,
	InvalidStateRoot,
	InvalidProof,
}

impl fmt::Display for TrieError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let printable = match *self {
			TrieError::DB(ref err) => format!("trie error: {:?}", err),
			TrieError::Decoder(ref err) => format!("trie error: {:?}", err),
			TrieError::InvalidData => "trie error: invalid data".to_owned(),
			TrieError::InvalidStateRoot => "trie error: invalid state root".to_owned(),
			TrieError::InvalidProof => "trie error: invalid proof".to_owned(),
		};
		write!(f, "{}", printable)
	}
}

impl From<DecoderError> for TrieError {
	fn from(error: DecoderError) -> Self {
		TrieError::Decoder(error)
	}
}
