//! Mock file for eth-backing.

//! Test utilities

//! Test utilities

use hex_literal::hex;
use std::{cell::RefCell, collections::HashSet};

use frame_support::{
	assert_ok, impl_outer_origin, parameter_types,
	traits::{Currency, FindAuthor, Get},
	weights::Weight,
	StorageLinkedMap, StorageValue,
};
use sp_core::{crypto::key_types, H256};
use sp_io;
use sp_runtime::{
	testing::{Header, UintAuthorityId},
	traits::{IdentityLookup, OnInitialize, OpaqueKeys, SaturatedConversion},
	{KeyTypeId, Perbill},
};
use sp_staking::offence::{OffenceDetails, OnOffenceHandler};

use darwinia_phragmen::{PhragmenStakedAssignment, Power, Votes};

use crate::*;

pub type AccountId = u64;
pub type BlockNumber = u64;
pub type Balance = u128;

pub type System = frame_system::Module<Test>;
//pub type Session = pallet_session::Module<Test>;
pub type Timestamp = pallet_timestamp::Module<Test>;

pub type EthBacking = Module<Test>;
pub type Ring = pallet_ring::Module<Test>;
pub type Kton = pallet_kton::Module<Test>;
pub type Staking = pallet_staking::Module<Test>;
pub type EthRelay = darwinia_eth_relay::Module<Test>;

pub const NANO: Balance = 1;
pub const MICRO: Balance = 1_000 * NANO;
pub const MILLI: Balance = 1_000 * MICRO;
pub const COIN: Balance = 1_000 * MILLI;

pub const CAP: Balance = 10_000_000_000 * COIN;
pub const GENESIS_TIME: Moment = 0;
pub const TOTAL_POWER: Power = 1_000_000_000;

impl_outer_origin! {
	pub enum Origin for Test  where system = system {}
}

/// Author of block is always 11
pub struct Author11;
impl FindAuthor<u64> for Author11 {
	fn find_author<'a, I>(_digests: I) -> Option<u64>
	where
		I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
	{
		Some(11)
	}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
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
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
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
	type AccountData = darwinia_support::balance::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
}
impl pallet_kton::Trait for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ();
	type AccountStore = System;
	type TryDropRing = ();
}
impl pallet_ring::Trait for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ();
	type AccountStore = System;
	type TryDropKton = ();
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

impl darwinia_eth_relay::Trait for Test {
	type Event = ();
	type EthNetwork = EthRopsten;
}

parameter_types! {
	pub const SessionsPerEra: SessionIndex = 3;
	pub const BondingDurationInEra: EraIndex = 3;
	// assume 60 blocks per session
	pub const BondingDurationInBlockNumber: BlockNumber = 3 * 3 * 60;

	pub const Cap: Balance = CAP;
	pub const TotalPower: Power = TOTAL_POWER;
	pub const GenesisTime: Moment = GENESIS_TIME;
}

impl pallet_staking::Trait for Test {
	type Time = Timestamp;
	type Event = ();
	type SessionsPerEra = SessionsPerEra;
	type BondingDurationInEra = ();
	type BondingDurationInBlockNumber = ();
	type SlashDeferDuration = ();
	type SlashCancelOrigin = system::EnsureRoot<Self::AccountId>;
	type SessionInterface = ();
	type RingCurrency = Ring;
	type RingRewardRemainder = ();
	type RingSlash = ();
	type RingReward = ();
	type KtonCurrency = Kton;
	type KtonSlash = ();
	type KtonReward = ();
	type Cap = Cap;
	type TotalPower = TotalPower;
	type GenesisTime = GenesisTime;
}

impl Trait for Test {
	type Event = ();
	type Time = Timestamp;
	type EthRelay = EthRelay;
	type Ring = Ring;
	type Kton = Kton;
	type OnDepositRedeem = Staking;
	type DetermineAccountId = AccountIdDeterminator<Test>;
	type RingReward = ();
	type KtonReward = ();
}

pub struct ExtBuilder;
impl Default for ExtBuilder {
	fn default() -> Self {
		Self
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		self.set_associated_consts();

		let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let _ = GenesisConfig::<Test> {
			ring_redeem_address: hex!["dbc888d701167cbfb86486c516aafbefc3a4de6e"].into(),
			kton_redeem_address: hex!["dbc888d701167cbfb86486c516aafbefc3a4de6e"].into(),
			deposit_redeem_address: hex!["6ef538314829efa8386fc43386cb13b4e0a67d1e"].into(),
			ring_locked: 20000000000000,
			kton_locked: 5000000000000,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
