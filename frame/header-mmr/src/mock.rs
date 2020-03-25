//! Test utilities

#![cfg(test)]

// --- third-party ---
use codec::{Decode, Encode};
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, DigestItem, Perbill};

// --- custom ---
use crate::*;

impl_outer_origin! {
	pub enum Origin for Test  where system = frame_system {}
}

// --- substrate ---
pub type System = frame_system::Module<Test>;

// --- custom ---
pub type HeaderMMR = Module<Test>;

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug, Decode, Encode)]
pub struct Test;

impl Trait for Test {}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Trait for Test {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = ();
	type Hash = H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = u64;
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

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

pub fn header_mmr_log(hash: H256) -> DigestItem<H256> {
	DigestItem::MerkleMountainRangeRoot(hash)
}

pub fn initialize_block(number: u64, parent_hash: H256) {
	System::initialize(
		&number,
		&parent_hash,
		&Default::default(),
		&Default::default(),
		Default::default(),
	);
}
