//! Test utilities

use crate::*;
use phragmen::{build_support_map, elect, equalize, ExtendedBalance as Power, PhragmenStakedAssignment};
use sr_primitives::{
	impl_opaque_keys,
	testing::{Header, UintAuthorityId},
	traits::{Convert, IdentityLookup, OpaqueKeys},
	weights::Weight,
	AccountId32, KeyTypeId, Perbill,
};
use std::{cell::RefCell, collections::HashSet};

use sr_staking_primitives::SessionIndex;
use support::{
	impl_outer_origin, parameter_types,
	traits::{Currency, FindAuthor, Get},
	ConsensusEngineId, StorageLinkedMap,
};

use primitives::{crypto::key_types, H256};

use hex_literal::hex;
use rustc_hex::FromHex;

/// The AccountId alias in this test module.
pub type AccountId = AccountId32;
pub type BlockNumber = u64;

pub type System = system::Module<Test>;

pub type EthRelay = darwinia_eth_relay::Module<Test>;

pub type EthBacking = Module<Test>;

pub type Ring = balances::Module<Test>;
pub type Kton = kton::Module<Test>;

pub type Timestamp = timestamp::Module<Test>;

pub type Staking = staking::Module<Test>;

pub type Balance = u128;
pub type Moment = u64;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

pub type Points = u32;

pub const NANO: Balance = 1;
pub const MICRO: Balance = 1_000 * NANO;
pub const MILLI: Balance = 1_000 * MICRO;
pub const COIN: Balance = 1_000 * MILLI;

/// Simple structure that exposes how u64 currency can be represented as... u64.
pub struct CurrencyToVoteHandler;
impl Convert<u64, u64> for CurrencyToVoteHandler {
	fn convert(x: u64) -> u64 {
		x
	}
}
impl Convert<u128, u128> for CurrencyToVoteHandler {
	fn convert(x: u128) -> u128 {
		x
	}
}
impl Convert<u128, u64> for CurrencyToVoteHandler {
	fn convert(x: u128) -> u64 {
		x as u64
	}
}

thread_local! {
//	static SESSION: RefCell<(Vec<AccountId>, HashSet<AccountId>)> = RefCell::new(Default::default());
	static EXISTENTIAL_DEPOSIT: RefCell<Balance> = RefCell::new(0);
}

pub struct TestSessionHandler;
impl session::SessionHandler<AccountId> for TestSessionHandler {
	const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];

	fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AccountId, Ks)]) {}

	fn on_new_session<Ks: OpaqueKeys>(
		_changed: bool,
		validators: &[(AccountId, Ks)],
		_queued_validators: &[(AccountId, Ks)],
	) {
		//		SESSION.with(|x| *x.borrow_mut() = (validators.iter().map(|x| x.0.clone()).collect(), HashSet::new()));
	}

	fn on_disabled(validator_index: usize) {
		//		SESSION.with(|d| {
		//			let mut d = d.borrow_mut();
		//			let value = d.0[validator_index];
		//			d.1.insert(value);
		//		})
	}
}

pub fn is_disabled(controller: AccountId) -> bool {
	//	let stash = Staking::ledger(&controller).unwrap().stash;
	//	SESSION.with(|d| d.borrow().1.contains(&stash))
	false
}

pub struct ExistentialDeposit;
impl Get<Balance> for ExistentialDeposit {
	fn get() -> Balance {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
	}
}

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
	type Hashing = ::sr_primitives::traits::BlakeTwo256;
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
	pub const Period: BlockNumber = 1;
	pub const Offset: BlockNumber = 0;
	pub const UncleGenerations: u64 = 0;
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(25);
}
impl session::Trait for Test {
	type Event = ();
	type ValidatorId = AccountId;
	type ValidatorIdOf = staking::StashOf<Test>;
	type ShouldEndSession = session::PeriodicSessions<Period, Offset>;
	type OnSessionEnding = session::historical::NoteHistoricalRoot<Test, Staking>;
	type SessionHandler = TestSessionHandler; // <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = UintAuthorityId;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type SelectInitialValidators = Staking;
}

impl session::historical::Trait for Test {
	type FullIdentification = staking::Exposure<AccountId, Power>;
	type FullIdentificationOf = staking::ExposureOf<Test>;
}

impl authorship::Trait for Test {
	type FindAuthor = ();
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = Staking;
}

parameter_types! {
	pub const MinimumPeriod: Moment = 5;
}
impl timestamp::Trait for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
	pub const TransferFee: Balance = 0;
	pub const CreationFee: Balance = 0;
}
impl balances::Trait for Test {
	type Balance = Balance;
	type OnFreeBalanceZero = Staking;
	type OnNewAccount = ();
	type TransferPayment = ();
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ExistentialDeposit;
	type TransferFee = TransferFee;
	type CreationFee = CreationFee;
}
impl kton::Trait for Test {
	type Event = ();
}

parameter_types! {
	pub const SessionsPerEra: SessionIndex = 3;
	pub const BondingDuration: Moment = 60;
	pub const BondingDurationInEra: EraIndex = 60;
	pub const CAP: Balance = 10_000_000_000 * COIN;
	pub const GenesisTime: Moment = 0;
}
impl staking::Trait for Test {
	type Time = Timestamp;
	type CurrencyToVote = ();
	type Event = ();
	type SessionsPerEra = ();
	type BondingDuration = ();
	type BondingDurationInEra = ();
	type SessionInterface = Self;
	type Ring = Ring;
	type RingRewardRemainder = ();
	type RingSlash = ();
	type RingReward = ();
	type Kton = Kton;
	type KtonSlash = ();
	type KtonReward = ();

	type Cap = CAP;
	type GenesisTime = GenesisTime;
}

parameter_types! {
//	pub const EthMainet: u64 = 0;
	pub const EthRopsten: u64 = 1;
}

impl darwinia_eth_relay::Trait for Test {
	type Event = ();
	type EthNetwork = EthRopsten;
}

impl Trait for Test {
	type Event = ();
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
	pub fn build(self) -> runtime_io::TestExternalities {
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
