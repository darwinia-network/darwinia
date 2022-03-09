// --- darwinia-network ---
use crate::*;
use darwinia_vesting::Config;
// --- paritytech ---
use sp_runtime::traits::ConvertInto;

frame_support::parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * MILLI;
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Ring;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = ();

	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
}
