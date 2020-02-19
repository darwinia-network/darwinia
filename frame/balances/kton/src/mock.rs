use std::cell::RefCell;

use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{
		BlakeTwo256,
		//ConvertInto,
		IdentityLookup,
	},
	Perbill,
};

use crate::*;

pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type Index = u64;
pub type Moment = u64;

pub type System = system::Module<Test>;

pub type Ring = darwinia_ring::Module<Test>;
pub type Kton = Module<Test>;

impl_outer_origin! {
	pub enum Origin for Test {}
}

#[cfg(feature = "with-fee")]
thread_local! {
	pub(crate) static EXISTENTIAL_DEPOSIT: RefCell<Balance> = RefCell::new(1 * COIN);
	static TRANSFER_FEE: RefCell<Balance> = RefCell::new(1 * MILLI);
	static CREATION_FEE: RefCell<Balance> = RefCell::new(1 * MILLI);
}
#[cfg(not(feature = "with-fee"))]
thread_local! {
	pub(crate) static EXISTENTIAL_DEPOSIT: RefCell<Balance> = RefCell::new(0);
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
	type Index = Index;
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
	type ModuleToIndex = ();
}
impl darwinia_ring::Trait for Test {
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
impl Trait for Test {
	type Balance = Balance;
	type Event = ();
	type RingCurrency = Ring;
	type TransferPayment = Ring;
	type ExistentialDeposit = ();
	type TransferFee = ();
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
	#[allow(dead_code)]
	pub fn existential_deposit(mut self, existential_deposit: Balance) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}
	#[allow(dead_code)]
	pub fn transfer_fee(mut self, transfer_fee: Balance) -> Self {
		self.transfer_fee = transfer_fee;
		self
	}
	#[allow(dead_code)]
	pub fn creation_fee(mut self, creation_fee: Balance) -> Self {
		self.creation_fee = creation_fee;
		self
	}
	#[allow(dead_code)]
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
	pub fn build(self) -> sp_io::TestExternalities {
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
