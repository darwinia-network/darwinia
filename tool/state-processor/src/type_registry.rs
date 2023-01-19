// std
use std::iter::once;
// crates.io
use enumflags2::{bitflags, BitFlags};
use parity_scale_codec::{Decode, Encode, EncodeLike, Error, Input};

#[derive(Default, Debug, PartialEq, Eq, Encode, Decode)]
pub struct AccountInfo {
	pub nonce: u32,
	pub consumers: u32,
	pub providers: u32,
	pub sufficients: u32,
	pub data: AccountData,
}
#[derive(Default, Debug, PartialEq, Eq, Encode, Decode)]
pub struct AccountData {
	pub free: u128,
	pub reserved: u128,
	pub free_kton_or_misc_frozen: u128,
	pub reserved_kton_or_fee_frozen: u128,
}

#[derive(Default, Debug, Encode, Decode)]
pub struct BalanceLock {
	pub id: [u8; 8],
	pub amount: u128,
	pub reasons: Reasons,
}

#[allow(clippy::unnecessary_cast)]
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
pub enum Reasons {
	Fee = 0,
	Misc = 1,
	All = 2,
}
impl Default for Reasons {
	fn default() -> Self {
		Self::All
	}
}

// https://github.dev/paritytech/substrate/blob/polkadot-v0.9.30/frame/assets/src/types.rs#L33
#[derive(Default, Debug, Encode, Decode)]
pub struct AssetDetails {
	pub owner: [u8; 20],
	pub issuer: [u8; 20],
	pub admin: [u8; 20],
	pub freezer: [u8; 20],
	pub supply: u128,
	pub deposit: u128,
	pub min_balance: u128,
	pub is_sufficient: bool,
	pub accounts: u32,
	pub sufficients: u32,
	pub approvals: u32,
	pub is_frozen: bool,
}

// https://github.dev/paritytech/substrate/blob/polkadot-v0.9.30/frame/assets/src/types.rs#L115
#[derive(Default, Debug, Encode, Decode)]
pub struct AssetAccount {
	pub balance: u128,
	pub is_frozen: bool,
	pub reason: ExistenceReason,
	pub extra: (),
}

// https://github.dev/paritytech/substrate/blob/polkadot-v0.9.30/frame/assets/src/types.rs#L88
#[derive(Debug, Encode, Decode)]
pub enum ExistenceReason {
	#[codec(index = 0)]
	Consumer,
	#[codec(index = 1)]
	Sufficient,
	#[codec(index = 2)]
	DepositHeld(u128),
	#[codec(index = 3)]
	DepositRefunded,
}
impl Default for ExistenceReason {
	fn default() -> Self {
		ExistenceReason::Sufficient
	}
}

// https://github.dev/paritytech/substrate/blob/polkadot-v0.9.30/frame/assets/src/types.rs#L73
#[derive(Debug, Encode, Decode)]
pub struct Approval {
	pub amount: u128,
	pub deposit: u128,
}

// https://github.dev/paritytech/substrate/blob/polkadot-v0.9.30/frame/assets/src/types.rs#L127
#[derive(Clone, Default, Encode, Decode)]
pub struct AssetMetadata {
	pub deposit: u128,
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub is_frozen: bool,
}

#[derive(Default, Debug, Encode, Decode)]
pub struct VestingInfo {
	pub locked: u128,
	pub per_block: u128,
	pub starting_block: u32,
}

#[derive(Default, Debug, Encode, Decode)]
pub struct Deposit {
	pub id: u16,
	pub value: u128,
	pub start_time: u128,
	pub expired_time: u128,
	pub in_use: bool,
}

#[derive(Default, Debug, Encode, Decode)]
pub struct StakingLedger {
	pub stash: [u8; 32],
	#[codec(compact)]
	pub active: u128,
	#[codec(compact)]
	pub active_deposit_ring: u128,
	#[codec(compact)]
	pub active_kton: u128,
	pub deposit_items: Vec<TimeDepositItem>,
	pub ring_staking_lock: StakingLock,
	pub kton_staking_lock: StakingLock,
	pub claimed_rewards: Vec<u32>,
}
#[derive(Default, Debug, Encode, Decode)]
pub struct TimeDepositItem {
	#[codec(compact)]
	pub value: u128,
	#[codec(compact)]
	pub start_time: u64,
	#[codec(compact)]
	pub expire_time: u64,
}
#[derive(Default, Debug, Encode, Decode)]
pub struct StakingLock {
	pub staking_amount: u128,
	pub unbondings: Vec<Unbonding>,
}
#[derive(Default, Debug, Encode, Decode)]
pub struct Unbonding {
	pub amount: u128,
	pub until: u32,
}

#[derive(Default, Debug, Encode, Decode)]
pub struct Ledger {
	pub staked_ring: u128,
	pub staked_kton: u128,
	pub staked_deposits: Vec<u16>,
	pub unstaking_ring: Vec<(u128, u32)>,
	pub unstaking_kton: Vec<(u128, u32)>,
	pub unstaking_deposits: Vec<(u16, u32)>,
}

#[derive(Default, Debug, Encode)]
pub struct Registration {
	pub judgements: Vec<(u32, Judgement)>,
	pub deposit: u128,
	pub info: IdentityInfo,
}
impl Decode for Registration {
	fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
		let (judgements, deposit, info) = Decode::decode(&mut AppendZerosInput::new(input))?;
		Ok(Self { judgements, deposit, info })
	}
}
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
pub enum Judgement {
	Unknown,
	FeePaid(u128),
	Reasonable,
	KnownGood,
	OutOfDate,
	LowQuality,
	Erroneous,
}
#[derive(Default, Debug, Encode, Decode)]
pub struct IdentityInfo {
	pub additional: Vec<(Data, Data)>,
	pub display: Data,
	pub legal: Data,
	pub web: Data,
	pub riot: Data,
	pub email: Data,
	pub pgp_fingerprint: Option<[u8; 20]>,
	pub image: Data,
	pub twitter: Data,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Data {
	None,
	Raw(Vec<u8>),
	BlakeTwo256([u8; 32]),
	Sha256([u8; 32]),
	Keccak256([u8; 32]),
	ShaThree256([u8; 32]),
}
impl Default for Data {
	fn default() -> Self {
		Data::None
	}
}
impl Encode for Data {
	fn encode(&self) -> Vec<u8> {
		match self {
			Data::None => vec![0u8; 1],
			Data::Raw(ref x) => {
				let l = x.len().min(32);
				let mut r = vec![l as u8 + 1; l + 1];
				r[1..].copy_from_slice(&x[..l as usize]);
				r
			},
			Data::BlakeTwo256(ref h) => once(34u8).chain(h.iter().cloned()).collect(),
			Data::Sha256(ref h) => once(35u8).chain(h.iter().cloned()).collect(),
			Data::Keccak256(ref h) => once(36u8).chain(h.iter().cloned()).collect(),
			Data::ShaThree256(ref h) => once(37u8).chain(h.iter().cloned()).collect(),
		}
	}
}
impl Decode for Data {
	fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
		let b = input.read_byte()?;
		Ok(match b {
			0 => Data::None,
			n @ 1..=33 => {
				let mut r = vec![0u8; n as usize - 1];
				input.read(&mut r[..])?;
				Data::Raw(r)
			},
			34 => Data::BlakeTwo256(<[u8; 32]>::decode(input)?),
			35 => Data::Sha256(<[u8; 32]>::decode(input)?),
			36 => Data::Keccak256(<[u8; 32]>::decode(input)?),
			37 => Data::ShaThree256(<[u8; 32]>::decode(input)?),
			_ => return Err(Error::from("invalid leading byte")),
		})
	}
}
impl EncodeLike for Data {}
// Copied from substrate repo
pub struct AppendZerosInput<'a, T>(&'a mut T);
impl<'a, T> AppendZerosInput<'a, T> {
	pub fn new(input: &'a mut T) -> Self {
		Self(input)
	}
}
impl<'a, T: Input> Input for AppendZerosInput<'a, T> {
	fn remaining_len(&mut self) -> Result<Option<usize>, Error> {
		Ok(None)
	}

	fn read(&mut self, into: &mut [u8]) -> Result<(), Error> {
		let remaining = self.0.remaining_len()?;
		let completed = if let Some(n) = remaining {
			let readable = into.len().min(n);
			// this should never fail if `remaining_len` API is implemented correctly.
			self.0.read(&mut into[..readable])?;
			readable
		} else {
			// Fill it byte-by-byte.
			let mut i = 0;
			while i < into.len() {
				if let Ok(b) = self.0.read_byte() {
					into[i] = b;
					i += 1;
				} else {
					break;
				}
			}
			i
		};
		// Fill the rest with zeros.
		for i in &mut into[completed..] {
			*i = 0;
		}
		Ok(())
	}
}

#[derive(Debug, Encode, Decode, PartialEq, Eq)]
pub struct RegistrarInfo<A> {
	pub account: A,
	pub fee: u128,
	pub fields: IdentityFields,
}
#[derive(Debug, PartialEq, Eq)]
pub struct IdentityFields(pub BitFlags<IdentityField>);
impl Encode for IdentityFields {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl Decode for IdentityFields {
	fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
		let field = u64::decode(input)?;
		Ok(Self(<BitFlags<IdentityField>>::from_bits(field as u64).map_err(|_| "invalid value")?))
	}
}
#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IdentityField {
	Display = 0b0000000000000000000000000000000000000000000000000000000000000001,
	Legal = 0b0000000000000000000000000000000000000000000000000000000000000010,
	Web = 0b0000000000000000000000000000000000000000000000000000000000000100,
	Riot = 0b0000000000000000000000000000000000000000000000000000000000001000,
	Email = 0b0000000000000000000000000000000000000000000000000000000000010000,
	PgpFingerprint = 0b0000000000000000000000000000000000000000000000000000000000100000,
	Image = 0b0000000000000000000000000000000000000000000000000000000001000000,
	Twitter = 0b0000000000000000000000000000000000000000000000000000000010000000,
}
