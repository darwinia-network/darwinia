// --- paritytech ---
use frame_support::{traits::Contains, weights::constants::RocksDbWeight};
use frame_system::Config;
use sp_version::RuntimeVersion;
// --- darwinia-network ---
use crate::{weights::frame_system::WeightInfo, *};

pub struct BaseFilter;
impl Contains<Call> for BaseFilter {
	fn contains(_: &Call) -> bool {
		true
	}
}

frame_support::parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u16 = 42;
}

impl Config for Runtime {
	type BaseCallFilter = BaseFilter;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type DbWeight = RocksDbWeight;
	type Origin = Origin;
	type Call = Call;
	type Index = Nonce;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = Hashing;
	type AccountId = AccountId;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCountForCrab;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = WeightInfo<Runtime>;
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}
