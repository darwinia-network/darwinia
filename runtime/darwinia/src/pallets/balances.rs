// --- darwinia-network ---
use crate::{weights::darwinia_balances::WeightInfo, *};
use darwinia_balances::Config;

frame_support::parameter_types! {
	pub const RingExistentialDeposit: Balance = 100 * MICRO;
	pub const KtonExistentialDeposit: Balance = MICRO;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl Config<RingInstance> for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = RingExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = WeightInfo<Runtime>;

	type BalanceInfo = AccountData<Balance>;
	type OtherCurrencies = (Kton,);
}
impl Config<KtonInstance> for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = KtonExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = WeightInfo<Runtime>;

	type BalanceInfo = AccountData<Balance>;
	type OtherCurrencies = (Ring,);
}
