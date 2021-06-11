// --- substrate ---
use pallet_recovery::Config;
// --- darwinia ---
use crate::*;

frame_support::parameter_types! {
	pub const ConfigDepositBase: Balance = 5 * COIN;
	pub const FriendDepositFactor: Balance = 50 * MILLI;
	pub const MaxFriends: u16 = 9;
	pub const RecoveryDeposit: Balance = 5 * COIN;
}

impl Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Ring;
	type ConfigDepositBase = ConfigDepositBase;
	type FriendDepositFactor = FriendDepositFactor;
	type MaxFriends = MaxFriends;
	type RecoveryDeposit = RecoveryDeposit;
}
