// --- paritytech ---
use pallet_identity::Config;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	// Minimum 100 bytes/CRAB deposited (1 MILLI/byte)
	pub const BasicDeposit: Balance = darwinia_deposit(1, 258);
	pub const FieldDeposit: Balance = darwinia_deposit(0, 66);
	pub const SubAccountDeposit: Balance = darwinia_deposit(1, 53);
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl Config for Runtime {
	type BasicDeposit = BasicDeposit;
	type Currency = Ring;
	type Event = Event;
	type FieldDeposit = FieldDeposit;
	type ForceOrigin = MoreThanHalf<CouncilCollective>;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type MaxSubAccounts = MaxSubAccounts;
	type RegistrarOrigin = MoreThanHalf<CouncilCollective>;
	type Slashed = Treasury;
	type SubAccountDeposit = SubAccountDeposit;
	type WeightInfo = ();
}
