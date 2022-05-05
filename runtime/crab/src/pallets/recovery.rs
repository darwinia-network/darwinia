// --- paritytech ---
use pallet_recovery::Config;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const ConfigDepositBase: Balance = 5 * COIN;
	pub const FriendDepositFactor: Balance = 50 * MILLI;
	pub const MaxFriends: u16 = 9;
	pub const RecoveryDeposit: Balance = 5 * COIN;
}

impl Config for Runtime {
	type Call = Call;
	type ConfigDepositBase = ConfigDepositBase;
	type Currency = Ring;
	type Event = Event;
	type FriendDepositFactor = FriendDepositFactor;
	type MaxFriends = MaxFriends;
	type RecoveryDeposit = RecoveryDeposit;
}
