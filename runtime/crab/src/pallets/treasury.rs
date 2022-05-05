// --- paritytech ---
use frame_support::PalletId;
use sp_runtime::Permill;
// --- darwinia-network ---
use crate::*;
use pallet_treasury::{Config, Instance2 as KtonTreasuryInstance};

frame_support::parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"da/trsry");
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const RingProposalBondMinimum: Balance = 10_000 * COIN;
	pub const KtonProposalBondMinimum: Balance = 10 * COIN;
	pub const SpendPeriod: BlockNumber = 6 * DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const MaxApprovals: u32 = 100;
}

// In order to use `Tips`, which bounded by `pallet_treasury::Config` rather
// `pallet_treasury::Config<I>` Still use `DefaultInstance` here instead `Instance1`
impl Config for Runtime {
	type ApproveOrigin = ApproveOrigin;
	type Burn = Burn;
	type BurnDestination = Society;
	type Currency = Ring;
	type Event = Event;
	type MaxApprovals = MaxApprovals;
	type OnSlash = Treasury;
	type PalletId = TreasuryPalletId;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = RingProposalBondMinimum;
	type RejectOrigin = EnsureRootOrMoreThanHalfCouncil;
	type SpendFunds = Bounties;
	type SpendPeriod = SpendPeriod;
	type WeightInfo = ();
}
impl Config<KtonTreasuryInstance> for Runtime {
	type ApproveOrigin = ApproveOrigin;
	type Burn = Burn;
	type BurnDestination = ();
	type Currency = Kton;
	type Event = Event;
	type MaxApprovals = MaxApprovals;
	type OnSlash = KtonTreasury;
	type PalletId = TreasuryPalletId;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = KtonProposalBondMinimum;
	type RejectOrigin = EnsureRootOrMoreThanHalfCouncil;
	type SpendFunds = ();
	type SpendPeriod = SpendPeriod;
	type WeightInfo = ();
}
