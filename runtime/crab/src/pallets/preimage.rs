// --- paritytech ---
use pallet_preimage::Config;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub const PreimageBaseDeposit: Balance = 500 * COIN;
	pub const PreimageByteDeposit: Balance = crab_deposit(0, 1);
}

impl Config for Runtime {
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
	type Currency = Balances;
	type Event = Event;
	type ManagerOrigin = Root;
	type MaxSize = PreimageMaxSize;
	type WeightInfo = ();
}
