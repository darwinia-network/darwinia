// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Test utilities
use std::cell::RefCell;

use primitives::H256;
use sr_primitives::{
	testing::Header,
	traits::{BlakeTwo256, ConvertInto, IdentityLookup},
	weights::{DispatchInfo, Weight},
	Perbill,
};
use support::{impl_outer_origin, parameter_types, traits::Get};

use crate::*;

/// The AccountId alias in this test module.
pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type Moment = u64;

pub type System = system::Module<Test>;
pub type Timestamp = timestamp::Module<Test>;

pub type Balances = Module<Test>;

pub const CALL: &<Test as system::Trait>::Call = &();

impl_outer_origin! {
	pub enum Origin for Test {}
}

thread_local! {
	static EXISTENTIAL_DEPOSIT: RefCell<Balance> = RefCell::new(0);
	static TRANSFER_FEE: RefCell<Balance> = RefCell::new(0);
	static CREATION_FEE: RefCell<Balance> = RefCell::new(0);
}

pub struct ExistentialDeposit;
impl Get<Balance> for ExistentialDeposit {
	fn get() -> Balance {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
	}
}

pub struct TransferFee;
impl Get<Balance> for TransferFee {
	fn get() -> Balance {
		TRANSFER_FEE.with(|v| *v.borrow())
	}
}

pub struct CreationFee;
impl Get<Balance> for CreationFee {
	fn get() -> Balance {
		CREATION_FEE.with(|v| *v.borrow())
	}
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
	pub const TransactionBaseFee: Balance = 0;
	pub const TransactionByteFee: Balance = 1;
}
impl transaction_payment::Trait for Test {
	type Currency = Module<Test>;
	type OnTransactionPayment = ();
	type TransactionBaseFee = TransactionBaseFee;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = ConvertInto;
	type FeeMultiplierUpdate = ();
}

parameter_types! {
	pub const MinimumPeriod: Moment = 5;
}

impl timestamp::Trait for Test {
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
}

impl Trait for Test {
	type Balance = Balance;
	type OnFreeBalanceZero = ();
	type OnNewAccount = ();
	type TransferPayment = ();
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ExistentialDeposit;
	type TransferFee = TransferFee;
	type CreationFee = CreationFee;
}

pub struct ExtBuilder {
	existential_deposit: Balance,
	transfer_fee: Balance,
	creation_fee: Balance,
	monied: bool,
	vesting: bool,
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			existential_deposit: 0,
			transfer_fee: 0,
			creation_fee: 0,
			monied: false,
			vesting: false,
		}
	}
}
impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: Balance) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}
	#[allow(dead_code)]
	pub fn transfer_fee(mut self, transfer_fee: Balance) -> Self {
		self.transfer_fee = transfer_fee;
		self
	}
	pub fn creation_fee(mut self, creation_fee: Balance) -> Self {
		self.creation_fee = creation_fee;
		self
	}
	pub fn monied(mut self, monied: bool) -> Self {
		self.monied = monied;
		if self.existential_deposit == 0 {
			self.existential_deposit = 1;
		}
		self
	}
	pub fn vesting(mut self, vesting: bool) -> Self {
		self.vesting = vesting;
		self
	}
	pub fn set_associated_consts(&self) {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
		TRANSFER_FEE.with(|v| *v.borrow_mut() = self.transfer_fee);
		CREATION_FEE.with(|v| *v.borrow_mut() = self.creation_fee);
	}
	pub fn build(self) -> runtime_io::TestExternalities {
		self.set_associated_consts();
		let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
		GenesisConfig::<Test> {
			balances: if self.monied {
				vec![
					(1, 10 * self.existential_deposit),
					(2, 20 * self.existential_deposit),
					(3, 30 * self.existential_deposit),
					(4, 40 * self.existential_deposit),
					(12, 10 * self.existential_deposit),
				]
			} else {
				vec![]
			},
			vesting: if self.vesting && self.monied {
				vec![
					(1, 0, 10, 5 * self.existential_deposit),
					(2, 10, 20, 0),
					(12, 10, 20, 5 * self.existential_deposit),
				]
			} else {
				vec![]
			},
		}
		.assimilate_storage(&mut t)
		.unwrap();
		t.into()
	}
}

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
	DispatchInfo {
		weight: w,
		..Default::default()
	}
}
