// --- paritytech ---
use pallet_indices::Config;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const IndexDeposit: Balance = 1 * COIN;
}

impl Config for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Ring;
	type Deposit = IndexDeposit;
	type Event = Event;
	type WeightInfo = ();
}
