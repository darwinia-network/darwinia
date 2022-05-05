// --- paritytech ---
use pallet_multisig::Config;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = crab_deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = crab_deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl Config for Runtime {
	type Call = Call;
	type Currency = Ring;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type Event = Event;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = ();
}
