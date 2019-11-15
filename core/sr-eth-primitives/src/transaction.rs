pub use codec::{Decode, Encode};
use ethereum_types::{Address, BigEndianHash, H160, H256, U256};
use keccak_hash::keccak;
pub use parity_crypto::publickey::{public_to_address, recover, Public, Secret, Signature};
use rlp::{self, DecoderError, Encodable, Rlp, RlpStream};
use rstd::ops::Deref;
use rstd::prelude::*;
use substrate_primitives::RuntimeDebug;

pub type Bytes = Vec<u8>;

/// Fake address for unsigned transactions as defined by EIP-86.
pub const UNSIGNED_SENDER: Address = H160([0xff; 20]);

/// Replay protection logic for v part of transaction's signature
pub mod signature {
	/// Adds chain id into v
	pub fn add_chain_replay_protection(v: u64, chain_id: Option<u64>) -> u64 {
		v + if let Some(n) = chain_id { 35 + n * 2 } else { 27 }
	}

	/// Returns refined v
	/// 0 if `v` would have been 27 under "Electrum" notation, 1 if 28 or 4 if invalid.
	pub fn check_replay_protection(v: u64) -> u8 {
		match v {
			v if v == 27 => 0,
			v if v == 28 => 1,
			v if v >= 35 => ((v - 1) % 2) as u8,
			_ => 4,
		}
	}
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum Action {
	/// Create creates new contract.
	Create,
	/// Calls contract at given address.
	/// In the case of a transfer, this is the receiver's address.'
	Call(Address),
}

impl Default for Action {
	fn default() -> Action {
		Action::Create
	}
}

impl rlp::Decodable for Action {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		if rlp.is_empty() {
			if rlp.is_data() {
				Ok(Action::Create)
			} else {
				Err(DecoderError::RlpExpectedToBeData)
			}
		} else {
			Ok(Action::Call(rlp.as_val()?))
		}
	}
}

impl rlp::Encodable for Action {
	fn rlp_append(&self, s: &mut RlpStream) {
		match *self {
			Action::Create => s.append_internal(&""),
			Action::Call(ref addr) => s.append_internal(addr),
		};
	}
}

pub struct BestBLock {
	height: u64, // enough for ethereum poa network (kovan)
	hash: H256,
	total_difficulty: U256,
}

#[derive(Default, PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
struct EthHeader<Moment> {
	parent_hash: H256,
	ommers_hash: H256,
	beneficiary: Address,
	state_root: H256,
	transactions_root: H256,
	receipt_root: H256,
	logs_bloom: H256,
	difficulty: u64,
	number: u64,
	gas_limit: u64,
	gas_used: u64,
	timestamp: Moment,
	extra_data: Bytes,
	mix_hash: H256,
	nonce: u64,
}

#[derive(Default, PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct PlainTransaction {
	pub nonce: U256,
	pub gas_price: U256,
	pub gas: U256,
	pub action: Action,
	pub value: U256,
	pub data: Bytes,
}

impl PlainTransaction {
	/// Append object with a without signature into RLP stream
	pub fn rlp_append_unsigned_transaction(&self, s: &mut RlpStream, chain_id: Option<u64>) {
		s.begin_list(if chain_id.is_none() { 6 } else { 9 });
		s.append(&self.nonce);
		s.append(&self.gas_price);
		s.append(&self.gas);
		s.append(&self.action);
		s.append(&self.value);
		s.append(&self.data);
		if let Some(n) = chain_id {
			s.append(&n);
			s.append(&0u8);
			s.append(&0u8);
		}
	}

	/// The message hash of the transaction.
	pub fn hash(&self, chain_id: Option<u64>) -> H256 {
		let mut stream = RlpStream::new();
		self.rlp_append_unsigned_transaction(&mut stream, chain_id);
		keccak(stream.as_raw())
	}

	/// Signs the transaction as coming from `sender`.
	pub fn sign(self, secret: &Secret, chain_id: Option<u64>) -> SignedTransaction {
		let sig = parity_crypto::publickey::sign(secret, &self.hash(chain_id))
			.expect("data is valid and context has signing capabilities; qed");
		SignedTransaction::new(self.with_signature(sig, chain_id)).expect("secret is valid so it's recoverable")
	}

	/// Signs the transaction with signature.
	pub fn with_signature(self, sig: Signature, chain_id: Option<u64>) -> UnverifiedTransaction {
		UnverifiedTransaction {
			unsigned: self,
			r: sig.r().into(),
			s: sig.s().into(),
			v: signature::add_chain_replay_protection(sig.v() as u64, chain_id),
			hash: H256::zero(),
		}
		.compute_hash()
	}

	/// Useful for test incorrectly signed transactions.
	#[cfg(test)]
	pub fn invalid_sign(self) -> UnverifiedTransaction {
		UnverifiedTransaction {
			unsigned: self,
			r: U256::one(),
			s: U256::one(),
			v: 0,
			hash: H256::zero(),
		}
		.compute_hash()
	}

	/// Specify the sender; this won't survive the serialize/deserialize process, but can be cloned.
	pub fn fake_sign(self, from: Address) -> SignedTransaction {
		SignedTransaction {
			transaction: UnverifiedTransaction {
				unsigned: self,
				r: U256::one(),
				s: U256::one(),
				v: 0,
				hash: H256::zero(),
			}
			.compute_hash(),
			sender: from,
			public: None,
		}
	}

	/// Add EIP-86 compatible empty signature.
	pub fn null_sign(self, chain_id: u64) -> SignedTransaction {
		SignedTransaction {
			transaction: UnverifiedTransaction {
				unsigned: self,
				r: U256::zero(),
				s: U256::zero(),
				v: chain_id,
				hash: H256::zero(),
			}
			.compute_hash(),
			sender: UNSIGNED_SENDER,
			public: None,
		}
	}
}

#[derive(Default, PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct UnverifiedTransaction {
	unsigned: PlainTransaction,
	v: u64,
	r: U256,
	s: U256,
	hash: H256,
}

impl Deref for UnverifiedTransaction {
	type Target = PlainTransaction;

	fn deref(&self) -> &Self::Target {
		&self.unsigned
	}
}

impl rlp::Decodable for UnverifiedTransaction {
	fn decode(d: &Rlp) -> Result<Self, DecoderError> {
		if d.item_count()? != 9 {
			return Err(DecoderError::RlpIncorrectListLen);
		}
		let hash = keccak(d.as_raw());
		Ok(UnverifiedTransaction {
			unsigned: PlainTransaction {
				nonce: d.val_at(0)?,
				gas_price: d.val_at(1)?,
				gas: d.val_at(2)?,
				action: d.val_at(3)?,
				value: d.val_at(4)?,
				data: d.val_at(5)?,
			},
			v: d.val_at(6)?,
			r: d.val_at(7)?,
			s: d.val_at(8)?,
			hash: hash,
		})
	}
}

impl rlp::Encodable for UnverifiedTransaction {
	fn rlp_append(&self, s: &mut RlpStream) {
		self.rlp_append_sealed_transaction(s)
	}
}

impl UnverifiedTransaction {
	/// Used to compute hash of created transactions
	fn compute_hash(mut self) -> UnverifiedTransaction {
		let hash = keccak(&*self.rlp_bytes());
		self.hash = hash;
		self
	}

	/// Checks if the signature is empty.
	pub fn is_unsigned(&self) -> bool {
		self.r.is_zero() && self.s.is_zero()
	}

	/// Returns transaction receiver, if any
	pub fn receiver(&self) -> Option<Address> {
		match self.unsigned.action {
			Action::Create => None,
			Action::Call(receiver) => Some(receiver),
		}
	}

	/// Append object with a signature into RLP stream
	fn rlp_append_sealed_transaction(&self, s: &mut RlpStream) {
		s.begin_list(9);
		s.append(&self.nonce);
		s.append(&self.gas_price);
		s.append(&self.gas);
		s.append(&self.action);
		s.append(&self.value);
		s.append(&self.data);
		s.append(&self.v);
		s.append(&self.r);
		s.append(&self.s);
	}

	///	Reference to unsigned part of this transaction.
	pub fn as_unsigned(&self) -> &PlainTransaction {
		&self.unsigned
	}

	/// Returns standardized `v` value (0, 1 or 4 (invalid))
	pub fn standard_v(&self) -> u8 {
		signature::check_replay_protection(self.v)
	}

	/// The `v` value that appears in the RLP.
	pub fn original_v(&self) -> u64 {
		self.v
	}

	/// The chain ID, or `None` if this is a global transaction.
	pub fn chain_id(&self) -> Option<u64> {
		match self.v {
			v if self.is_unsigned() => Some(v),
			v if v >= 35 => Some((v - 35) / 2),
			_ => None,
		}
	}

	/// Construct a signature object from the sig.
	pub fn signature(&self) -> Signature {
		let r: H256 = BigEndianHash::from_uint(&self.r);
		let s: H256 = BigEndianHash::from_uint(&self.s);
		Signature::from_rsv(&r, &s, self.standard_v())
	}

	/// Checks whether the signature has a low 's' value.
	pub fn check_low_s(&self) -> Result<(), parity_crypto::publickey::Error> {
		if !self.signature().is_low_s() {
			Err(parity_crypto::publickey::Error::InvalidSignature.into())
		} else {
			Ok(())
		}
	}

	/// Get the hash of this transaction (keccak of the RLP).
	pub fn hash(&self) -> H256 {
		self.hash
	}

	/// Recovers the public key of the sender.
	pub fn recover_public(&self) -> Result<Public, parity_crypto::publickey::Error> {
		Ok(recover(&self.signature(), &self.unsigned.hash(self.chain_id()))?)
	}

	/// Try to verify transaction and recover sender.
	pub fn verify_unordered(self) -> Result<SignedTransaction, parity_crypto::publickey::Error> {
		SignedTransaction::new(self)
	}
}

#[derive(Default, PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct SignedTransaction {
	transaction: UnverifiedTransaction,
	sender: Address,
	public: Option<Public>,
}

impl rlp::Encodable for SignedTransaction {
	fn rlp_append(&self, s: &mut RlpStream) {
		self.transaction.rlp_append_sealed_transaction(s)
	}
}

impl Deref for SignedTransaction {
	type Target = UnverifiedTransaction;
	fn deref(&self) -> &Self::Target {
		&self.transaction
	}
}

impl From<SignedTransaction> for UnverifiedTransaction {
	fn from(tx: SignedTransaction) -> Self {
		tx.transaction
	}
}

impl SignedTransaction {
	/// Try to verify transaction and recover sender.
	pub fn new(transaction: UnverifiedTransaction) -> Result<Self, parity_crypto::publickey::Error> {
		if transaction.is_unsigned() {
			Ok(SignedTransaction {
				transaction: transaction,
				sender: UNSIGNED_SENDER,
				public: None,
			})
		} else {
			let public = transaction.recover_public()?;
			let sender = public_to_address(&public);
			Ok(SignedTransaction {
				transaction: transaction,
				sender: sender,
				public: Some(public),
			})
		}
	}

	/// Returns transaction sender.
	pub fn sender(&self) -> Address {
		self.sender
	}

	/// Returns a public key of the sender.
	pub fn public_key(&self) -> Option<Public> {
		self.public
	}

	/// Checks is signature is empty.
	pub fn is_unsigned(&self) -> bool {
		self.transaction.is_unsigned()
	}

	/// Deconstructs this transaction back into `UnverifiedTransaction`
	pub fn deconstruct(self) -> (UnverifiedTransaction, Address, Option<Public>) {
		(self.transaction, self.sender, self.public)
	}
}
