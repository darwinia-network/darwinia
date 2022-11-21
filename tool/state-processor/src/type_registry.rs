// crates.io
use parity_scale_codec::{Decode, Encode};

pub const GWEI: u128 = 1_000_000_000;

#[derive(Debug, Encode, Decode)]
pub struct AccountInfo {
	pub nonce: u32,
	pub consumers: u32,
	pub providers: u32,
	pub sufficients: u32,
	pub data: AccountData,
}
#[derive(Debug, Encode, Decode)]
pub struct AccountData {
	pub free: u128,
	pub reserved: u128,
	pub free_kton_or_misc_frozen: u128,
	pub reserved_kton_or_fee_frozen: u128,
}

#[derive(Debug, Encode, Decode)]
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
