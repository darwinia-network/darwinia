pub use node_primitives::Balance;
pub use node_runtime::constants::currency::COIN;

use std::{cell::RefCell, collections::HashSet};

use sr_primitives::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	weights::Weight,
	Perbill,
};
use srml_support::{impl_outer_origin, parameter_types};
use substrate_primitives::H256;

use super::*;
use crate::{GenesisConfig, Module};

thread_local! {
	static SESSION: RefCell<(Vec<AccountId>, HashSet<AccountId>)> = RefCell::new(Default::default());
	static EXISTENTIAL_DEPOSIT: RefCell<Balance> = RefCell::new(0);
}

/// The AccountId alias in this test module.
pub type AccountId = u64;
// FIXME:
//     replace
//     	  testing::Header.number: u64
//     with
//         node_primitives::BlockNumber
pub type BlockNumber = u64;

impl_outer_origin! {
	pub enum Origin for Test {}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl system::Trait for Test {
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}
impl timestamp::Trait for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
}

impl Trait for Test {
	type Balance = Balance;
	type Event = ();
	type OnMinted = ();
	type OnRemoval = ();
}

pub struct ExtBuilder {
	existential_deposit: Balance,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { existential_deposit: 0 }
	}
}

impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: Balance) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}

	pub fn set_associated_consts(&self) {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
	}

	pub fn build(self) -> runtime_io::TestExternalities {
		self.set_associated_consts();
		let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
		let balance_factor = if self.existential_deposit > 0 {
			1_000 * COIN
		} else {
			1 * COIN
		};

		let _ = GenesisConfig::<Test> {
			balances: vec![
				(1, 10 * balance_factor),
				(2, 20 * balance_factor),
				(3, 300 * balance_factor),
				(4, 400 * balance_factor),
				(10, balance_factor),
				(11, balance_factor * 1000),
				(20, balance_factor),
				(21, balance_factor * 2000),
				(30, balance_factor),
				(31, balance_factor * 2000),
				(40, balance_factor),
				(41, balance_factor * 2000),
				(100, 2000 * balance_factor),
				(101, 2000 * balance_factor),
			],
			vesting: vec![(1, 0, 4)],
		}
		.assimilate_storage(&mut t);

		t.into()
	}
}

pub type Timestamp = timestamp::Module<Test>;
pub type System = system::Module<Test>;
pub type Kton = Module<Test>;
