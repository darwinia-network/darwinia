
#![cfg(test)]
use runtime_io;
use primitives::BuildStorage;
use primitives::{traits::{IdentityLookup}, testing::{Header}};
use substrate_primitives::{H256, Blake2Hasher};
use srml_support::{ impl_outer_origin, traits::Get };
use crate::{GenesisConfig, Module, Trait};
use std::cell::RefCell;
use node_runtime::COIN;
use super::*;

impl_outer_origin!{
	pub enum Origin for Test {}
}

thread_local! {
	static EXISTENTIAL_DEPOSIT: RefCell<u128> = RefCell::new(0);
	static TRANSFER_FEE: RefCell<u128> = RefCell::new(0);
	static CREATION_FEE: RefCell<u128> = RefCell::new(0);
	static TRANSACTION_BASE_FEE: RefCell<u128> = RefCell::new(0);
	static TRANSACTION_BYTE_FEE: RefCell<u128> = RefCell::new(0);
}


pub struct ExistentialDeposit;
impl Get<u128> for ExistentialDeposit {
    fn get() -> u128 { EXISTENTIAL_DEPOSIT.with(|v| *v.borrow()) }
}

pub struct TransferFee;
impl Get<u128> for TransferFee {
    fn get() -> u128 { TRANSFER_FEE.with(|v| *v.borrow()) }
}

pub struct CreationFee;
impl Get<u128> for CreationFee {
    fn get() -> u128 { CREATION_FEE.with(|v| *v.borrow()) }
}

pub struct TransactionBaseFee;
impl Get<u128> for TransactionBaseFee {
    fn get() -> u128 { TRANSACTION_BASE_FEE.with(|v| *v.borrow()) }
}

pub struct TransactionByteFee;
impl Get<u128> for TransactionByteFee {
    fn get() -> u128 { TRANSACTION_BYTE_FEE.with(|v| *v.borrow()) }
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

impl system::Trait for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = ::primitives::traits::BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
}

impl timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
}


impl balances::Trait for Test {
    type Balance = u128;
    type OnFreeBalanceZero = ();
    type OnNewAccount = ();
    type Event = ();
    type TransactionPayment = ();
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
}

impl Trait for Test {
    type Balance = u128;
    type Currency = balances::Module<Self>;
    type Event = ();
    type OnMinted = ();
    type OnRemoval = ();
    type SystemRefund = ();
}

pub struct ExtBuilder {
    transaction_base_fee: u128,
    transaction_byte_fee: u128,
    existential_deposit: u128,
    transfer_fee: u128,
    creation_fee: u128,
    sys_acc: u64,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            transaction_base_fee: 0,
            transaction_byte_fee: 0,
            existential_deposit: 0,
            transfer_fee: 0,
            creation_fee: 0,
            sys_acc: 0
        }
    }
}


impl ExtBuilder {
    pub fn existential_deposit(mut self, existential_deposit: u128) -> Self {
        self.existential_deposit = existential_deposit;
        self
    }

    #[allow(dead_code)]
    pub fn transfer_fee(mut self, transfer_fee: u128) -> Self {
        self.transfer_fee = transfer_fee;
        self
    }
    pub fn creation_fee(mut self, creation_fee: u128) -> Self {
        self.creation_fee = creation_fee;
        self
    }
    pub fn transaction_fees(mut self, base_fee: u128, byte_fee: u128) -> Self {
        self.transaction_base_fee = base_fee;
        self.transaction_byte_fee = byte_fee;
        self
    }

    pub fn set_associated_consts(&self) {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
        TRANSFER_FEE.with(|v| *v.borrow_mut() = self.transfer_fee);
        CREATION_FEE.with(|v| *v.borrow_mut() = self.creation_fee);
        TRANSACTION_BASE_FEE.with(|v| *v.borrow_mut() = self.transaction_base_fee);
        TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.transaction_byte_fee);
    }


    pub fn build(self) -> runtime_io::TestExternalities<Blake2Hasher> {
        self.set_associated_consts();
        let (mut t, mut c) = system::GenesisConfig::default().build_storage::<Test>().unwrap();
        let balance_factor = if self.existential_deposit > 0 {
            1000 * COIN
        } else {
            1 * COIN
        };

        let _ = timestamp::GenesisConfig::<Test> {
            minimum_period: 5,
        }.assimilate_storage(&mut t, &mut c);

        let _ = balances::GenesisConfig::<Test> {
            balances: vec![
                (1, 10 * balance_factor),
                (2, 20 * balance_factor),
                (3, 300 * balance_factor),
                (4, 400 * balance_factor),
                (10, balance_factor),
                (11, balance_factor * 10000000), // 10 b
                (20, balance_factor),
                (21, balance_factor * 2000000), // 2 b
                (30, balance_factor),
                (31, balance_factor * 2000), // 2 m
                (40, balance_factor),
                (41, balance_factor * 2000), // 2 m
                (100, 200000 * balance_factor),
                (101, 200000 * balance_factor),
            ],
            vesting: vec![],
        }.assimilate_storage(&mut t, &mut c);

        let _ = GenesisConfig::<Test> {
            sys_acc: 42,
            balances: vec![
                (1, 10 * balance_factor, 12),
                (2, 20 * balance_factor, 12),
                (3, 300 * balance_factor, 12),
                (4, 400 * balance_factor, 12),
            ],
            vesting: vec![],
        }.assimilate_storage(&mut t, &mut c);
        t.into()

    }
}

pub type System = system::Module<Test>;
pub type Ring = balances::Module<Test>;
pub type Timestamp = timestamp::Module<Test>;
pub type Kton = Module<Test>;
