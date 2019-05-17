#![cfg(test)]
extern crate sr_io as runtime_io;
use primitives::BuildStorage;
use primitives::{traits::{IdentityLookup}, testing::{Digest, DigestItem, Header}};
use substrate_primitives::{H256, Blake2Hasher};
use srml_support::impl_outer_origin;
use crate::{GenesisConfig, Module, Trait};

impl_outer_origin!{
	pub enum Origin for Test {}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

impl system::Trait for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = ::primitives::traits::BlakeTwo256;
    type Digest = Digest;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type Log = DigestItem;
}

impl timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
}


impl ring::Trait for Test {
    type Balance = u64;
    type OnFreeBalanceZero = ();
    type OnNewAccount = ();
    type Event = ();
    type TransactionPayment = ();
    type TransferPayment = ();
    type DustRemoval = ();
}

impl Trait for Test {
    type Balance = u64;
    type Currency = ring::Module<Self>;
    type Event = ();
}

pub struct ExtBuilder {
    transaction_base_fee: u64,
    transaction_byte_fee: u64,
    existential_deposit: u64,
    transfer_fee: u64,
    creation_fee: u64,
    sys_account: u64,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            transaction_base_fee: 0,
            transaction_byte_fee: 0,
            existential_deposit: 0,
            transfer_fee: 0,
            creation_fee: 0,
            sys_account: 0
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
    pub fn creation_fee(mut self, creation_fee: u64) -> Self {
        self.creation_fee = creation_fee;
        self
    }
    pub fn transaction_fees(mut self, base_fee: u64, byte_fee: u64) -> Self {
        self.transaction_base_fee = base_fee;
        self.transaction_byte_fee = byte_fee;
        self
    }


    pub fn build(self) -> runtime_io::TestExternalities<Blake2Hasher> {
        let (mut t, mut c) = system::GenesisConfig::<Test>::default().build_storage().unwrap();
        let balance_factor = if self.existential_deposit > 0 {
            256
        } else {
            1
        };

        let _ = timestamp::GenesisConfig::<Test> {
            minimum_period: 5,
        }.assimilate_storage(&mut t, &mut c);

        let _ = ring::GenesisConfig::<Test> {
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
            transaction_base_fee: self.transaction_base_fee,
            transaction_byte_fee: self.transaction_byte_fee,
            existential_deposit: self.existential_deposit,
            transfer_fee: self.transfer_fee,
            creation_fee: self.creation_fee,
            vesting: vec![],
        }.assimilate_storage(&mut t, &mut c);

        let _ = GenesisConfig::<Test> {
            sys_account: 42,
        }.assimilate_storage(&mut t, &mut c);

        t.into()

    }
}

pub type System = system::Module<Test>;
pub type Ring = ring::Module<Test>;
pub type Timestamp = timestamp::Module<Test>;
pub type Kton = Module<Test>;