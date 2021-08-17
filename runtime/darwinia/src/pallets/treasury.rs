// --- paritytech ---
use frame_support::PalletId;
use sp_runtime::{Percent, Permill};
// --- darwinia-network ---
use crate::{weights::darwinia_treasury::WeightInfo, *};
use darwinia_treasury::Config;

#[cfg(feature = "dev")]
frame_support::parameter_types! {
	pub const SpendPeriod: BlockNumber = 3 * MINUTES;
	pub const TipCountdown: BlockNumber = 3 * MINUTES;
	pub const BountyDepositPayoutDelay: BlockNumber = 3 * MINUTES;
	pub const BountyUpdatePeriod: BlockNumber = 3 * MINUTES;
}
#[cfg(not(feature = "dev"))]
frame_support::parameter_types! {
	pub const SpendPeriod: BlockNumber = 24 * DAYS;
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const BountyDepositPayoutDelay: BlockNumber = 8 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
}
frame_support::parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"da/trsry");
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const RingProposalBondMinimum: Balance = 100 * MILLI;
	pub const KtonProposalBondMinimum: Balance = 100 * MILLI;
	pub const Burn: Permill = Permill::from_percent(1);
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * MILLI;
	pub const DataDepositPerByte: Balance = 1 * MILLI;
	pub const BountyDepositBase: Balance = 1 * COIN;
	pub const MaximumReasonLength: u32 = 16384;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 10 * COIN;
	pub const MaxApprovals: u32 = 100;
}

impl Config for Runtime {
	type PalletId = TreasuryPalletId;
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
	type ApproveOrigin = ApproveOrigin;
	type RejectOrigin = EnsureRootOrMoreThanHalfCouncil;
	type Tippers = PhragmenElection;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type DataDepositPerByte = DataDepositPerByte;
	type Event = Event;
	type OnSlashRing = Treasury;
	type OnSlashKton = Treasury;
	type ProposalBond = ProposalBond;
	type RingProposalBondMinimum = RingProposalBondMinimum;
	type KtonProposalBondMinimum = KtonProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type MaximumReasonLength = MaximumReasonLength;
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyValueMinimum = BountyValueMinimum;
	type RingBurnDestination = ();
	type KtonBurnDestination = ();
	type MaxApprovals = MaxApprovals;
	type WeightInfo = WeightInfo<Runtime>;
}
