use crate::*;
use frame_support::{assert_noop, assert_ok, impl_outer_origin, parameter_types, weights::Weight};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, OnFinalize},
	Perbill,
};

type Ring = darwinia_ring::Module<Test>;
type Kton = darwinia_kton::Module<Test>;
type Treasury = Module<Test>;

impl_outer_origin! {
	pub enum Origin for Test  where system = frame_system {}
}

#[derive(Clone, Eq, PartialEq)]
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
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
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
parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const TransferFee: u64 = 0;
	pub const CreationFee: u64 = 0;
}
impl darwinia_kton::Trait for Test {
	type Balance = u64;
	type Event = ();
	type RingCurrency = Ring;
	type TransferPayment = ();
	type ExistentialDeposit = ExistentialDeposit;
	type TransferFee = TransferFee;
}
impl darwinia_ring::Trait for Test {
	type Balance = u64;
	type OnFreeBalanceZero = ();
	type OnNewAccount = ();
	type TransferPayment = ();
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ExistentialDeposit;
	type TransferFee = TransferFee;
	type CreationFee = CreationFee;
}
parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const RingProposalBondMinimum: u64 = 1;
	pub const KtonProposalBondMinimum: u64 = 1;
	pub const SpendPeriod: u64 = 2;
	pub const Burn: Permill = Permill::from_percent(50);
}
impl Trait for Test {
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
	type ApproveOrigin = frame_system::EnsureRoot<u64>;
	type RejectOrigin = frame_system::EnsureRoot<u64>;
	type Event = ();
	type KtonProposalRejection = ();
	type RingProposalRejection = ();
	type ProposalBond = ProposalBond;
	type RingProposalBondMinimum = RingProposalBondMinimum;
	type KtonProposalBondMinimum = KtonProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
}

fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let _ = darwinia_ring::GenesisConfig::<Test> {
		// Total issuance will be 200 with treasury account initialized at ED.
		balances: vec![(0, 100), (1, 98), (2, 1)],
		vesting: vec![],
	}
	.assimilate_storage(&mut t);
	let _ = darwinia_kton::GenesisConfig::<Test> {
		// Total issuance will be 200 with treasury account initialized at ED.
		balances: vec![(0, 100), (1, 98), (2, 1)],
		vesting: vec![],
	}
	.assimilate_storage(&mut t);
	let _ = GenesisConfig::default().assimilate_storage::<Test>(&mut t);
	t.into()
}

#[test]
fn genesis_config_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Treasury::pot::<Ring>(), 0);
		assert_eq!(Treasury::pot::<Kton>(), 0);
		assert_eq!(Treasury::proposal_count(), 0);
	});
}

#[test]
fn minting_works() {
	new_test_ext().execute_with(|| {
		// Check that accumulate works when we have Some value in Dummy already.
		Ring::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Ring>(), 100);
		assert_eq!(Treasury::pot::<Kton>(), 0);

		// Make sure kton and ring have different storages
		Kton::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Kton>(), 100);
	});
}

/// min deposit is 0 now
#[test]
fn spend_proposal_takes_min_deposit() {
	new_test_ext().execute_with(|| {
		// ring
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::RingBalance(1),
			3
		));
		assert_eq!(Ring::free_balance(&0), 99);
		assert_eq!(Ring::reserved_balance(&0), 1);

		// kton
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::KtonBalance(1),
			3
		));
		assert_eq!(Kton::free_balance(&0), 99);
		assert_eq!(Kton::reserved_balance(&0), 1);
	});
}

#[test]
fn spend_proposal_takes_proportional_deposit() {
	new_test_ext().execute_with(|| {
		// ring
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::RingBalance(100),
			3
		));
		assert_eq!(Ring::free_balance(&0), 95);
		assert_eq!(Ring::reserved_balance(&0), 5);

		// kton
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::KtonBalance(100),
			3
		));
		assert_eq!(Kton::free_balance(&0), 95);
		assert_eq!(Kton::reserved_balance(&0), 5);
	});
}

#[test]
fn spend_proposal_fails_when_proposer_poor() {
	new_test_ext().execute_with(|| {
		// ring
		assert_noop!(
			Treasury::propose_spend(Origin::signed(2), StakingBalance::RingBalance(100), 3),
			Error::<Test>::InsufficientProposersBalance,
		);

		// kton
		assert_noop!(
			Treasury::propose_spend(Origin::signed(2), StakingBalance::KtonBalance(100), 3),
			Error::<Test>::InsufficientProposersBalance,
		);
	});
}

#[test]
fn accepted_spend_proposal_ignored_outside_spend_period() {
	new_test_ext().execute_with(|| {
		// ring
		Ring::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::RingBalance(100),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 0));

		<Treasury as OnFinalize<u64>>::on_finalize(1);
		assert_eq!(Ring::free_balance(&3), 0);
		assert_eq!(Treasury::pot::<Ring>(), 100);

		// kton
		Kton::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::KtonBalance(100),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 1));

		<Treasury as OnFinalize<u64>>::on_finalize(3);
		assert_eq!(Kton::free_balance(&3), 0);
		assert_eq!(Treasury::pot::<Kton>(), 100);
	});
}

#[test]
fn unused_pot_should_diminish() {
	new_test_ext().execute_with(|| {
		let init_total_ring_issuance = Ring::total_issuance();
		let init_total_kton_issuance = Kton::total_issuance();

		// ring
		Ring::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Ring::total_issuance(), init_total_ring_issuance + 100);

		<Treasury as OnFinalize<u64>>::on_finalize(2);
		assert_eq!(Treasury::pot::<Ring>(), 50);
		assert_eq!(Ring::total_issuance(), init_total_ring_issuance + 50);

		// kton
		Kton::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Kton::total_issuance(), init_total_kton_issuance + 100);

		<Treasury as OnFinalize<u64>>::on_finalize(2);
		assert_eq!(Treasury::pot::<Kton>(), 50);
		assert_eq!(Kton::total_issuance(), init_total_kton_issuance + 50);
	});
}

#[test]
fn rejected_spend_proposal_ignored_on_spend_period() {
	new_test_ext().execute_with(|| {
		// ring
		Ring::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::RingBalance(100),
			3
		));
		assert_ok!(Treasury::reject_proposal(Origin::ROOT, 0));

		<Treasury as OnFinalize<u64>>::on_finalize(2);
		assert_eq!(Ring::free_balance(&3), 0);
		assert_eq!(Treasury::pot::<Ring>(), 50);

		// kton
		Kton::make_free_balance_be(&Treasury::account_id(), 101);

		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::KtonBalance(100),
			3
		));
		assert_ok!(Treasury::reject_proposal(Origin::ROOT, 1));

		<Treasury as OnFinalize<u64>>::on_finalize(2);
		assert_eq!(Kton::free_balance(&3), 0);
		assert_eq!(Treasury::pot::<Kton>(), 50);
	});
}

#[test]
fn reject_already_rejected_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		// ring
		Ring::make_free_balance_be(&Treasury::account_id(), 101);

		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::RingBalance(100),
			3
		));
		assert_ok!(Treasury::reject_proposal(Origin::ROOT, 0));
		assert_noop!(
			Treasury::reject_proposal(Origin::ROOT, 0),
			Error::<Test>::InvalidProposalIndex
		);

		// kton
		Kton::make_free_balance_be(&Treasury::account_id(), 101);

		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::KtonBalance(100),
			3
		));
		assert_ok!(Treasury::reject_proposal(Origin::ROOT, 1));
		assert_noop!(
			Treasury::reject_proposal(Origin::ROOT, 1),
			Error::<Test>::InvalidProposalIndex
		);
	});
}

#[test]
fn reject_non_existant_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Treasury::reject_proposal(Origin::ROOT, 0),
			Error::<Test>::InvalidProposalIndex
		);
	});
}

#[test]
fn accept_non_existant_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Treasury::approve_proposal(Origin::ROOT, 0),
			Error::<Test>::InvalidProposalIndex
		);
	});
}

#[test]
fn accept_already_rejected_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		// ring
		Ring::make_free_balance_be(&Treasury::account_id(), 101);

		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::RingBalance(100),
			3
		));
		assert_ok!(Treasury::reject_proposal(Origin::ROOT, 0));
		assert_noop!(
			Treasury::approve_proposal(Origin::ROOT, 0),
			Error::<Test>::InvalidProposalIndex
		);

		// kton
		Kton::make_free_balance_be(&Treasury::account_id(), 101);

		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::KtonBalance(100),
			3
		));
		assert_ok!(Treasury::reject_proposal(Origin::ROOT, 1));
		assert_noop!(
			Treasury::approve_proposal(Origin::ROOT, 1),
			Error::<Test>::InvalidProposalIndex
		);
	});
}

#[test]
fn accepted_spend_proposal_enacted_on_spend_period() {
	new_test_ext().execute_with(|| {
		// ring
		Ring::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Ring>(), 100);

		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::RingBalance(100),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 0));

		<Treasury as OnFinalize<u64>>::on_finalize(2);
		assert_eq!(Ring::free_balance(&3), 100);
		assert_eq!(Treasury::pot::<Ring>(), 0);
	});

	// kton
	new_test_ext().execute_with(|| {
		Kton::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Kton>(), 100);

		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::KtonBalance(100),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 0));

		<Treasury as OnFinalize<u64>>::on_finalize(2);
		assert_eq!(Kton::free_balance(&3), 100);
		assert_eq!(Treasury::pot::<Kton>(), 0);
	});
}

#[test]
fn pot_underflow_should_not_diminish() {
	new_test_ext().execute_with(|| {
		// ring
		Ring::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Ring>(), 100);
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::RingBalance(150),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 0));

		<Treasury as OnFinalize<u64>>::on_finalize(2);
		assert_eq!(Treasury::pot::<Ring>(), 100); // Pot hasn't changed

		let _ = Ring::deposit_into_existing(&Treasury::account_id(), 100).unwrap();
		<Treasury as OnFinalize<u64>>::on_finalize(4);
		assert_eq!(Ring::free_balance(&3), 150); // Fund has been spent
		assert_eq!(Treasury::pot::<Ring>(), 25); // Pot has finally changed

		// kton
		Kton::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Kton>(), 100);
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::KtonBalance(150),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 1));

		<Treasury as OnFinalize<u64>>::on_finalize(3);
		assert_eq!(Treasury::pot::<Kton>(), 100); // Pot hasn't changed

		let _ = Kton::deposit_into_existing(&Treasury::account_id(), 100).unwrap();
		<Treasury as OnFinalize<u64>>::on_finalize(4);
		assert_eq!(Kton::free_balance(&3), 150); // Fund has been spent
		assert_eq!(Treasury::pot::<Kton>(), 25); // Pot has finally changed
	});
}

// Treasury account doesn't get deleted if amount approved to spend is all its free balance.
// i.e. pot should not include existential deposit needed for account survival.
#[test]
fn treasury_account_doesnt_get_deleted() {
	new_test_ext().execute_with(|| {
		Ring::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Ring>(), 100);
		let ring_treasury_balance = StakingBalance::RingBalance(Ring::free_balance(&Treasury::account_id()));

		assert_ok!(Treasury::propose_spend(Origin::signed(0), ring_treasury_balance, 3));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 0));

		<Treasury as OnFinalize<u64>>::on_finalize(2);
		assert_eq!(Treasury::pot::<Ring>(), 100); // Pot hasn't changed

		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::RingBalance(Treasury::pot::<Ring>()),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 1));

		<Treasury as OnFinalize<u64>>::on_finalize(4);
		assert_eq!(Treasury::pot::<Ring>(), 0); // Pot is emptied
		assert_eq!(Ring::free_balance(&Treasury::account_id()), 1); // but the account is still there
	});

	new_test_ext().execute_with(|| {
		Kton::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Kton>(), 100);
		let kton_treasury_balance = StakingBalance::KtonBalance(Kton::free_balance(&Treasury::account_id()));

		assert_ok!(Treasury::propose_spend(Origin::signed(0), kton_treasury_balance, 3));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 0));

		<Treasury as OnFinalize<u64>>::on_finalize(2);
		assert_eq!(Treasury::pot::<Kton>(), 100); // Pot hasn't changed

		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::KtonBalance(Treasury::pot::<Kton>()),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 1));

		<Treasury as OnFinalize<u64>>::on_finalize(4);
		assert_eq!(Treasury::pot::<Kton>(), 0); // Pot is emptied
		assert_eq!(Kton::free_balance(&Treasury::account_id()), 1); // but the account is still there
	});
}

// In case treasury account is not existing then it works fine.
// This is usefull for chain that will just update runtime.
#[test]
fn inexisting_account_works() {
	let mut tr = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut tk = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	darwinia_ring::GenesisConfig::<Test> {
		balances: vec![(0, 100), (1, 99), (2, 1)],
		vesting: vec![],
	}
	.assimilate_storage(&mut tr)
	.unwrap();

	darwinia_kton::GenesisConfig::<Test> {
		balances: vec![(0, 100), (1, 99), (2, 1)],
		vesting: vec![],
	}
	.assimilate_storage(&mut tk)
	.unwrap();

	// Treasury genesis config is not build thus treasury account does not exist
	let mut tr: sp_io::TestExternalities = tr.into();
	let mut tk: sp_io::TestExternalities = tk.into();

	tr.execute_with(|| {
		// Account does not exist
		assert_eq!(Ring::free_balance(&Treasury::account_id()), 0);

		// Pot is empty
		assert_eq!(Treasury::pot::<Ring>(), 0);
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::RingBalance(99),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 0));
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::RingBalance(1),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 1));

		<Treasury as OnFinalize<u64>>::on_finalize(2);
		// Pot hasn't changed
		assert_eq!(Treasury::pot::<Ring>(), 0);
		// Balance of `3` hasn't changed
		assert_eq!(Ring::free_balance(&3), 0);

		Ring::make_free_balance_be(&Treasury::account_id(), 100);
		assert_eq!(Treasury::pot::<Ring>(), 99); // Pot now contains funds
		assert_eq!(Ring::free_balance(&Treasury::account_id()), 100); // Account does exist

		<Treasury as OnFinalize<u64>>::on_finalize(4);

		assert_eq!(Treasury::pot::<Ring>(), 0); // Pot has changed
		assert_eq!(Ring::free_balance(&3), 99); // Balance of `3` has changed
	});

	tk.execute_with(|| {
		// Account does not exist
		assert_eq!(Kton::free_balance(&Treasury::account_id()), 0);

		// Pot is empty
		assert_eq!(Treasury::pot::<Kton>(), 0);
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::KtonBalance(99),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 0));
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			StakingBalance::KtonBalance(1),
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::ROOT, 1));

		<Treasury as OnFinalize<u64>>::on_finalize(2);
		// Pot hasn't changed
		assert_eq!(Treasury::pot::<Kton>(), 0);
		// Balance of `3` hasn't changed
		assert_eq!(Kton::free_balance(&3), 0);

		Kton::make_free_balance_be(&Treasury::account_id(), 100);
		assert_eq!(Treasury::pot::<Kton>(), 99); // Pot now contains funds
		assert_eq!(Kton::free_balance(&Treasury::account_id()), 100); // Account does exist

		<Treasury as OnFinalize<u64>>::on_finalize(4);

		assert_eq!(Treasury::pot::<Kton>(), 0); // Pot has changed
		assert_eq!(Kton::free_balance(&3), 99); // Balance of `3` has changed
	});
}
