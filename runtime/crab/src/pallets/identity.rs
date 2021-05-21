// --- substrate ---
use pallet_identity::Config;
// --- darwinia ---
use crate::{weights::pallet_identity::WeightInfo, *};

frame_support::parameter_types! {
	// Minimum 100 bytes/CRING deposited (1 MILLI/byte)
	pub const BasicDeposit: Balance = 10 * COIN;       // 258 bytes on-chain
	pub const FieldDeposit: Balance = 250 * MILLI;     // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 2 * COIN;   // 53 bytes on-chain
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Ring;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRootOrMoreThanHalfCouncil;
	type RegistrarOrigin = EnsureRootOrMoreThanHalfCouncil;
	type WeightInfo = WeightInfo<Runtime>;
}
