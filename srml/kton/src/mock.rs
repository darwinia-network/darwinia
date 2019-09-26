use super::*;
use crate::{GenesisConfig, Module};
use primitives::testing::Header;
use primitives::traits::IdentityLookup;
use srml_support::impl_outer_origin;
use std::{cell::RefCell, collections::HashSet};
use substrate_primitives::{Blake2Hasher, H256};

const COIN: u64 = 1_000_000_000;

thread_local! {
	static SESSION: RefCell<(Vec<AccountId>, HashSet<AccountId>)> = RefCell::new(Default::default());
	static EXISTENTIAL_DEPOSIT: RefCell<u64> = RefCell::new(0);
}

/// The AccountId alias in this test module.
pub type AccountId = u64;
// pub type BlockNumber = u64;
pub type Balance = u64;

#[allow(unused_doc_comments)]
/// Simple structure that exposes how u64 currency can be represented as... u64.
// pub struct CurrencyToVoteHandler;
// impl Convert<u64, u64> for CurrencyToVoteHandler {
// fn convert(x: u64) -> u64 {
// x
// }
// }
// impl Convert<u128, u64> for CurrencyToVoteHandler {
// fn convert(x: u128) -> u64 {
// x as u64
// }
// }

// pub struct ExistentialDeposit;
// impl Get<u64> for ExistentialDeposit {
// fn get() -> u64 {
// EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
// }
// }

impl_outer_origin! {
	pub enum Origin for Test {}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

impl system::Trait for Test {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = ::primitives::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
}

impl timestamp::Trait for Test {
	type Moment = u64;
	type OnTimestampSet = ();
}

impl Trait for Test {
	type Balance = Balance;
	type Event = ();
	type OnMinted = ();
	type OnRemoval = ();
}

pub struct ExtBuilder {
	existential_deposit: u64,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { existential_deposit: 0 }
	}
}

impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: u64) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}

	pub fn set_associated_consts(&self) {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
	}

	pub fn build(self) -> runtime_io::TestExternalities<Blake2Hasher> {
		self.set_associated_consts();
		let (mut t, mut c) = system::GenesisConfig::default().build_storage::<Test>().unwrap();
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
			vesting: vec![],
		}
		.assimilate_storage(&mut t, &mut c);
		t.into()
	}
}
pub type System = system::Module<Test>;
pub type Kton = Module<Test>;
// pub type Timestamp = timestamp::Module<Test>;
