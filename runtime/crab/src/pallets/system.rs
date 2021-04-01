// --- substrate ---
use frame_support::weights::constants::RocksDbWeight;
use frame_system::Config;
use sp_runtime::traits::BlakeTwo256;
use sp_version::RuntimeVersion;
// --- darwinia ---
use crate::{weights::frame_system::WeightInfo, *};

frame_support::parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u8 = 42;
}
impl Config for Runtime {
	type BaseCallFilter = ();
	type BlockWeights = BlockWeights;
	type BlockLength = BlockLength;
	type Origin = Origin;
	type Call = Call;
	type Index = Nonce;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = WeightInfo<Runtime>;
	type SS58Prefix = SS58Prefix;
}
