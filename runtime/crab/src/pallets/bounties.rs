// --- paritytech ---
use sp_runtime::Permill;
// --- darwinia-network ---
use crate::*;
use pallet_bounties::Config;

frame_support::parameter_types! {
	pub const BountyDepositBase: Balance = crab_deposit(1, 0) + CRAB_PROPOSAL_REQUIREMENT;
	pub const BountyDepositPayoutDelay: BlockNumber = 4 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 100_000 * COIN;
}

impl Config for Runtime {
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type Event = Event;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = ();
}
