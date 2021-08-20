// --- paritytech ---
use sp_runtime::Permill;
// --- darwinia-network ---
use crate::*;
use pallet_bounties::Config;

#[cfg(feature = "dev")]
frame_support::parameter_types! {
	pub const BountyDepositPayoutDelay: BlockNumber = 3 * MINUTES;
	pub const BountyUpdatePeriod: BlockNumber = 3 * MINUTES;
}
#[cfg(not(feature = "dev"))]
frame_support::parameter_types! {
	pub const BountyDepositPayoutDelay: BlockNumber = 8 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
}
frame_support::parameter_types! {
	pub const BountyDepositBase: Balance = 1 * COIN;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 10 * COIN;
}

impl Config for Runtime {
	type Event = Event;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = ();
}
