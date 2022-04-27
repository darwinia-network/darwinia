// --- darwinia-network ---
use crate::*;
use darwinia_balances::Config;

frame_support::parameter_types! {
	pub const RingExistentialDeposit: Balance = 0;
	pub const KtonExistentialDeposit: Balance = 0;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl Config<RingInstance> for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type BalanceInfo = AccountData<Balance>;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = RingExistentialDeposit;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type OtherCurrencies = (Kton,);
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}
impl Config<KtonInstance> for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type BalanceInfo = AccountData<Balance>;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = KtonExistentialDeposit;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type OtherCurrencies = (Ring,);
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}
