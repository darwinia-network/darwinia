//! Test utilities

#![cfg(test)]

use crate::{Module, Trait};
use codec::{Decode, Encode};
use frame_support::{impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
use sp_core::H256;
use sp_io;
use sp_runtime::{testing::Header, traits::IdentityLookup, DigestItem, Perbill};

use frame_system as system;
impl_outer_origin! {
	pub enum Origin for Test  where system = frame_system {}
}

pub fn header_mmr_log(hash: H256) -> DigestItem<H256> {
	DigestItem::MerkleMountainRangeRoot(hash)
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug, Decode, Encode)]
pub struct Test;

impl Trait for Test {
	type Event = TestEvent;
}
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
	type Event = TestEvent;
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

mod headermmr {
	pub use crate::Event;
}

impl_outer_event! {
	pub enum TestEvent for Test {
		system<T>,
		headermmr<T>,
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	//	GenesisConfig {}.assimilate_storage::<Test>(&mut t).unwrap();
	t.into()
}

pub type System = frame_system::Module<Test>;
pub type HeaderMMR = Module<Test>;
