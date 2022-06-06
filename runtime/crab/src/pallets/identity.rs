// --- paritytech ---
use pallet_identity::Config;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	// Minimum 100 bytes/CRAB deposited (1 MILLI/byte)
	pub const BasicDeposit: Balance = 10 * COIN;       // 258 bytes on-chain
	pub const FieldDeposit: Balance = 250 * MILLI;     // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 2 * COIN;   // 53 bytes on-chain
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl Config for Runtime {
	type BasicDeposit = BasicDeposit;
	type Currency = Ring;
	type Event = Event;
	type FieldDeposit = FieldDeposit;
	type ForceOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type MaxSubAccounts = MaxSubAccounts;
	type RegistrarOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type Slashed = Treasury;
	type SubAccountDeposit = SubAccountDeposit;
	type WeightInfo = ();
}
