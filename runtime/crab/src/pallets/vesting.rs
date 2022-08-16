// --- darwinia-network ---
use crate::*;
// --- paritytech ---
use pallet_vesting::Config;
use sp_runtime::traits::ConvertInto;

frame_support::parameter_types! {
	pub const MinVestedTransfer: Balance = COIN;
}

impl Config for Runtime {
	type BlockNumberToBalance = ConvertInto;
	type Currency = Ring;
	type Event = Event;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = ();

	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
}
