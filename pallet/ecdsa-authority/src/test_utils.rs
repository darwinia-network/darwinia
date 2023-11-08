// core
use core::iter;
// crates.io
use libsecp256k1::{Message, PublicKey, SecretKey};
// darwinia
use crate::*;
// substrate
use sp_io::hashing;

impl<T> Pallet<T>
where
	T: Config,
{
	/// Apply the authority changes immediately.
	///
	/// This function is only used for testing.
	pub(crate) fn presume_authority_change_succeed() {
		Self::apply_next_authorities();
	}
}

pub(crate) fn gen_pair(byte: u8) -> (SecretKey, AccountId) {
	let seed = iter::repeat(byte).take(32).collect::<Vec<_>>();
	let secret_key = SecretKey::parse_slice(&seed).unwrap();
	let public_key = PublicKey::from_secret_key(&secret_key).serialize();
	let address = array_bytes::slice_n_into_unchecked::<20, _, _>(
		&hashing::keccak_256(&public_key[1..65])[12..],
	);

	(secret_key, address)
}

pub(crate) fn sign(secret_key: &SecretKey, message: &[u8; 32]) -> Signature {
	let (sig, recovery_id) = libsecp256k1::sign(&Message::parse(message), secret_key);
	let mut signature = [0u8; 65];

	signature[0..64].copy_from_slice(&sig.serialize()[..]);
	signature[64] = recovery_id.serialize();

	Signature(signature)
}
