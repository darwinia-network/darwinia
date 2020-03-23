// --- third-party ---
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, Extrinsic as ExtrinsicsT, IdentityLookup},
	Perbill,
};

// --- custom ---
use crate::*;

impl_outer_origin! {
	pub enum Origin for Test  where system = frame_system {}
}

pub type EthOffchain = Module<Test>;

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Trait for Test {
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = sp_core::H256;
	type Hashing = BlakeTwo256;
	type AccountId = sp_core::sr25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = ();
	type MigrateAccount = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
}

type Extrinsic = TestXt<Call<Test>, ()>;
type SubmitTransaction = frame_system::offchain::TransactionSubmitter<crypto::Public, Test, Extrinsic>;

impl frame_system::offchain::CreateTransaction<Test, Extrinsic> for Test {
	type Public = sp_core::sr25519::Public;
	type Signature = sp_core::sr25519::Signature;

	fn create_transaction<F: frame_system::offchain::Signer<Self::Public, Self::Signature>>(
		call: <Extrinsic as ExtrinsicsT>::Call,
		_public: Self::Public,
		_account: <Test as frame_system::Trait>::AccountId,
		nonce: <Test as frame_system::Trait>::Index,
	) -> Option<(
		<Extrinsic as ExtrinsicsT>::Call,
		<Extrinsic as ExtrinsicsT>::SignaturePayload,
	)> {
		Some((call, (nonce, ())))
	}
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Trait for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
	pub const EthRopsten: u64 = 1;
}
impl pallet_eth_relay::Trait for Test {
	type Event = ();
	type EthNetwork = EthRopsten;
}

impl Trait for Test {
	type Event = ();
	type Time = pallet_timestamp::Module<Self>;
	type Call = Call<Test>;
	type SubmitSignedTransaction = SubmitTransaction;
	type BlockFetchDur = ();
	type APIKey = ();
}
