// --- darwinia ---
use crate::{weights::darwinia_vesting::WeightInfo, *};
use darwinia_vesting::Config;

frame_support::parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * MILLI;
}
impl Config for Runtime {
	type Event = Event;
	type Currency = Ring;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = WeightInfo<Runtime>;
}
