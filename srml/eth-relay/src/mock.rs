//! Test utilities

use crate::{Module, Trait};
use primitives::H256;
use runtime_io;
use sr_primitives::{
	testing::Header,
	traits::IdentityLookup,
	//	weights::{DispatchInfo, Weight},
	Perbill,
};
use std::cell::RefCell;
//use support::traits::Get;
use support::{impl_outer_origin, parameter_types};

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

thread_local! {
	static EXISTENTIAL_DEPOSIT: RefCell<u64> = RefCell::new(0);
	static TRANSFER_FEE: RefCell<u64> = RefCell::new(0);
	static CREATION_FEE: RefCell<u64> = RefCell::new(0);
}

//pub struct ExistentialDeposit;
//impl Get<u64> for ExistentialDeposit {
//	fn get() -> u64 {
//		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
//	}
//}
//
//pub struct TransferFee;
//impl Get<u64> for TransferFee {
//	fn get() -> u64 {
//		TRANSFER_FEE.with(|v| *v.borrow())
//	}
//}
//
//pub struct CreationFee;
//impl Get<u64> for CreationFee {
//	fn get() -> u64 {
//		CREATION_FEE.with(|v| *v.borrow())
//	}
//}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runtime;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl system::Trait for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = ();
	type Hash = H256;
	type Hashing = ::sr_primitives::traits::BlakeTwo256;
	type AccountId = u64;
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
	pub const TransactionBaseFee: u64 = 0;
	pub const TransactionByteFee: u64 = 1;
}

impl Trait for Runtime {
	type Event = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

pub struct ExtBuilder {
	existential_deposit: u64,
	transfer_fee: u64,
	creation_fee: u64,
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
	pub fn existential_deposit(mut self, existential_deposit: u64) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}
	#[allow(dead_code)]
	pub fn transfer_fee(mut self, transfer_fee: u64) -> Self {
		self.transfer_fee = transfer_fee;
		self
	}
	#[allow(dead_code)]
	pub fn creation_fee(mut self, creation_fee: u64) -> Self {
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
	#[allow(dead_code)]
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
		let t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

		t.into()
	}
}

pub type System = system::Module<Runtime>;
pub type EthRelay = Module<Runtime>;

//pub const CALL: &<Runtime as system::Trait>::Call = &();

// create a transaction info struct from weight. Handy to avoid building the whole struct.
//pub fn info_from_weight(w: Weight) -> DispatchInfo {
//	DispatchInfo {
//		weight: w,
//		..Default::default()
//	}
//}
