// --- darwinia ---
use crate::{weights::darwinia_balances::WeightInfo, *};
use darwinia_balances::Config;

frame_support::parameter_types! {
	pub const RingExistentialDeposit: Balance = 0;
	pub const KtonExistentialDeposit: Balance = 0;
	pub const MaxLocks: u32 = 50;
}

impl Config<RingInstance> for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = RingExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type OtherCurrencies = (Kton,);
	type MaxLocks = MaxLocks;
	type WeightInfo = WeightInfo<Runtime>;
}
impl Config<KtonInstance> for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = KtonExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type OtherCurrencies = (Ring,);
	type MaxLocks = MaxLocks;
	type WeightInfo = WeightInfo<Runtime>;
}
