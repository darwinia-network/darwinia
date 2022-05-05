// --- paritytech ---
use sp_runtime::Percent;
// --- darwinia-network ---
use crate::*;
use pallet_tips::Config;

frame_support::parameter_types! {
	pub const DataDepositPerByte: Balance = 1 * MILLI;
	pub const MaximumReasonLength: u32 = 16384;
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * COIN;
}

impl Config for Runtime {
	type DataDepositPerByte = DataDepositPerByte;
	type Event = Event;
	type MaximumReasonLength = MaximumReasonLength;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type Tippers = PhragmenElection;
	type WeightInfo = ();
}
