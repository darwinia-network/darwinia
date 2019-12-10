use sr_primitives::{assert_eq_error_rate, traits::OnInitialize};
use srml_support::{
	assert_eq_uvec, assert_err, assert_noop, assert_ok,
	traits::{Currency, ReservableCurrency},
};

use crate::{mock::*, *};
use darwinia_support::{BalanceLock, NormalLock, StakingLock, WithdrawLock, WithdrawReason, WithdrawReasons};

/// gen_paired_account!(a(1), b(2), m(12));
/// will create stash `a` and controller `b`
/// `a` has 100 Ring and 100 Kton
/// promise for `m` month with 50 Ring and 50 Kton
///
/// `m` can be ignore, this won't create variable `m`
/// ```rust
/// gen_parired_account!(a(1), b(2), 12);
/// ```
///
/// `m(12)` can be ignore, and it won't perform `bond` action
/// ```rust
/// gen_paired_account!(a(1), b(2));
/// ```
macro_rules! gen_paired_account {
	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr), $promise_month:ident($how_long:expr)) => {
		#[allow(non_snake_case, unused)]
		let $stash = $stash_id;
		let _ = Ring::deposit_creating(&$stash, 100 * COIN);
		Kton::deposit_creating(&$stash, 100 * COIN);
		#[allow(non_snake_case, unused)]
		let $controller = $controller_id;
		let _ = Ring::deposit_creating(&$controller, COIN);
		#[allow(non_snake_case, unused)]
		let $promise_month = $how_long;
		assert_ok!(Staking::bond(
			Origin::signed($stash),
			$controller,
			StakingBalance::Ring(50 * COIN),
			RewardDestination::Stash,
			$how_long,
			));
		assert_ok!(Staking::bond_extra(
			Origin::signed($stash),
			StakingBalance::Kton(50 * COIN),
			$how_long
			));
	};
	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr), $how_long:expr) => {
		#[allow(non_snake_case, unused)]
		let $stash = $stash_id;
		let _ = Ring::deposit_creating(&$stash, 100 * COIN);
		Kton::deposit_creating(&$stash, 100 * COIN);
		#[allow(non_snake_case, unused)]
		let $controller = $controller_id;
		let _ = Ring::deposit_creating(&$controller, COIN);
		assert_ok!(Staking::bond(
			Origin::signed($stash),
			$controller,
			StakingBalance::Ring(50 * COIN),
			RewardDestination::Stash,
			$how_long,
			));
		assert_ok!(Staking::bond_extra(
			Origin::signed($stash),
			StakingBalance::Kton(50 * COIN),
			$how_long,
			));
	};
	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr)) => {
		#[allow(non_snake_case, unused)]
		let $stash = $stash_id;
		let _ = Ring::deposit_creating(&$stash, 100 * COIN);
		Kton::deposit_creating(&$stash, 100 * COIN);
		#[allow(non_snake_case, unused)]
		let $controller = $controller_id;
		let _ = Ring::deposit_creating(&$controller, COIN);
	};
}

#[test]
fn force_unstake_works() {
	// Verifies initial conditions of mock.
	ExtBuilder::default().build().execute_with(|| {
		// Account 11 is stashed and locked, and account 10 is the controller.
		assert_eq!(Staking::bonded(&11), Some(10));
		// Cant transfer.
		assert_noop!(
			Ring::transfer(Origin::signed(11), 1, 10),
			"account liquidity restrictions prevent withdrawal",
		);
		// Force unstake requires root.
		assert_noop!(Staking::force_unstake(Origin::signed(11), 11), "RequireRootOrigin");
		// We now force them to unstake.
		assert_ok!(Staking::force_unstake(Origin::ROOT, 11));
		// No longer bonded.
		assert_eq!(Staking::bonded(&11), None);
		// Transfer works.
		assert_ok!(Ring::transfer(Origin::signed(11), 1, 10));
	});
}

#[test]
fn basic_setup_works() {
	// Verifies initial conditions of mock.
	ExtBuilder::default().build().execute_with(|| {
		// Account 11 is stashed and locked, and account 10 is the controller.
		assert_eq!(Staking::bonded(&11), Some(10));
		// Account 21 is stashed and locked, and account 20 is the controller.
		assert_eq!(Staking::bonded(&21), Some(20));
		// Account 1 is not a stashed.
		assert_eq!(Staking::bonded(&1), None);

		// Account 10 controls the stash from account 11, which is 100 * balance_factor units.
		assert_eq!(
			Staking::ledger(&10),
			Some(StakingLedger {
				stash: 11,
				active_ring: 1000,
				active_deposit_ring: 0,
				active_kton: 0,
				deposit_items: vec![],
				ring_staking_lock: StakingLock {
					staking_amount: 1000,
					unbondings: vec![],
				},
				kton_staking_lock: Default::default(),
			})
		);
		// Account 20 controls the stash from account 21, which is 200 * balance_factor units.
		assert_eq!(
			Staking::ledger(&20),
			Some(StakingLedger {
				stash: 21,
				active_ring: 1000,
				active_deposit_ring: 0,
				active_kton: 0,
				deposit_items: vec![],
				ring_staking_lock: StakingLock {
					staking_amount: 1000,
					unbondings: vec![],
				},
				kton_staking_lock: Default::default(),
			})
		);
		// Account 1 does not control any stash.
		assert_eq!(Staking::ledger(&1), None);

		// ValidatorPrefs are default.
		{
			let validator_prefs = ValidatorPrefs {
				node_name: "Darwinia Node".bytes().collect(),
				..Default::default()
			};
			assert_eq!(
				<Validators<Test>>::enumerate().collect::<Vec<_>>(),
				vec![
					(31, validator_prefs.clone()),
					(21, validator_prefs.clone()),
					(11, validator_prefs.clone()),
				]
			);
		}

		assert_eq!(
			Staking::ledger(100),
			Some(StakingLedger {
				stash: 101,
				active_ring: 500,
				active_deposit_ring: 0,
				active_kton: 0,
				deposit_items: vec![],
				ring_staking_lock: StakingLock {
					staking_amount: 500,
					unbondings: vec![],
				},
				kton_staking_lock: Default::default(),
			})
		);
		assert_eq!(Staking::nominators(101), vec![11, 21]);

		if cfg!(feature = "equalize") {
			let vote_form_101_per_validator = Staking::power_of(&101) / 2;

			let exposure_own_of_11 = Staking::power_of(&11);
			let exposure_total_of_11 = exposure_own_of_11 + vote_form_101_per_validator;

			let exposure_own_of_21 = Staking::power_of(&21);
			let exposure_total_of_21 = exposure_own_of_21 + vote_form_101_per_validator;

			assert_eq!(
				Staking::stakers(11),
				Exposure {
					total: exposure_total_of_11,
					own: exposure_own_of_11,
					others: vec![IndividualExposure {
						who: 101,
						value: vote_form_101_per_validator,
					}],
				}
			);
			assert_eq!(
				Staking::stakers(21),
				Exposure {
					total: exposure_total_of_21,
					own: exposure_own_of_21,
					others: vec![IndividualExposure {
						who: 101,
						value: vote_form_101_per_validator,
					}],
				}
			);
			// initial slot_stake.
			assert_eq!(exposure_total_of_11, exposure_total_of_21);
			assert_eq!(Staking::slot_stake(), exposure_total_of_11);
		} else {
			let vote_of_101 = Staking::power_of(&101);

			let exposure_own_of_11 = Staking::power_of(&11);
			let exposure_others_of_11 = vote_of_101 * 4 / 1;
			let exposure_total_of_11 = exposure_own_of_11 + exposure_others_of_11;

			assert_eq!(
				Staking::stakers(11),
				Exposure {
					total: exposure_total_of_11,
					own: exposure_own_of_11,
					others: vec![IndividualExposure {
						who: 101,
						value: exposure_others_of_11,
					}],
				}
			);
			assert_eq!(
				Staking::stakers(21),
				Exposure {
					total: Staking::power_of(&21),
					own: 1000,
					others: vec![IndividualExposure {
						who: 101,
						value: vote_of_101 * 4 / 3,
					}],
				}
			);
			// initial slot_stake.
			assert_eq!(Staking::slot_stake(), exposure_total_of_11);
		}

		// The number of validators required.
		assert_eq!(Staking::validator_count(), 2);

		// Initial Era and session.
		assert_eq!(Staking::current_era(), 0);

		// Account 10 has `balance_factor` free balance.
		assert_eq!(Ring::free_balance(&10), 1);
		assert_eq!(Ring::free_balance(&10), 1);

		// New era is not being forced.
		assert_eq!(Staking::force_era(), Forcing::NotForcing);

		// All exposures must be correct.
		check_exposure_all();
		check_nominator_all();
	});
}

#[test]
fn change_controller_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Staking::bonded(&11), Some(10));

		assert!(<Validators<Test>>::enumerate()
			.map(|(c, _)| c)
			.collect::<Vec<AccountId>>()
			.contains(&11));
		// 10 can control 11 who is initially a validator.
		assert_ok!(Staking::chill(Origin::signed(10)));
		assert!(!<Validators<Test>>::enumerate()
			.map(|(c, _)| c)
			.collect::<Vec<AccountId>>()
			.contains(&11));

		assert_ok!(Staking::set_controller(Origin::signed(11), 5));

		start_era(1);

		assert_noop!(
			Staking::validate(
				Origin::signed(10),
				ValidatorPrefs {
					node_name: "Darwinia Node".bytes().collect(),
					..Default::default()
				}
			),
			err::CONTROLLER_INVALID,
		);
		assert_ok!(Staking::validate(
			Origin::signed(5),
			ValidatorPrefs {
				node_name: "Darwinia Node".bytes().collect(),
				..Default::default()
			}
		));
	})
}

// TODO
#[test]
fn rewards_should_work() {
	// should check that:
	// * rewards get recorded per session
	// * rewards get paid per Era
	// * Check that nominators are also rewarded
	ExtBuilder::default().nominate(false).build().execute_with(|| {
		// Init some balances.
		let _ = Ring::make_free_balance_be(&2, 500);

		let delay = 1000;
		let init_balance_2 = Ring::total_balance(&2);
		let init_balance_10 = Ring::total_balance(&10);
		let init_balance_11 = Ring::total_balance(&11);

		// Set payee to controller.
		assert_ok!(Staking::set_payee(Origin::signed(10), RewardDestination::Controller));

		// Initial config should be correct.
		assert_eq!(Staking::current_era(), 0);
		assert_eq!(Session::current_index(), 0);

		// Add a dummy nominator.
		//
		// Equal division indicates that the reward will be equally divided among validator and
		// nominator.
		<Stakers<Test>>::insert(
			&11,
			Exposure {
				own: 500,
				total: 1000,
				others: vec![IndividualExposure { who: 2, value: 500 }],
			},
		);

		<Payee<Test>>::insert(&2, RewardDestination::Stash);
		assert_eq!(Staking::payee(2), RewardDestination::Stash);
		assert_eq!(Staking::payee(11), RewardDestination::Controller);

		let mut block = 3; // Block 3 => Session 1 => Era 0.
		System::set_block_number(block);
		Timestamp::set_timestamp(block * 5000); // on time.
		Session::on_initialize(System::block_number());
		assert_eq!(Staking::current_era(), 0);
		assert_eq!(Session::current_index(), 1);
		<Module<Test>>::reward_by_ids(vec![(11, 50)]);
		<Module<Test>>::reward_by_ids(vec![(11, 50)]);
		// This is the second validator of the current elected set.
		<Module<Test>>::reward_by_ids(vec![(21, 50)]);
		// This must be no-op as it is not an elected validator.
		<Module<Test>>::reward_by_ids(vec![(1001, 10_000)]);

		// Compute total payout now for whole duration as other parameter won't change.
		//		let total_payout = current_total_payout_for_duration(9 * 5 * 1000);
		//		assert!(total_payout > 10); // Test is meaningful if reward something

		// No reward yet
		assert_eq!(Ring::total_balance(&2), init_balance_2);
		assert_eq!(Ring::total_balance(&10), init_balance_10);
		assert_eq!(Ring::total_balance(&11), init_balance_11);

		block = 6; // Block 6 => Session 2 => Era 0.
		System::set_block_number(block);
		Timestamp::set_timestamp(block * 5000 + delay); // a little late.
		Session::on_initialize(System::block_number());
		assert_eq!(Staking::current_era(), 0);
		assert_eq!(Session::current_index(), 2);

		block = 9; // Block 9 => Session 3 => Era 1.
		System::set_block_number(block);
		Timestamp::set_timestamp(block * 5000); // back to being on time. no delays.
		Session::on_initialize(System::block_number());
		assert_eq!(Staking::current_era(), 1);
		assert_eq!(Session::current_index(), 3);

		// 11 validator has 2/3 of the total rewards and half half for it and its nominator.
		//		assert_eq_error_rate!(Balances::total_balance(&2), init_balance_2 + total_payout / 3, 1);
		//		assert_eq_error_rate!(Balances::total_balance(&10), init_balance_10 + total_payout / 3, 1);
		assert_eq!(Ring::total_balance(&11), init_balance_11);
	});
}

// TODO
#[test]
fn multi_era_reward_should_work() {
	// Should check that:
	// The value of current_session_reward is set at the end of each era, based on
	// slot_stake and session_reward.
	ExtBuilder::default().nominate(false).build().execute_with(|| {
		let init_balance_10 = Ring::total_balance(&10);

		// Set payee to controller.
		assert_ok!(Staking::set_payee(Origin::signed(10), RewardDestination::Controller));

		// Compute now as other parameter won't change
		//		let total_payout_0 = current_total_payout_for_duration(3000);
		//		assert!(total_payout_0 > 10); // Test is meaningfull if reward something
		<Module<Test>>::reward_by_ids(vec![(11, 1)]);

		start_session(0);
		start_session(1);
		start_session(2);
		start_session(3);

		assert_eq!(Staking::current_era(), 1);
		//		assert_eq!(Ring::total_balance(&10), init_balance_10 + total_payout_0);

		start_session(4);

		//		let total_payout_1 = current_total_payout_for_duration(3000);
		//		assert!(total_payout_1 > 10); // Test is meaningfull if reward something
		<Module<Test>>::reward_by_ids(vec![(11, 101)]);

		// New era is triggered here.
		start_session(5);

		// Pay time.
		//		assert_eq!(
		//			Ring::total_balance(&10),
		//			init_balance_10 + total_payout_0 + total_payout_1
		//		);
	});
}

#[test]
fn staking_should_work() {
	// should test:
	// * new validators can be added to the default set
	// * new ones will be chosen per era
	// * either one can unlock the stash and back-down from being a validator via `chill`ing.
	ExtBuilder::default()
		.nominate(false)
		.fair(false) // to give 20 more staked value
		.build()
		.execute_with(|| {
			Timestamp::set_timestamp(1); // Initialize time.

			// remember + compare this along with the test.
			assert_eq_uvec!(validator_controllers(), vec![20, 10]);

			// Put some money in account that we'll use.
			for i in 1..5 { let _ = Ring::make_free_balance_be(&i, 2000); }

			// --- Block 1:
			start_session(1);
			// Add a new candidate for being a validator. account 3 controlled by 4.
			assert_ok!(Staking::bond(Origin::signed(3), 4, StakingBalance::Ring(1500), RewardDestination::Controller, 0));
			assert_ok!(Staking::validate(
				Origin::signed(4),
				ValidatorPrefs {
					node_name: "Darwinia Node".bytes().collect(),
					..Default::default()
				},
			));

			// No effects will be seen so far.
			assert_eq_uvec!(validator_controllers(), vec![20, 10]);

			// --- Block 2:
			start_session(2);

			// No effects will be seen so far. Era has not been yet triggered.
			assert_eq_uvec!(validator_controllers(), vec![20, 10]);


			// --- Block 3: the validators will now be queued.
			start_session(3);
			assert_eq!(Staking::current_era(), 1);

			// --- Block 4: the validators will now be changed.
			start_session(4);

			assert_eq_uvec!(validator_controllers(), vec![20, 4]);
			// --- Block 4: Unstake 4 as a validator, freeing up the balance stashed in 3.
			// 4 will chill.
			Staking::chill(Origin::signed(4)).unwrap();

			// --- Block 5: nothing. 4 is still there.
			start_session(5);
			assert_eq_uvec!(validator_controllers(), vec![20, 4]);

			// --- Block 6: 4 will not be a validator.
			start_session(7);
			assert_eq_uvec!(validator_controllers(), vec![20, 10]);

			// Note: the stashed value of 4 is still lock.
			assert_eq!(
				Staking::ledger(&4).unwrap(),
				StakingLedger {
					stash: 3,
					active_ring: 1500,
					active_deposit_ring: 0,
					active_kton: 0,
					deposit_items: vec![],
					ring_staking_lock: StakingLock {
						staking_amount: 1500,
						unbondings: vec![],
					},
					kton_staking_lock: Default::default(),
				},
			);
			// e.g. It cannot spend more than 500 that it has free from the total 2000.
			assert_noop!(Ring::reserve(&3, 501), "account liquidity restrictions prevent withdrawal");
			assert_ok!(Ring::reserve(&3, 409));
		});
}

#[test]
fn less_than_needed_candidates_works() {
	ExtBuilder::default()
		.minimum_validator_count(1)
		.validator_count(4)
		.nominate(false)
		.num_validators(3)
		.build()
		.execute_with(|| {
			assert_eq!(Staking::validator_count(), 4);
			assert_eq!(Staking::minimum_validator_count(), 1);
			assert_eq_uvec!(validator_controllers(), vec![30, 20, 10]);

			start_era(1);

			// Previous set is selected. NO election algorithm is even executed.
			assert_eq_uvec!(validator_controllers(), vec![30, 20, 10]);

			// But the exposure is updated in a simple way. No external votes exists.
			// This is purely self-vote.
			assert_eq!(Staking::stakers(10).others.len(), 0);
			assert_eq!(Staking::stakers(20).others.len(), 0);
			assert_eq!(Staking::stakers(30).others.len(), 0);
			check_exposure_all();
			check_nominator_all();
		});
}

#[test]
fn no_candidate_emergency_condition() {
	ExtBuilder::default()
		.minimum_validator_count(10)
		.validator_count(15)
		.num_validators(4)
		.validator_pool(true)
		.nominate(false)
		.build()
		.execute_with(|| {
			// Initial validators.
			assert_eq_uvec!(validator_controllers(), vec![10, 20, 30, 40]);

			// Set the minimum validator count.
			<Staking as crate::Store>::MinimumValidatorCount::put(10);
			<Staking as crate::Store>::ValidatorCount::put(15);
			assert_eq!(Staking::validator_count(), 15);

			let _ = Staking::chill(Origin::signed(10));

			// Trigger era.
			System::set_block_number(1);
			Session::on_initialize(System::block_number());

			// Previous ones are elected. chill is invalidates. TODO: #2494
			assert_eq_uvec!(validator_controllers(), vec![10, 20, 30, 40]);
			assert_eq!(Staking::current_elected().len(), 0);
		});
}

// TODO
//#[test]
//fn nominating_and_rewards_should_work() {
//	// PHRAGMEN OUTPUT: running this test with the reference impl gives:
//	//
//	// Sequential Phragmén gives
//	// 10  is elected with stake  2200.0 and score  0.0003333333333333333
//	// 20  is elected with stake  1800.0 and score  0.0005555555555555556
//
//	// 10  has load  0.0003333333333333333 and supported
//	// 10  with stake  1000.0
//	// 20  has load  0.0005555555555555556 and supported
//	// 20  with stake  1000.0
//	// 30  has load  0 and supported
//	// 30  with stake  0
//	// 40  has load  0 and supported
//	// 40  with stake  0
//	// 2  has load  0.0005555555555555556 and supported
//	// 10  with stake  600.0 20  with stake  400.0 30  with stake  0.0
//	// 4  has load  0.0005555555555555556 and supported
//	// 10  with stake  600.0 20  with stake  400.0 40  with stake  0.0
//
//	// Sequential Phragmén with post processing gives
//	// 10  is elected with stake  2000.0 and score  0.0003333333333333333
//	// 20  is elected with stake  2000.0 and score  0.0005555555555555556
//
//	// 10  has load  0.0003333333333333333 and supported
//	// 10  with stake  1000.0
//	// 20  has load  0.0005555555555555556 and supported
//	// 20  with stake  1000.0
//	// 30  has load  0 and supported
//	// 30  with stake  0
//	// 40  has load  0 and supported
//	// 40  with stake  0
//	// 2  has load  0.0005555555555555556 and supported
//	// 10  with stake  400.0 20  with stake  600.0 30  with stake  0
//	// 4  has load  0.0005555555555555556 and supported
//	// 10  with stake  600.0 20  with stake  400.0 40  with stake  0.0
//	ExtBuilder::default()
//		.nominate(false)
//		.validator_pool(true)
//		.build()
//		.execute_with(|| {
//			// initial validators -- everyone is actually even.
//			assert_eq_uvec!(validator_controllers(), vec![40, 30]);
//
//			// Set payee to controller
//			assert_ok!(Staking::set_payee(Origin::signed(10), RewardDestination::Controller));
//			assert_ok!(Staking::set_payee(Origin::signed(20), RewardDestination::Controller));
//			assert_ok!(Staking::set_payee(Origin::signed(30), RewardDestination::Controller));
//			assert_ok!(Staking::set_payee(Origin::signed(40), RewardDestination::Controller));
//
//			// give the man some money
//			let initial_balance = 1000;
//			for i in [1, 2, 3, 4, 5, 10, 11, 20, 21].iter() {
//				let _ = Balances::make_free_balance_be(i, initial_balance);
//			}
//
//			// bond two account pairs and state interest in nomination.
//			// 2 will nominate for 10, 20, 30
//			assert_ok!(Staking::bond(Origin::signed(1), 2, 1000, RewardDestination::Controller));
//			assert_ok!(Staking::nominate(Origin::signed(2), vec![11, 21, 31]));
//			// 4 will nominate for 10, 20, 40
//			assert_ok!(Staking::bond(Origin::signed(3), 4, 1000, RewardDestination::Controller));
//			assert_ok!(Staking::nominate(Origin::signed(4), vec![11, 21, 41]));
//
//			// the total reward for era 0
//			let total_payout_0 = current_total_payout_for_duration(3000);
//			assert!(total_payout_0 > 100); // Test is meaningfull if reward something
//			<Module<Test>>::reward_by_ids(vec![(41, 1)]);
//			<Module<Test>>::reward_by_ids(vec![(31, 1)]);
//			<Module<Test>>::reward_by_ids(vec![(21, 10)]); // must be no-op
//			<Module<Test>>::reward_by_ids(vec![(11, 10)]); // must be no-op
//
//			start_era(1);
//
//			// 10 and 20 have more votes, they will be chosen by phragmen.
//			assert_eq_uvec!(validator_controllers(), vec![20, 10]);
//
//			// OLD validators must have already received some rewards.
//			assert_eq!(Balances::total_balance(&40), 1 + total_payout_0 / 2);
//			assert_eq!(Balances::total_balance(&30), 1 + total_payout_0 / 2);
//
//			// ------ check the staked value of all parties.
//
//			if cfg!(feature = "equalize") {
//				// total expo of 10, with 1200 coming from nominators (externals), according to phragmen.
//				assert_eq!(Staking::stakers(11).own, 1000);
//				assert_eq_error_rate!(Staking::stakers(11).total, 1000 + 1000, 2);
//				// 2 and 4 supported 10, each with stake 600, according to phragmen.
//				assert_eq!(
//					Staking::stakers(11).others.iter().map(|e| e.value).collect::<Vec<BalanceOf<Test>>>(),
//					vec![600, 400]
//				);
//				assert_eq!(
//					Staking::stakers(11).others.iter().map(|e| e.who).collect::<Vec<u64>>(),
//					vec![3, 1]
//				);
//				// total expo of 20, with 500 coming from nominators (externals), according to phragmen.
//				assert_eq!(Staking::stakers(21).own, 1000);
//				assert_eq_error_rate!(Staking::stakers(21).total, 1000 + 1000, 2);
//				// 2 and 4 supported 20, each with stake 250, according to phragmen.
//				assert_eq!(
//					Staking::stakers(21).others.iter().map(|e| e.value).collect::<Vec<BalanceOf<Test>>>(),
//					vec![400, 600]
//				);
//				assert_eq!(
//					Staking::stakers(21).others.iter().map(|e| e.who).collect::<Vec<u64>>(),
//					vec![3, 1]
//				);
//			} else {
//				// total expo of 10, with 1200 coming from nominators (externals), according to phragmen.
//				assert_eq!(Staking::stakers(11).own, 1000);
//				assert_eq!(Staking::stakers(11).total, 1000 + 800);
//				// 2 and 4 supported 10, each with stake 600, according to phragmen.
//				assert_eq!(
//					Staking::stakers(11).others.iter().map(|e| e.value).collect::<Vec<BalanceOf<Test>>>(),
//					vec![400, 400]
//				);
//				assert_eq!(
//					Staking::stakers(11).others.iter().map(|e| e.who).collect::<Vec<u64>>(),
//					vec![3, 1]
//				);
//				// total expo of 20, with 500 coming from nominators (externals), according to phragmen.
//				assert_eq!(Staking::stakers(21).own, 1000);
//				assert_eq_error_rate!(Staking::stakers(21).total, 1000 + 1200, 2);
//				// 2 and 4 supported 20, each with stake 250, according to phragmen.
//				assert_eq!(
//					Staking::stakers(21).others.iter().map(|e| e.value).collect::<Vec<BalanceOf<Test>>>(),
//					vec![600, 600]
//				);
//				assert_eq!(
//					Staking::stakers(21).others.iter().map(|e| e.who).collect::<Vec<u64>>(),
//					vec![3, 1]
//				);
//			}
//
//			// They are not chosen anymore
//			assert_eq!(Staking::stakers(31).total, 0);
//			assert_eq!(Staking::stakers(41).total, 0);
//
//			// the total reward for era 1
//			let total_payout_1 = current_total_payout_for_duration(3000);
//			assert!(total_payout_1 > 100); // Test is meaningfull if reward something
//			<Module<Test>>::reward_by_ids(vec![(41, 10)]); // must be no-op
//			<Module<Test>>::reward_by_ids(vec![(31, 10)]); // must be no-op
//			<Module<Test>>::reward_by_ids(vec![(21, 2)]);
//			<Module<Test>>::reward_by_ids(vec![(11, 1)]);
//
//			start_era(2);
//
//			// nothing else will happen, era ends and rewards are paid again,
//			// it is expected that nominators will also be paid. See below
//
//			let payout_for_10 = total_payout_1 / 3;
//			let payout_for_20 = 2 * total_payout_1 / 3;
//			if cfg!(feature = "equalize") {
//				// Nominator 2: has [400 / 2000 ~ 1 / 5 from 10] + [600 / 2000 ~ 3 / 10 from 20]'s reward.
//				assert_eq_error_rate!(
//					Balances::total_balance(&2),
//					initial_balance + payout_for_10 / 5 + payout_for_20 * 3 / 10,
//					2,
//				);
//				// Nominator 4: has [400 / 2000 ~ 1 / 5 from 20] + [600 / 2000 ~ 3 / 10 from 10]'s reward.
//				assert_eq_error_rate!(
//					Balances::total_balance(&4),
//					initial_balance + payout_for_20 / 5 + payout_for_10 * 3 / 10,
//					2,
//				);
//
//				// Validator 10: got 1000 / 2000 external stake.
//				assert_eq_error_rate!(
//					Balances::total_balance(&10),
//					initial_balance + payout_for_10 / 2,
//					1,
//				);
//				// Validator 20: got 1000 / 2000 external stake.
//				assert_eq_error_rate!(
//					Balances::total_balance(&20),
//					initial_balance + payout_for_20 / 2,
//					1,
//				);
//			} else {
//				// Nominator 2: has [400/1800 ~ 2/9 from 10] + [600/2200 ~ 3/11 from 20]'s reward. ==> 2/9 + 3/11
//				assert_eq_error_rate!(
//					Balances::total_balance(&2),
//					initial_balance + (2 * payout_for_10 / 9 + 3 * payout_for_20 / 11),
//					1,
//				);
//				// Nominator 4: has [400/1800 ~ 2/9 from 10] + [600/2200 ~ 3/11 from 20]'s reward. ==> 2/9 + 3/11
//				assert_eq_error_rate!(
//					Balances::total_balance(&4),
//					initial_balance + (2 * payout_for_10 / 9 + 3 * payout_for_20 / 11),
//					1,
//				);
//
//				// Validator 10: got 800 / 1800 external stake => 8/18 =? 4/9 => Validator's share = 5/9
//				assert_eq_error_rate!(
//					Balances::total_balance(&10),
//					initial_balance + 5 * payout_for_10 / 9,
//					1,
//				);
//				// Validator 20: got 1200 / 2200 external stake => 12/22 =? 6/11 => Validator's share = 5/11
//				assert_eq_error_rate!(
//					Balances::total_balance(&20),
//					initial_balance + 5 * payout_for_20 / 11,
//					1,
//				);
//			}
//
//			check_exposure_all();
//			check_nominator_all();
//		});
//}

// TODO
//#[test]
//fn nominators_also_get_slashed() {
//	// A nominator should be slashed if the validator they nominated is slashed
//	// Here is the breakdown of roles:
//	// 10 - is the controller of 11
//	// 11 - is the stash.
//	// 2 - is the nominator of 20, 10
//	ExtBuilder::default().nominate(false).build().execute_with(|| {
//		assert_eq!(Staking::validator_count(), 2);
//
//		// Set payee to controller
//		assert_ok!(Staking::set_payee(Origin::signed(10), RewardDestination::Controller));
//
//		// give the man some money.
//		let initial_balance = 1000;
//		for i in [1, 2, 3, 10].iter() {
//			let _ = Balances::make_free_balance_be(i, initial_balance);
//		}
//
//		// 2 will nominate for 10, 20
//		let nominator_stake = 500;
//		assert_ok!(Staking::bond(Origin::signed(1), 2, nominator_stake, RewardDestination::default()));
//		assert_ok!(Staking::nominate(Origin::signed(2), vec![20, 10]));
//
//		let total_payout = current_total_payout_for_duration(3000);
//		assert!(total_payout > 100); // Test is meaningfull if reward something
//		<Module<Test>>::reward_by_ids(vec![(11, 1)]);
//
//		// new era, pay rewards,
//		start_era(1);
//
//		// Nominator stash didn't collect any.
//		assert_eq!(Balances::total_balance(&2), initial_balance);
//
//		// 10 goes offline
//		Staking::on_offence(
//			&[OffenceDetails {
//				offender: (
//					11,
//					Staking::stakers(&11),
//				),
//				reporters: vec![],
//			}],
//			&[Perbill::from_percent(5)],
//		);
//		let expo = Staking::stakers(11);
//		let slash_value = 50;
//		let total_slash = expo.total.min(slash_value);
//		let validator_slash = expo.own.min(total_slash);
//		let nominator_slash = nominator_stake.min(total_slash - validator_slash);
//
//		// initial + first era reward + slash
//		assert_eq!(Balances::total_balance(&11), initial_balance - validator_slash);
//		assert_eq!(Balances::total_balance(&2), initial_balance - nominator_slash);
//		check_exposure_all();
//		check_nominator_all();
//		// Because slashing happened.
//		assert!(is_disabled(10));
//	});
//}

#[test]
fn double_staking_should_fail() {
	// should test (in the same order):
	// * an account already bonded as stash cannot be be stashed again.
	// * an account already bonded as stash cannot nominate.
	// * an account already bonded as controller can nominate.
	ExtBuilder::default().build().execute_with(|| {
		let arbitrary_value = 5;
		// 2 = controller, 1 stashed => ok
		assert_ok!(Staking::bond(
			Origin::signed(1),
			2,
			StakingBalance::Ring(arbitrary_value),
			RewardDestination::default(),
			0,
		));
		// 4 = not used so far, 1 stashed => not allowed.
		assert_noop!(
			Staking::bond(
				Origin::signed(1),
				4,
				StakingBalance::Ring(arbitrary_value),
				RewardDestination::default(),
				0,
			),
			err::STASH_ALREADY_BONDED,
		);
		// 1 = stashed => attempting to nominate should fail.
		assert_noop!(Staking::nominate(Origin::signed(1), vec![1]), err::CONTROLLER_INVALID);
		// 2 = controller  => nominating should work.
		assert_ok!(Staking::nominate(Origin::signed(2), vec![1]));
	});
}

#[test]
fn double_controlling_should_fail() {
	// should test (in the same order):
	// * an account already bonded as controller CANNOT be reused as the controller of another account.
	ExtBuilder::default().build().execute_with(|| {
		let arbitrary_value = 5;
		// 2 = controller, 1 stashed => ok
		assert_ok!(Staking::bond(
			Origin::signed(1),
			2,
			StakingBalance::Ring(arbitrary_value),
			RewardDestination::default(),
			0,
		));
		// 2 = controller, 3 stashed (Note that 2 is reused.) => no-op
		assert_noop!(
			Staking::bond(
				Origin::signed(3),
				2,
				StakingBalance::Ring(arbitrary_value),
				RewardDestination::default(),
				0,
			),
			err::CONTROLLER_ALREADY_PAIRED,
		);
	});
}

#[test]
fn session_and_eras_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Staking::current_era(), 0);

		// Block 1: No change.
		start_session(0);
		assert_eq!(Session::current_index(), 1);
		assert_eq!(Staking::current_era(), 0);

		// Block 2: Simple era change.
		start_session(2);
		assert_eq!(Session::current_index(), 3);
		assert_eq!(Staking::current_era(), 1);

		// Block 3: Schedule an era length change; no visible changes.
		start_session(3);
		assert_eq!(Session::current_index(), 4);
		assert_eq!(Staking::current_era(), 1);

		// Block 4: Era change kicks in.
		start_session(5);
		assert_eq!(Session::current_index(), 6);
		assert_eq!(Staking::current_era(), 2);

		// Block 5: No change.
		start_session(6);
		assert_eq!(Session::current_index(), 7);
		assert_eq!(Staking::current_era(), 2);

		// Block 6: No change.
		start_session(7);
		assert_eq!(Session::current_index(), 8);
		assert_eq!(Staking::current_era(), 2);

		// Block 7: Era increment.
		start_session(8);
		assert_eq!(Session::current_index(), 9);
		assert_eq!(Staking::current_era(), 3);
	});
}

#[test]
fn forcing_new_era_works() {
	ExtBuilder::default().build().execute_with(|| {
		// normal flow of session.
		assert_eq!(Staking::current_era(), 0);
		start_session(0);
		assert_eq!(Staking::current_era(), 0);
		start_session(1);
		assert_eq!(Staking::current_era(), 0);
		start_session(2);
		assert_eq!(Staking::current_era(), 1);

		// no era change.
		ForceEra::put(Forcing::ForceNone);
		start_session(3);
		assert_eq!(Staking::current_era(), 1);
		start_session(4);
		assert_eq!(Staking::current_era(), 1);
		start_session(5);
		assert_eq!(Staking::current_era(), 1);
		start_session(6);
		assert_eq!(Staking::current_era(), 1);

		// back to normal.
		// this immediately starts a new session.
		ForceEra::put(Forcing::NotForcing);
		start_session(7);
		assert_eq!(Staking::current_era(), 2);
		start_session(8);
		assert_eq!(Staking::current_era(), 2);

		// forceful change
		ForceEra::put(Forcing::ForceAlways);
		start_session(9);
		assert_eq!(Staking::current_era(), 3);
		start_session(10);
		assert_eq!(Staking::current_era(), 4);
		start_session(11);
		assert_eq!(Staking::current_era(), 5);

		// just one forceful change
		ForceEra::put(Forcing::ForceNew);
		start_session(12);
		assert_eq!(Staking::current_era(), 6);

		assert_eq!(ForceEra::get(), Forcing::NotForcing);
		start_session(13);
		assert_eq!(Staking::current_era(), 6);
	});
}

#[test]
fn cannot_transfer_staked_balance() {
	// Tests that a stash account cannot transfer funds
	ExtBuilder::default().nominate(false).build().execute_with(|| {
		// Confirm account 11 is stashed
		assert_eq!(Staking::bonded(&11), Some(10));
		// Confirm account 11 has some free balance
		assert_eq!(Ring::free_balance(&11), 1000);
		// Confirm account 11 (via controller 10) is totally staked
		assert_eq!(Staking::stakers(&11).total, Staking::power_of(&11));
		// Confirm account 11 cannot transfer as a result
		assert_noop!(
			Ring::transfer(Origin::signed(11), 20, 1),
			"account liquidity restrictions prevent withdrawal",
		);

		// Give account 11 extra free balance
		let _ = Ring::make_free_balance_be(&11, 10000);
		// Confirm that account 11 can now transfer some balance
		assert_ok!(Ring::transfer(Origin::signed(11), 20, 1));
	});
}

#[test]
fn cannot_transfer_staked_balance_2() {
	// Tests that a stash account cannot transfer funds
	// Same test as above but with 20, and more accurate.
	// 21 has 2000 free balance but 1000 at stake
	ExtBuilder::default()
		.nominate(false)
		.fair(true)
		.build()
		.execute_with(|| {
			// Confirm account 21 is stashed
			assert_eq!(Staking::bonded(&21), Some(20));
			// Confirm account 21 has some free balance
			assert_eq!(Ring::free_balance(&21), 2000);
			// Confirm account 21 (via controller 20) is totally staked
			assert_eq!(Staking::stakers(&21).total, Staking::power_of(&11));
			// Confirm account 21 can transfer at most 1000
			assert_noop!(
				Ring::transfer(Origin::signed(21), 20, 1001),
				"account liquidity restrictions prevent withdrawal",
			);
			assert_ok!(Ring::transfer(Origin::signed(21), 20, 1000));
		});
}

#[test]
fn cannot_reserve_staked_balance() {
	// Checks that a bonded account cannot reserve balance from free balance
	ExtBuilder::default().build().execute_with(|| {
		// Confirm account 11 is stashed
		assert_eq!(Staking::bonded(&11), Some(10));
		// Confirm account 11 has some free balance
		assert_eq!(Ring::free_balance(&11), 1000);
		// Confirm account 11 (via controller 10) is totally staked
		assert_eq!(Staking::stakers(&11).own, Staking::power_of(&11));
		// Confirm account 11 cannot transfer as a result
		assert_noop!(
			Ring::reserve(&11, 1),
			"account liquidity restrictions prevent withdrawal"
		);

		// Give account 11 extra free balance
		let _ = Ring::make_free_balance_be(&11, 10000);
		// Confirm account 11 can now reserve balance
		assert_ok!(Ring::reserve(&11, 1));
	});
}

// TODO
//#[test]
//fn reward_destination_works() {
//	// Rewards go to the correct destination as determined in Payee
//	ExtBuilder::default().nominate(false).build().execute_with(|| {
//		// Check that account 11 is a validator
//		assert!(Staking::current_elected().contains(&11));
//		// Check the balance of the validator account
//		assert_eq!(Balances::free_balance(&10), 1);
//		// Check the balance of the stash account
//		assert_eq!(Balances::free_balance(&11), 1000);
//		// Check how much is at stake
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedger {
//				stash: 11,
//				total: 1000,
//				active: 1000,
//				unlocking: vec![],
//			})
//		);
//
//		// Compute total payout now for whole duration as other parameter won't change
//		let total_payout_0 = current_total_payout_for_duration(3000);
//		assert!(total_payout_0 > 100); // Test is meaningfull if reward something
//		<Module<Test>>::reward_by_ids(vec![(11, 1)]);
//
//		start_era(1);
//
//		// Check that RewardDestination is Staked (default)
//		assert_eq!(Staking::payee(&11), RewardDestination::Staked);
//		// Check that reward went to the stash account of validator
//		assert_eq!(Balances::free_balance(&11), 1000 + total_payout_0);
//		// Check that amount at stake increased accordingly
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedger {
//				stash: 11,
//				total: 1000 + total_payout_0,
//				active: 1000 + total_payout_0,
//				unlocking: vec![],
//			})
//		);
//
//		//Change RewardDestination to Stash
//		<Payee<Test>>::insert(&11, RewardDestination::Stash);
//
//		// Compute total payout now for whole duration as other parameter won't change
//		let total_payout_1 = current_total_payout_for_duration(3000);
//		assert!(total_payout_1 > 100); // Test is meaningfull if reward something
//		<Module<Test>>::reward_by_ids(vec![(11, 1)]);
//
//		start_era(2);
//
//		// Check that RewardDestination is Stash
//		assert_eq!(Staking::payee(&11), RewardDestination::Stash);
//		// Check that reward went to the stash account
//		assert_eq!(Balances::free_balance(&11), 1000 + total_payout_0 + total_payout_1);
//		// Record this value
//		let recorded_stash_balance = 1000 + total_payout_0 + total_payout_1;
//		// Check that amount at stake is NOT increased
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedger {
//				stash: 11,
//				total: 1000 + total_payout_0,
//				active: 1000 + total_payout_0,
//				unlocking: vec![],
//			})
//		);
//
//		// Change RewardDestination to Controller
//		<Payee<Test>>::insert(&11, RewardDestination::Controller);
//
//		// Check controller balance
//		assert_eq!(Balances::free_balance(&10), 1);
//
//		// Compute total payout now for whole duration as other parameter won't change
//		let total_payout_2 = current_total_payout_for_duration(3000);
//		assert!(total_payout_2 > 100); // Test is meaningfull if reward something
//		<Module<Test>>::reward_by_ids(vec![(11, 1)]);
//
//		start_era(3);
//
//		// Check that RewardDestination is Controller
//		assert_eq!(Staking::payee(&11), RewardDestination::Controller);
//		// Check that reward went to the controller account
//		assert_eq!(Balances::free_balance(&10), 1 + total_payout_2);
//		// Check that amount at stake is NOT increased
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedger {
//				stash: 11,
//				total: 1000 + total_payout_0,
//				active: 1000 + total_payout_0,
//				unlocking: vec![],
//			})
//		);
//		// Check that amount in staked account is NOT increased.
//		assert_eq!(Balances::free_balance(&11), recorded_stash_balance);
//	});
//}

// TODO
//#[test]
//fn validator_payment_prefs_work() {
//	// Test that validator preferences are correctly honored
//	// Note: unstake threshold is being directly tested in slashing tests.
//	// This test will focus on validator payment.
//	ExtBuilder::default().build().execute_with(|| {
//		// Initial config
//		let validator_cut = 5;
//		let stash_initial_balance = Balances::total_balance(&11);
//
//		// check the balance of a validator accounts.
//		assert_eq!(Balances::total_balance(&10), 1);
//		// check the balance of a validator's stash accounts.
//		assert_eq!(Balances::total_balance(&11), stash_initial_balance);
//		// and the nominator (to-be)
//		let _ = Balances::make_free_balance_be(&2, 500);
//
//		// add a dummy nominator.
//		<Stakers<Test>>::insert(
//			&11,
//			Exposure {
//				own: 500, // equal division indicates that the reward will be equally divided among validator and nominator.
//				total: 1000,
//				others: vec![IndividualExposure { who: 2, value: 500 }],
//			},
//		);
//		<Payee<Test>>::insert(&2, RewardDestination::Stash);
//		<Validators<Test>>::insert(
//			&11,
//			ValidatorPrefs {
//				validator_payment: validator_cut,
//			},
//		);
//
//		// Compute total payout now for whole duration as other parameter won't change
//		let total_payout_0 = current_total_payout_for_duration(3000);
//		assert!(total_payout_0 > 100); // Test is meaningfull if reward something
//		<Module<Test>>::reward_by_ids(vec![(11, 1)]);
//
//		start_era(1);
//
//		// whats left to be shared is the sum of 3 rounds minus the validator's cut.
//		let shared_cut = total_payout_0 - validator_cut;
//		// Validator's payee is Staked account, 11, reward will be paid here.
//		assert_eq!(
//			Balances::total_balance(&11),
//			stash_initial_balance + shared_cut / 2 + validator_cut
//		);
//		// Controller account will not get any reward.
//		assert_eq!(Balances::total_balance(&10), 1);
//		// Rest of the reward will be shared and paid to the nominator in stake.
//		assert_eq!(Balances::total_balance(&2), 500 + shared_cut / 2);
//
//		check_exposure_all();
//		check_nominator_all();
//	});
//}

#[test]
fn bond_extra_works() {
	// Tests that extra `free_balance` in the stash can be added to stake
	// NOTE: this tests only verifies `StakingLedger` for correct updates
	// See `bond_extra_and_withdraw_unbonded_works` for more details and updates on `Exposure`.
	ExtBuilder::default().build().execute_with(|| {
		// Check that account 10 is a validator
		assert!(<Validators<Test>>::exists(11));
		// Check that account 10 is bonded to account 11
		assert_eq!(Staking::bonded(&11), Some(10));
		// Check how much is at stake
		assert_eq!(
			Staking::ledger(&10).unwrap(),
			StakingLedger {
				stash: 11,
				active_ring: 1000,
				active_deposit_ring: 0,
				active_kton: 0,
				deposit_items: vec![],
				ring_staking_lock: StakingLock {
					staking_amount: 1000,
					unbondings: vec![],
				},
				kton_staking_lock: Default::default(),
			},
		);

		// Give account 11 some large free balance greater than total
		let _ = Ring::make_free_balance_be(&11, 1000000);

		// Call the bond_extra function from controller, add only 100
		assert_ok!(Staking::bond_extra(Origin::signed(11), StakingBalance::Ring(100), 12));
		// There should be 100 more `total` and `active` in the ledger
		assert_eq!(
			Staking::ledger(&10).unwrap(),
			StakingLedger {
				stash: 11,
				active_ring: 1000 + 100,
				active_deposit_ring: 100,
				active_kton: 0,
				deposit_items: vec![TimeDepositItem {
					value: 100,
					start_time: 0,
					expire_time: 31104000000,
				}],
				ring_staking_lock: StakingLock {
					staking_amount: 1000 + 100,
					unbondings: vec![],
				},
				kton_staking_lock: Default::default(),
			},
		);

		// Call the bond_extra function with a large number, should handle it
		assert_ok!(Staking::bond_extra(
			Origin::signed(11),
			StakingBalance::Ring(Balance::max_value()),
			0,
		));
		// The full amount of the funds should now be in the total and active
		assert_eq!(
			Staking::ledger(&10).unwrap(),
			StakingLedger {
				stash: 11,
				active_ring: 1000000,
				active_deposit_ring: 100,
				active_kton: 0,
				deposit_items: vec![TimeDepositItem {
					value: 100,
					start_time: 0,
					expire_time: 31104000000,
				}],
				ring_staking_lock: StakingLock {
					staking_amount: 1000000,
					unbondings: vec![],
				},
				kton_staking_lock: Default::default(),
			},
		);
	});
}

// TODO
//#[test]
//fn bond_extra_and_withdraw_unbonded_automatically_works() {
//	// * Should test
//	// * Given an account being bonded [and chosen as a validator](not mandatory)
//	// * It can add extra funds to the bonded account.
//	// * it can unbond a portion of its funds from the stash account.
//	// * Once the unbonding period is done, it can actually take the funds out of the stash.
//	ExtBuilder::default().nominate(false).build().execute_with(|| {
//		// Set payee to controller. avoids confusion
//		assert_ok!(Staking::set_payee(Origin::signed(10), RewardDestination::Controller));
//
//		// Give account 11 some large free balance greater than total
//		let _ = Balances::make_free_balance_be(&11, 1000000);
//
//		// Initial config should be correct
//		assert_eq!(Staking::current_era(), 0);
//		assert_eq!(Session::current_index(), 0);
//
//		// check the balance of a validator accounts.
//		assert_eq!(Balances::total_balance(&10), 1);
//
//		// confirm that 10 is a normal validator and gets paid at the end of the era.
//		start_era(1);
//
//		// Initial state of 10
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedger {
//				stash: 11,
//				total: 1000,
//				active: 1000,
//				unlocking: vec![],
//			})
//		);
//		assert_eq!(
//			Staking::stakers(&11),
//			Exposure {
//				total: 1000,
//				own: 1000,
//				others: vec![]
//			}
//		);
//
//		// deposit the extra 100 units
//		Staking::bond_extra(Origin::signed(11), 100).unwrap();
//
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedger {
//				stash: 11,
//				total: 1000 + 100,
//				active: 1000 + 100,
//				unlocking: vec![],
//			})
//		);
//		// Exposure is a snapshot! only updated after the next era update.
//		assert_ne!(
//			Staking::stakers(&11),
//			Exposure {
//				total: 1000 + 100,
//				own: 1000 + 100,
//				others: vec![]
//			}
//		);
//
//		// trigger next era.
//		Timestamp::set_timestamp(10);
//		start_era(2);
//		assert_eq!(Staking::current_era(), 2);
//
//		// ledger should be the same.
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedger {
//				stash: 11,
//				total: 1000 + 100,
//				active: 1000 + 100,
//				unlocking: vec![],
//			})
//		);
//		// Exposure is now updated.
//		assert_eq!(
//			Staking::stakers(&11),
//			Exposure {
//				total: 1000 + 100,
//				own: 1000 + 100,
//				others: vec![]
//			}
//		);
//
//		// Unbond almost all of the funds in stash.
//		Staking::unbond(Origin::signed(10), 1000).unwrap();
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedger {
//				stash: 11,
//				total: 1000 + 100,
//				active: 100,
//				unlocking: vec![UnlockChunk {
//					value: 1000,
//					era: 2 + 3
//				}]
//			})
//		);
//
//		// Attempting to free the balances now will fail. 2 eras need to pass.
//		Staking::withdraw_unbonded(Origin::signed(10)).unwrap();
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedger {
//				stash: 11,
//				total: 1000 + 100,
//				active: 100,
//				unlocking: vec![UnlockChunk {
//					value: 1000,
//					era: 2 + 3
//				}]
//			})
//		);
//
//		// trigger next era.
//		start_era(3);
//
//		// nothing yet
//		Staking::withdraw_unbonded(Origin::signed(10)).unwrap();
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedger {
//				stash: 11,
//				total: 1000 + 100,
//				active: 100,
//				unlocking: vec![UnlockChunk {
//					value: 1000,
//					era: 2 + 3
//				}]
//			})
//		);
//
//		// trigger next era.
//		start_era(5);
//
//		Staking::withdraw_unbonded(Origin::signed(10)).unwrap();
//		// Now the value is free and the staking ledger is updated.
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedger {
//				stash: 11,
//				total: 100,
//				active: 100,
//				unlocking: vec![]
//			})
//		);
//	})
//}

#[test]
fn too_many_unbond_calls_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		// Locked at Moment(60).
		for _ in 0..MAX_UNLOCKING_CHUNKS - 1 {
			assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Ring(1)));
		}

		Timestamp::set_timestamp(1);

		// Locked at MomentT(61).
		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Ring(1)));

		// Can't do more.
		assert_noop!(
			Staking::unbond(Origin::signed(10), StakingBalance::Ring(1)),
			err::UNLOCK_CHUNKS_REACH_MAX,
		);

		// Free up automatically.
		Timestamp::set_timestamp(BondingDuration::get());

		// Can add again.
		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Ring(1)));
		assert_eq!(Staking::ledger(&10).unwrap().ring_staking_lock.unbondings.len(), 2);
	})
}

// TODO
//#[test]
//fn slot_stake_is_least_staked_validator_and_exposure_defines_maximum_punishment() {
//	// Test that slot_stake is determined by the least staked validator
//	// Test that slot_stake is the maximum punishment that can happen to a validator
//	ExtBuilder::default()
//		.nominate(false)
//		.fair(false)
//		.build()
//		.execute_with(|| {
//			// Confirm validator count is 2
//			assert_eq!(Staking::validator_count(), 2);
//			// Confirm account 10 and 20 are validators
//			assert!(<Validators<Test>>::exists(&11) && <Validators<Test>>::exists(&21));
//
//			assert_eq!(Staking::stakers(&11).total, 1000);
//			assert_eq!(Staking::stakers(&21).total, 2000);
//
//			// Give the man some money.
//			let _ = Balances::make_free_balance_be(&10, 1000);
//			let _ = Balances::make_free_balance_be(&20, 1000);
//
//			// We confirm initialized slot_stake is this value
//			assert_eq!(Staking::slot_stake(), Staking::stakers(&11).total);
//
//			// Now lets lower account 20 stake
//			<Stakers<Test>>::insert(
//				&21,
//				Exposure {
//					total: 69,
//					own: 69,
//					others: vec![],
//				},
//			);
//			assert_eq!(Staking::stakers(&21).total, 69);
//			<Ledger<Test>>::insert(
//				&20,
//				StakingLedger {
//					stash: 22,
//					total: 69,
//					active: 69,
//					unlocking: vec![],
//				},
//			);
//
//			// Compute total payout now for whole duration as other parameter won't change
//			let total_payout_0 = current_total_payout_for_duration(3000);
//			assert!(total_payout_0 > 100); // Test is meaningfull if reward something
//			<Module<Test>>::reward_by_ids(vec![(11, 1)]);
//			<Module<Test>>::reward_by_ids(vec![(21, 1)]);
//
//			// New era --> rewards are paid --> stakes are changed
//			start_era(1);
//
//			// -- new balances + reward
//			assert_eq!(Staking::stakers(&11).total, 1000 + total_payout_0 / 2);
//			assert_eq!(Staking::stakers(&21).total, 69 + total_payout_0 / 2);
//
//			let _11_balance = Balances::free_balance(&11);
//			assert_eq!(_11_balance, 1000 + total_payout_0 / 2);
//
//			// -- slot stake should also be updated.
//			assert_eq!(Staking::slot_stake(), 69 + total_payout_0 / 2);
//
//			check_exposure_all();
//			check_nominator_all();
//		});
//}

#[test]
fn on_free_balance_zero_stash_removes_validator() {
	// Tests that validator storage items are cleaned up when stash is empty
	// Tests that storage items are untouched when controller is empty
	ExtBuilder::default().existential_deposit(10).build().execute_with(|| {
		// Check the balance of the validator account
		assert_eq!(Ring::free_balance(&10), 256);
		// Check the balance of the stash account
		assert_eq!(Ring::free_balance(&11), 256000);
		// Check these two accounts are bonded
		assert_eq!(Staking::bonded(&11), Some(10));

		// Set some storage items which we expect to be cleaned up
		// Set payee information
		assert_ok!(Staking::set_payee(Origin::signed(10), RewardDestination::Stash));

		// Check storage items that should be cleaned up
		assert!(<Ledger<Test>>::exists(&10));
		assert!(<Bonded<Test>>::exists(&11));
		assert!(<Validators<Test>>::exists(&11));
		assert!(<Payee<Test>>::exists(&11));

		// Reduce free_balance of controller to 0
		let _ = Ring::slash(&10, Balance::max_value());

		// Check the balance of the stash account has not been touched
		assert_eq!(Ring::free_balance(&11), 256000);
		// Check these two accounts are still bonded
		assert_eq!(Staking::bonded(&11), Some(10));

		// Check storage items have not changed
		assert!(<Ledger<Test>>::exists(&10));
		assert!(<Bonded<Test>>::exists(&11));
		assert!(<Validators<Test>>::exists(&11));
		assert!(<Payee<Test>>::exists(&11));

		// Reduce free_balance of stash to 0
		let _ = Ring::slash(&11, Balance::max_value());
		// Check total balance of stash
		assert_eq!(Ring::total_balance(&11), 0);

		// Check storage items do not exist
		assert!(!<Ledger<Test>>::exists(&10));
		assert!(!<Bonded<Test>>::exists(&11));
		assert!(!<Validators<Test>>::exists(&11));
		assert!(!<Nominators<Test>>::exists(&11));
		assert!(!<Payee<Test>>::exists(&11));
	});
}

#[test]
fn on_free_balance_zero_stash_removes_nominator() {
	// Tests that nominator storage items are cleaned up when stash is empty
	// Tests that storage items are untouched when controller is empty
	ExtBuilder::default().existential_deposit(10).build().execute_with(|| {
		// Make 10 a nominator
		assert_ok!(Staking::nominate(Origin::signed(10), vec![20]));
		// Check that account 10 is a nominator
		assert!(<Nominators<Test>>::exists(11));
		// Check the balance of the nominator account
		assert_eq!(Ring::free_balance(&10), 256);
		// Check the balance of the stash account
		assert_eq!(Ring::free_balance(&11), 256000);

		// Set payee information
		assert_ok!(Staking::set_payee(Origin::signed(10), RewardDestination::Stash));

		// Check storage items that should be cleaned up
		assert!(<Ledger<Test>>::exists(&10));
		assert!(<Bonded<Test>>::exists(&11));
		assert!(<Nominators<Test>>::exists(&11));
		assert!(<Payee<Test>>::exists(&11));

		// Reduce free_balance of controller to 0
		let _ = Ring::slash(&10, Balance::max_value());
		// Check total balance of account 10
		assert_eq!(Ring::total_balance(&10), 0);

		// Check the balance of the stash account has not been touched
		assert_eq!(Ring::free_balance(&11), 256000);
		// Check these two accounts are still bonded
		assert_eq!(Staking::bonded(&11), Some(10));

		// Check storage items have not changed
		assert!(<Ledger<Test>>::exists(&10));
		assert!(<Bonded<Test>>::exists(&11));
		assert!(<Nominators<Test>>::exists(&11));
		assert!(<Payee<Test>>::exists(&11));

		// Reduce free_balance of stash to 0
		let _ = Ring::slash(&11, Balance::max_value());
		// Check total balance of stash
		assert_eq!(Ring::total_balance(&11), 0);

		// Check storage items do not exist
		assert!(!<Ledger<Test>>::exists(&10));
		assert!(!<Bonded<Test>>::exists(&11));
		assert!(!<Validators<Test>>::exists(&11));
		assert!(!<Nominators<Test>>::exists(&11));
		assert!(!<Payee<Test>>::exists(&11));
	});
}

#[test]
fn switching_roles() {
	// Test that it should be possible to switch between roles (nominator, validator, idle) with minimal overhead.
	ExtBuilder::default().nominate(false).build().execute_with(|| {
		// Initialize time.
		Timestamp::set_timestamp(1);

		// Reset reward destination.
		for i in &[10, 20] {
			assert_ok!(Staking::set_payee(Origin::signed(*i), RewardDestination::Controller));
		}

		assert_eq_uvec!(validator_controllers(), vec![20, 10]);

		// Put some money in account that we'll use.
		for i in 1..7 {
			let _ = Ring::deposit_creating(&i, 5000);
		}

		// Add 2 nominators.
		assert_ok!(Staking::bond(
			Origin::signed(1),
			2,
			StakingBalance::Ring(2000),
			RewardDestination::Controller,
			0,
		));
		assert_ok!(Staking::nominate(Origin::signed(2), vec![11, 5]));

		assert_ok!(Staking::bond(
			Origin::signed(3),
			4,
			StakingBalance::Ring(500),
			RewardDestination::Controller,
			0,
		));
		assert_ok!(Staking::nominate(Origin::signed(4), vec![21, 1]));

		// Add a new validator candidate.
		assert_ok!(Staking::bond(
			Origin::signed(5),
			6,
			StakingBalance::Ring(1000),
			RewardDestination::Controller,
			0,
		));
		assert_ok!(Staking::validate(
			Origin::signed(6),
			ValidatorPrefs {
				node_name: "Darwinia Node".bytes().collect(),
				..Default::default()
			},
		));

		// New block.
		start_session(1);

		// No change.
		assert_eq_uvec!(validator_controllers(), vec![20, 10]);

		// New block.
		start_session(2);

		// No change.
		assert_eq_uvec!(validator_controllers(), vec![20, 10]);

		// new block --> ne era --> new validators.
		start_session(3);

		// With current nominators 10 and 5 have the most stake.
		assert_eq_uvec!(validator_controllers(), vec![6, 10]);

		// 2 decides to be a validator. Consequences:
		assert_ok!(Staking::validate(
			Origin::signed(2),
			ValidatorPrefs {
				node_name: "Darwinia Node".bytes().collect(),
				..Default::default()
			},
		));
		// New stakes:
		// 10: 1000 self vote
		// 20: 1000 self vote + 250 vote
		// 6 : 1000 self vote
		// 2 : 2000 self vote + 250 vote.
		// Winners: 20 and 2

		start_session(4);
		assert_eq_uvec!(validator_controllers(), vec![6, 10]);

		start_session(5);
		assert_eq_uvec!(validator_controllers(), vec![6, 10]);

		// ne era.
		start_session(6);
		assert_eq_uvec!(validator_controllers(), vec![2, 20]);

		check_exposure_all();
		check_nominator_all();
	});
}

#[test]
fn wrong_vote_is_null() {
	ExtBuilder::default()
		.nominate(false)
		.validator_pool(true)
		.build()
		.execute_with(|| {
			assert_eq_uvec!(validator_controllers(), vec![40, 30]);

			// Put some money in account that we'll use.
			for i in 1..3 {
				let _ = Ring::deposit_creating(&i, 5000);
			}

			// Add 1 nominators
			assert_ok!(Staking::bond(
				Origin::signed(1),
				2,
				StakingBalance::Ring(2000),
				RewardDestination::default(),
				0,
			));
			assert_ok!(Staking::nominate(
				Origin::signed(2),
				vec![
					11, 21, // Good votes.
					1, 2, 15, 1000, 25 // Crap votes. No effect.
				],
			));

			// New block.
			start_era(1);

			assert_eq_uvec!(validator_controllers(), vec![20, 10]);
		});
}

#[test]
fn bond_with_no_staked_value() {
	// Behavior when someone bonds with no staked value.
	// Particularly when she votes and the candidate is elected.
	ExtBuilder::default()
		.validator_count(3)
		.existential_deposit(5)
		.nominate(false)
		.minimum_validator_count(1)
		.build()
		.execute_with(|| {
			// Bonded with absolute minimum value possible.
			assert_ok!(Staking::bond(
				Origin::signed(1),
				2,
				StakingBalance::Ring(5),
				RewardDestination::Controller,
				0,
			));
			//			assert_eq!(Ring::locks(&1)[0].amount, 5);

			assert_ok!(Staking::unbond(Origin::signed(2), StakingBalance::Ring(5)));
			assert_eq!(
				Staking::ledger(2),
				Some(StakingLedger {
					stash: 1,
					active_ring: 0,
					active_deposit_ring: 0,
					active_kton: 0,
					deposit_items: vec![],
					ring_staking_lock: StakingLock {
						staking_amount: 0,
						unbondings: vec![NormalLock { amount: 5, until: 60 }],
					},
					kton_staking_lock: Default::default(),
				}),
			);

			Timestamp::set_timestamp(BondingDuration::get() - 1);

			// Not yet removed.
			assert!(Staking::ledger(2).is_some());
			//			assert_eq!(Ring::locks(&1)[0].amount, 5);

			Timestamp::set_timestamp(BondingDuration::get());

			// FIXME
			// Poof. Account 1 is removed from the staking system.
			//			assert!(Staking::ledger(2).is_none());
			//			assert_eq!(Ring::locks(&1).len(), 0);
		});
}

// TODO
//#[test]
//fn bond_with_little_staked_value_bounded_by_slot_stake() {
//	// Behavior when someone bonds with little staked value.
//	// Particularly when she votes and the candidate is elected.
//	ExtBuilder::default()
//		.validator_count(3)
//		.nominate(false)
//		.minimum_validator_count(1)
//		.build()
//		.execute_with(|| {
//			// setup
//			assert_ok!(Staking::chill(Origin::signed(30)));
//			assert_ok!(Staking::set_payee(Origin::signed(10), RewardDestination::Controller));
//			let init_balance_2 = Balances::free_balance(&2);
//			let init_balance_10 = Balances::free_balance(&10);
//
//			// Stingy validator.
//			assert_ok!(Staking::bond(Origin::signed(1), 2, 1, RewardDestination::Controller));
//			assert_ok!(Staking::validate(Origin::signed(2), ValidatorPrefs::default()));
//
//			let total_payout_0 = current_total_payout_for_duration(3000);
//			assert!(total_payout_0 > 100); // Test is meaningfull if reward something
//			reward_all_elected();
//			start_era(1);
//
//			// 2 is elected.
//			// and fucks up the slot stake.
//			assert_eq_uvec!(validator_controllers(), vec![20, 10, 2]);
//			assert_eq!(Staking::slot_stake(), 1);
//
//			// Old ones are rewarded.
//			assert_eq!(Balances::free_balance(&10), init_balance_10 + total_payout_0 / 3);
//			// no rewards paid to 2. This was initial election.
//			assert_eq!(Balances::free_balance(&2), init_balance_2);
//
//			let total_payout_1 = current_total_payout_for_duration(3000);
//			assert!(total_payout_1 > 100); // Test is meaningfull if reward something
//			reward_all_elected();
//			start_era(2);
//
//			assert_eq_uvec!(validator_controllers(), vec![20, 10, 2]);
//			assert_eq!(Staking::slot_stake(), 1);
//
//			assert_eq!(Balances::free_balance(&2), init_balance_2 + total_payout_1 / 3);
//			assert_eq!(
//				Balances::free_balance(&10),
//				init_balance_10 + total_payout_0 / 3 + total_payout_1 / 3,
//			);
//			check_exposure_all();
//			check_nominator_all();
//		});
//}

// TODO
//#[cfg(feature = "equalize")]
//#[test]
//fn phragmen_linear_worse_case_equalize() {
//	ExtBuilder::default()
//		.nominate(false)
//		.validator_pool(true)
//		.fair(true)
//		.build()
//		.execute_with(|| {
//			bond_validator(50, 1000);
//			bond_validator(60, 1000);
//			bond_validator(70, 1000);
//
//			bond_nominator(2, 2000, vec![11]);
//			bond_nominator(4, 1000, vec![11, 21]);
//			bond_nominator(6, 1000, vec![21, 31]);
//			bond_nominator(8, 1000, vec![31, 41]);
//			bond_nominator(110, 1000, vec![41, 51]);
//			bond_nominator(120, 1000, vec![51, 61]);
//			bond_nominator(130, 1000, vec![61, 71]);
//
//			for i in &[10, 20, 30, 40, 50, 60, 70] {
//				assert_ok!(Staking::set_payee(Origin::signed(*i), RewardDestination::Controller));
//			}
//
//			assert_eq_uvec!(validator_controllers(), vec![40, 30]);
//			assert_ok!(Staking::set_validator_count(Origin::ROOT, 7));
//
//			start_era(1);
//
//			assert_eq_uvec!(validator_controllers(), vec![10, 60, 40, 20, 50, 30, 70]);
//
//			assert_eq_error_rate!(Staking::stakers(11).total, 3000, 2);
//			assert_eq_error_rate!(Staking::stakers(21).total, 2255, 2);
//			assert_eq_error_rate!(Staking::stakers(31).total, 2255, 2);
//			assert_eq_error_rate!(Staking::stakers(41).total, 1925, 2);
//			assert_eq_error_rate!(Staking::stakers(51).total, 1870, 2);
//			assert_eq_error_rate!(Staking::stakers(61).total, 1890, 2);
//			assert_eq_error_rate!(Staking::stakers(71).total, 1800, 2);
//
//			check_exposure_all();
//			check_nominator_all();
//		})
//}

#[test]
fn new_era_elects_correct_number_of_validators() {
	ExtBuilder::default()
		.nominate(true)
		.validator_pool(true)
		.fair(true)
		.validator_count(1)
		.build()
		.execute_with(|| {
			assert_eq!(Staking::validator_count(), 1);
			assert_eq!(validator_controllers().len(), 1);

			System::set_block_number(1);
			Session::on_initialize(System::block_number());

			assert_eq!(validator_controllers().len(), 1);
			check_exposure_all();
			check_nominator_all();
		})
}

#[test]
fn reward_from_authorship_event_handler_works() {
	ExtBuilder::default().build().execute_with(|| {
		use authorship::EventHandler;

		assert_eq!(<authorship::Module<Test>>::author(), 11);

		<Module<Test>>::note_author(11);
		<Module<Test>>::note_uncle(21, 1);
		// An uncle author that is not currently elected doesn't get rewards,
		// but the block producer does get reward for referencing it.
		<Module<Test>>::note_uncle(31, 1);
		// Rewarding the same two times works.
		<Module<Test>>::note_uncle(11, 1);

		// Not mandatory but must be coherent with rewards.
		assert_eq!(<CurrentElected<Test>>::get(), vec![21, 11]);

		// 21 is rewarded as an uncle producer.
		// 11 is rewarded as a block producer and uncle referencer and uncle producer.
		assert_eq!(CurrentEraPointsEarned::get().individual, vec![1, 20 + 2 * 3 + 1]);
		assert_eq!(CurrentEraPointsEarned::get().total, 28);
	})
}

#[test]
fn add_reward_points_fns_works() {
	ExtBuilder::default().build().execute_with(|| {
		let validators = <Module<Test>>::current_elected();
		// Not mandatory but must be coherent with rewards.
		assert_eq!(validators, vec![21, 11]);

		<Module<Test>>::reward_by_indices(vec![(0, 1), (1, 1), (2, 1), (1, 1)]);

		<Module<Test>>::reward_by_ids(vec![(21, 1), (11, 1), (31, 1), (11, 1)]);

		assert_eq!(CurrentEraPointsEarned::get().individual, vec![2, 4]);
		assert_eq!(CurrentEraPointsEarned::get().total, 6);
	})
}

#[test]
fn unbonded_balance_is_not_slashable() {
	ExtBuilder::default().build().execute_with(|| {
		// Total amount staked is slashable.
		assert_eq!(Staking::ledger(&10).unwrap().active_ring, 1000);

		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Ring(800)));

		// Only the active portion.
		assert_eq!(Staking::ledger(&10).unwrap().active_ring, 200);
	})
}

#[test]
fn era_is_always_same_length() {
	// This ensures that the sessions is always of the same length if there is no forcing no
	// session changes.
	ExtBuilder::default().build().execute_with(|| {
		start_era(1);
		assert_eq!(Staking::current_era_start_session_index(), SessionsPerEra::get());

		start_era(2);
		assert_eq!(Staking::current_era_start_session_index(), SessionsPerEra::get() * 2);

		let session = Session::current_index();
		ForceEra::put(Forcing::ForceNew);
		advance_session();
		assert_eq!(Staking::current_era(), 3);
		assert_eq!(Staking::current_era_start_session_index(), session + 1);

		start_era(4);
		assert_eq!(
			Staking::current_era_start_session_index(),
			session + SessionsPerEra::get() + 1
		);
	});
}

#[test]
fn offence_forces_new_era() {
	ExtBuilder::default().build().execute_with(|| {
		Staking::on_offence(
			&[OffenceDetails {
				offender: (11, Staking::stakers(&11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(5)],
		);

		assert_eq!(Staking::force_era(), Forcing::ForceNew);
	});
}

#[test]
fn offence_ensures_new_era_without_clobbering() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Staking::force_new_era_always(Origin::ROOT));

		Staking::on_offence(
			&[OffenceDetails {
				offender: (11, Staking::stakers(&11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(5)],
		);

		assert_eq!(Staking::force_era(), Forcing::ForceAlways);
	});
}

//#[test]
//fn normal_kton_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		{
//			let (stash, controller) = (1001, 1000);
//
//			Kton::deposit_creating(&stash, 10 * COIN);
//			assert_ok!(Staking::bond(
//				Origin::signed(stash),
//				controller,
//				StakingBalance::Kton(10 * COIN),
//				RewardDestination::Stash,
//				0,
//			));
//			assert_eq!(
//				Staking::ledger(&controller).unwrap(),
//				StakingLedger {
//					stash,
//					active_ring: 0,
//					active_deposit_ring: 0,
//					active_kton: 10 * COIN,
//					deposit_items: vec![],
//					ring_staking_lock: Default::default(),
//					kton_staking_lock: StakingLock {
//						staking_amount: 10 * COIN,
//						unbondings: vec![],
//					},
//				}
//			);
//			assert_eq!(
//				Kton::locks(&stash),
//				vec![BalanceLock {
//					id: STAKING_ID,
//					withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//						staking_amount: 10 * COIN,
//						unbondings: vec![],
//					}),
//					reasons: WithdrawReasons::all(),
//				}]
//			);
//		}
//
//		{
//			let (stash, controller) = (2001, 2000);
//
//			// promise_month should not work for kton
//			Kton::deposit_creating(&stash, 10 * COIN);
//			assert_ok!(Staking::bond(
//				Origin::signed(stash),
//				controller,
//				StakingBalance::Kton(10 * COIN),
//				RewardDestination::Stash,
//				12,
//			));
//			assert_eq!(
//				Staking::ledger(&controller).unwrap(),
//				StakingLedger {
//					stash,
//					active_ring: 0,
//					active_deposit_ring: 0,
//					active_kton: 10 * COIN,
//					deposit_items: vec![],
//					ring_staking_lock: Default::default(),
//					kton_staking_lock: StakingLock {
//						staking_amount: 10 * COIN,
//						unbondings: vec![],
//					},
//				}
//			);
//		}
//	});
//}
//
//#[test]
//fn time_deposit_ring_unbond_and_withdraw_automatically_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let (stash, controller) = (11, 10);
//
//		{
//			let locks = vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 100 * COIN,
//					unbondings: vec![],
//				}),
//				reasons: WithdrawReasons::all(),
//			}];
//			let ledger = StakingLedger {
//				stash,
//				active_ring: 100 * COIN,
//				active_deposit_ring: 100 * COIN,
//				active_kton: 0,
//				deposit_items: vec![TimeDepositItem {
//					value: 100 * COIN,
//					start_time: 0,
//					expire_time: (12 * MONTH_IN_SECONDS) as _,
//				}],
//				ring_staking_lock: StakingLock {
//					staking_amount: 100 * COIN,
//					unbondings: vec![],
//				},
//				kton_staking_lock: Default::default(),
//			};
//
//			assert_ok!(Staking::unbond(
//				Origin::signed(controller),
//				StakingBalance::Ring(10 * COIN)
//			));
//			assert_eq!(Ring::locks(stash), locks);
//			assert_eq!(Staking::ledger(&controller).unwrap(), ledger,);
//
//			assert_ok!(Staking::unbond(
//				Origin::signed(controller),
//				StakingBalance::Ring(120 * COIN)
//			));
//			assert_eq!(Ring::locks(stash), locks);
//			assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
//		}
//
//		{
//			let (unbond_start, unbond_value) = ((13 * MONTH_IN_SECONDS) as _, 10 * COIN);
//			Timestamp::set_timestamp(unbond_start);
//
//			assert_ok!(Staking::unbond(
//				Origin::signed(controller),
//				StakingBalance::Ring(unbond_value)
//			));
//			assert_eq!(
//				Ring::locks(stash),
//				vec![BalanceLock {
//					id: STAKING_ID,
//					withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//						staking_amount: 100 * COIN - unbond_value,
//						unbondings: vec![NormalLock {
//							amount: unbond_value,
//							until: unbond_start + BondingDuration::get(),
//						}],
//					}),
//					reasons: WithdrawReasons::all(),
//				}]
//			);
//			assert_eq!(
//				Staking::ledger(&controller).unwrap(),
//				StakingLedger {
//					stash,
//					active_ring: 100 * COIN - unbond_value,
//					active_deposit_ring: 0,
//					active_kton: 0,
//					deposit_items: vec![],
//					ring_staking_lock: StakingLock {
//						staking_amount: 100 * COIN - unbond_value,
//						unbondings: vec![NormalLock {
//							amount: unbond_value,
//							until: unbond_start + BondingDuration::get(),
//						}],
//					},
//					kton_staking_lock: Default::default(),
//				}
//			);
//
//			Timestamp::set_timestamp(unbond_start + BondingDuration::get());
//			assert_err!(
//				Ring::ensure_can_withdraw(
//					&stash,
//					unbond_value,
//					WithdrawReason::Transfer.into(),
//					100 * COIN - unbond_value - 1,
//				),
//				"account liquidity restrictions prevent withdrawal"
//			);
//			assert_ok!(Ring::ensure_can_withdraw(
//				&stash,
//				unbond_value,
//				WithdrawReason::Transfer.into(),
//				100 * COIN - unbond_value,
//			));
//		}
//	});
//}
//
//#[test]
//fn normal_unbond_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let (stash, controller) = (11, 10);
//		let value = 200 * COIN;
//		let promise_month = 12;
//		let _ = Ring::deposit_creating(&stash, 1000 * COIN);
//
//		{
//			let kton_free_balance = Kton::free_balance(&stash);
//			let mut ledger = Staking::ledger(&controller).unwrap();
//
//			assert_ok!(Staking::bond_extra(
//				Origin::signed(stash),
//				StakingBalance::Ring(value),
//				promise_month,
//			));
//			assert_eq!(
//				Kton::free_balance(&stash),
//				kton_free_balance + inflation::compute_kton_return::<Test>(value, promise_month)
//			);
//			ledger.active_ring += value;
//			ledger.active_deposit_ring += value;
//			ledger.deposit_items.push(TimeDepositItem {
//				value,
//				start_time: 0,
//				expire_time: (promise_month * MONTH_IN_SECONDS) as _,
//			});
//			ledger.ring_staking_lock.staking_amount += value;
//			assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
//		}
//
//		{
//			let kton_free_balance = Kton::free_balance(&stash);
//			let mut ledger = Staking::ledger(&controller).unwrap();
//
//			// we try to bond 1 kton, but stash only has 0.03 Kton
//			// extra = 1.min(0.03)
//			// bond += 0.03
//			assert_ok!(Staking::bond_extra(
//				Origin::signed(stash),
//				StakingBalance::Kton(COIN),
//				0
//			));
//			ledger.active_kton += kton_free_balance;
//			ledger.kton_staking_lock.staking_amount += kton_free_balance;
//			assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
//
//			assert_ok!(Staking::unbond(
//				Origin::signed(controller),
//				StakingBalance::Kton(kton_free_balance)
//			));
//			ledger.active_kton = 0;
//			ledger.kton_staking_lock.staking_amount = 0;
//			ledger.kton_staking_lock.unbondings.push(NormalLock {
//				amount: kton_free_balance,
//				until: BondingDuration::get(),
//			});
//			assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
//		}
//	});
//}
//
//#[test]
//fn punished_claim_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let (stash, controller) = (1001, 1000);
//		let promise_month = 36;
//		let _ = Ring::deposit_creating(&stash, 100 * COIN);
//		Kton::deposit_creating(&stash, COIN / 100000);
//		let mut ledger = StakingLedger {
//			stash,
//			active_ring: 10 * COIN,
//			active_deposit_ring: 10 * COIN,
//			active_kton: 0,
//			deposit_items: vec![TimeDepositItem {
//				value: 10 * COIN,
//				start_time: 0,
//				expire_time: (promise_month * MONTH_IN_SECONDS) as _,
//			}],
//			ring_staking_lock: StakingLock {
//				staking_amount: 10 * COIN,
//				unbondings: vec![],
//			},
//			kton_staking_lock: Default::default(),
//		};
//
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Ring(10 * COIN),
//			RewardDestination::Stash,
//			promise_month,
//		));
//		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
//		// kton is 0, skip unbond_with_punish
//		assert_ok!(Staking::claim_deposits_with_punish(
//			Origin::signed(controller),
//			(promise_month * MONTH_IN_SECONDS) as _,
//		));
//		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
//
//		// set more kton balance to make it work
//		Kton::deposit_creating(&stash, 10 * COIN);
//		let kton_free_balance = Kton::free_balance(&stash);
//		let kton_punishment = inflation::compute_kton_return::<Test>(10 * COIN, promise_month);
//		assert_ok!(Staking::claim_deposits_with_punish(
//			Origin::signed(controller),
//			(promise_month * MONTH_IN_SECONDS) as _,
//		));
//		ledger.active_deposit_ring -= 10 * COIN;
//		ledger.deposit_items.clear();
//		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
//		assert_eq!(Kton::free_balance(&stash), kton_free_balance - 3 * kton_punishment);
//	});
//}
//
//#[test]
//fn transform_to_deposited_ring_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let (stash, controller) = (1001, 1000);
//		let _ = Ring::deposit_creating(&stash, 100 * COIN);
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Ring(10 * COIN),
//			RewardDestination::Stash,
//			0,
//		));
//		let kton_free_balance = Kton::free_balance(&stash);
//		let mut ledger = Staking::ledger(&controller).unwrap();
//
//		assert_ok!(Staking::deposit_extra(Origin::signed(controller), 5 * COIN, 12));
//		ledger.active_deposit_ring += 5 * COIN;
//		ledger.deposit_items.push(TimeDepositItem {
//			value: 5 * COIN,
//			start_time: 0,
//			expire_time: (12 * MONTH_IN_SECONDS) as u64,
//		});
//		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
//		assert_eq!(Kton::free_balance(&stash), kton_free_balance + (5 * COIN / 10000));
//	});
//}
//
//#[test]
//fn expired_ring_should_capable_to_promise_again() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let (stash, controller) = (1001, 1000);
//		let _ = Ring::deposit_creating(&stash, 100 * COIN);
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Ring(10 * COIN),
//			RewardDestination::Stash,
//			12,
//		));
//		let mut ledger = Staking::ledger(&controller).unwrap();
//		let ts = (13 * MONTH_IN_SECONDS) as u64;
//		let promise_extra_value = 5 * COIN;
//
//		Timestamp::set_timestamp(ts);
//		assert_ok!(Staking::deposit_extra(
//			Origin::signed(controller),
//			promise_extra_value,
//			13,
//		));
//		ledger.active_deposit_ring = promise_extra_value;
//		// old deposit_item with 12 months promised removed
//		ledger.deposit_items = vec![TimeDepositItem {
//			value: promise_extra_value,
//			start_time: ts,
//			expire_time: 2 * ts,
//		}];
//		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
//	});
//}
//
//#[test]
//fn inflation_should_be_correct() {
//	ExtBuilder::default().build().execute_with(|| {
//		let initial_issuance = 1_200_000_000 * COIN;
//		let surplus_needed = initial_issuance - Ring::total_issuance();
//		let _ = Ring::deposit_into_existing(&11, surplus_needed);
//
//		assert_eq!(Ring::total_issuance(), initial_issuance);
//	});
//
//	//	// breakpoint test
//	//	ExtBuilder::default().build().execute_with(|| {
//	//		gen_paired_account!(validator_1_stash(123), validator_1_controller(456), 0);
//	//		gen_paired_account!(validator_2_stash(234), validator_2_controller(567), 0);
//	//		gen_paired_account!(nominator_stash(345), nominator_controller(678), 0);
//	//
//	//		assert_ok!(Staking::validate(
//	//			Origin::signed(validator_1_controller),
//	//			ValidatorPrefs {
//	//				node_name: vec![0; 8],
//	//				..Default::default()
//	//			},
//	//		));
//	//		assert_ok!(Staking::validate(
//	//			Origin::signed(validator_2_controller),
//	//			ValidatorPrefs {
//	//				node_name: vec![1; 8],
//	//				..Default::default()
//	//			},
//	//		));
//	//		assert_ok!(Staking::nominate(
//	//			Origin::signed(nominator_controller),
//	//			vec![validator_1_stash, validator_2_stash],
//	//		));
//	//
//	//		Timestamp::set_timestamp(1_575_448_345_000 - 12_000);
//	//		// breakpoint here
//	//		Staking::new_era(1);
//	//
//	//		Timestamp::set_timestamp(1_575_448_345_000);
//	//		// breakpoint here
//	//		Staking::new_era(2);
//	//
//	//		// breakpoint here
//	//		inflation::compute_total_payout::<Test>(11_999, 1_295_225_000, 9_987_999_900_000_000_000);
//	//
//	//		loop {}
//	//	});
//}
//
//#[test]
//fn validator_prefs_should_work() {
//	ExtBuilder::default().build().execute_with(|| {
//		gen_paired_account!(validator_stash(123), validator_controller(456), 0);
//		gen_paired_account!(nominator_stash(345), nominator_controller(678), 0);
//
//		//		println!(
//		//			"{}, {}",
//		//			Ring::free_balance(&validator_stash),
//		//			Kton::free_balance(&validator_stash)
//		//		);
//		//		println!("{:#?}", Staking::ledger(&validator_controller));
//		//		println!(
//		//			"{}, {}",
//		//			Ring::free_balance(&nominator_stash),
//		//			Kton::free_balance(&nominator_stash)
//		//		);
//		//		println!("{:#?}", Staking::ledger(&nominator_controller));
//
//		assert_ok!(Staking::validate(
//			Origin::signed(validator_controller),
//			ValidatorPrefs {
//				node_name: vec![0; 8],
//				validator_payment_ratio: 50,
//			},
//		));
//		assert_ok!(Staking::nominate(
//			Origin::signed(nominator_controller),
//			vec![validator_stash],
//		));
//
//		println!("{:#?}", Staking::stakers(validator_stash));
//		Staking::new_era(1);
//		println!("{:#?}", Staking::stakers(validator_stash));
//	});
//}
//
//#[test]
//fn reward_and_slash_should_work() {
//	//	ExtBuilder::default().build().execute_with(|| {
//	//		gen_paired_account!(stash_1(123), _c(456), 12);
//	//		gen_paired_account!(stash_2(234), _c(567), 12);
//	//
//	//		<Stakers<Test>>::insert(
//	//			&stash_1,
//	//			Exposure {
//	//				total: 1,
//	//				own: 1,
//	//				others: vec![],
//	//			},
//	//		);
//	//		assert_eq!(Ring::free_balance(&stash_1), 100 * COIN);
//	//		let _ = Staking::reward_validator(&stash_1, 20 * COIN);
//	//		assert_eq!(Ring::free_balance(&stash_1), 120 * COIN);
//	//	});
//
//	ExtBuilder::default().build().execute_with(|| {
//		gen_paired_account!(validator_stash(123), validator_controller(456), 0);
//		gen_paired_account!(nominator_stash(345), nominator_controller(678), 0);
//
//		println!(
//			"{}, {}",
//			Ring::free_balance(&validator_stash),
//			Kton::free_balance(&validator_stash)
//		);
//		println!("{:#?}", Staking::ledger(&validator_controller));
//		println!(
//			"{}, {}",
//			Ring::free_balance(&nominator_stash),
//			Kton::free_balance(&nominator_stash)
//		);
//		println!("{:#?}", Staking::ledger(&nominator_controller));
//
//		assert_ok!(Staking::validate(
//			Origin::signed(validator_controller),
//			ValidatorPrefs {
//				node_name: vec![0; 8],
//				..Default::default()
//			},
//		));
//		assert_ok!(Staking::nominate(
//			Origin::signed(nominator_controller),
//			vec![validator_stash],
//		));
//
//		println!("{:#?}", Staking::stakers(validator_stash));
//		start_era(1);
//		println!("{:#?}", Staking::stakers(validator_stash));
//	});
//}
//
//#[test]
//fn set_controller_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let (stash, old_controller, new_controller) = (11, 10, 12);
//		let ledger = Staking::ledger(&old_controller).unwrap();
//
//		assert_ok!(Staking::set_controller(Origin::signed(stash), new_controller));
//		assert_eq!(Staking::ledger(&old_controller), None);
//		assert_eq!(Staking::ledger(&new_controller).unwrap(), ledger);
//	});
//}
//
//#[test]
//fn slash_should_not_touch_unbondings() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let (stash, controller) = (11, 10);
//		let ledger = Staking::ledger(&controller).unwrap();
//
//		// only deposit_ring, no normal_ring
//		assert_eq!((ledger.active_ring, ledger.active_deposit_ring), (1000, 1000));
//
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(100), 0));
//		Kton::deposit_creating(&stash, 10);
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(10), 0));
//
//		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(10)));
//		let ledger = Staking::ledger(&controller).unwrap();
//		let unbondings = (
//			ledger.ring_staking_lock.unbondings.clone(),
//			ledger.kton_staking_lock.unbondings.clone(),
//		);
//		assert_eq!((ledger.active_ring, ledger.active_deposit_ring), (190, 100));
//
//		<Stakers<Test>>::insert(
//			&stash,
//			Exposure {
//				total: 1,
//				own: 1,
//				others: vec![],
//			},
//		);
//		// FIXME: slash strategy
//		let _ = Staking::slash_validator(
//			&stash,
//			ExtendedBalance::max_value(),
//			&Staking::stakers(&stash),
//			&mut vec![],
//		);
//		let ledger = Staking::ledger(&controller).unwrap();
//		assert_eq!(
//			(
//				ledger.ring_staking_lock.unbondings.clone(),
//				ledger.kton_staking_lock.unbondings.clone(),
//			),
//			unbondings,
//		);
//		assert_eq!((ledger.active_ring, ledger.active_deposit_ring), (0, 0));
//	});
//}
//
//#[test]
//fn bond_over_max_promise_month_should_fail() {
//	ExtBuilder::default().build().execute_with(|| {
//		gen_paired_account!(stash(123), controller(456));
//		assert_err!(
//			Staking::bond(
//				Origin::signed(stash),
//				controller,
//				StakingBalance::Ring(COIN),
//				RewardDestination::Stash,
//				37,
//			),
//			"months at most is 36.",
//		);
//
//		gen_paired_account!(stash(123), controller(456), promise_month(12));
//		assert_err!(
//			Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(COIN), 37),
//			"months at most is 36.",
//		);
//	});
//}
//
//#[test]
//fn check_stash_already_bonded_and_controller_already_paired() {
//	ExtBuilder::default().build().execute_with(|| {
//		gen_paired_account!(unpaired_stash(123), unpaired_controller(456));
//		assert_err!(
//			Staking::bond(
//				Origin::signed(11),
//				unpaired_controller,
//				StakingBalance::Ring(COIN),
//				RewardDestination::Stash,
//				0,
//			),
//			"stash already bonded",
//		);
//		assert_err!(
//			Staking::bond(
//				Origin::signed(unpaired_stash),
//				10,
//				StakingBalance::Ring(COIN),
//				RewardDestination::Stash,
//				0,
//			),
//			"controller already paired",
//		);
//	});
//}
//
//#[test]
//fn pool_should_be_increased_and_decreased_correctly() {
//	ExtBuilder::default().build().execute_with(|| {
//		let mut ring_pool = Staking::ring_pool();
//		let mut kton_pool = Staking::kton_pool();
//
//		// bond: 100COIN
//		gen_paired_account!(stash_1(111), controller_1(222), 0);
//		gen_paired_account!(stash_2(333), controller_2(444), promise_month(12));
//		ring_pool += 100 * COIN;
//		kton_pool += 100 * COIN;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//		assert_eq!(Staking::kton_pool(), kton_pool);
//
//		// unbond: 50Ring 50Kton
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_1),
//			StakingBalance::Ring(50 * COIN)
//		));
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_1),
//			StakingBalance::Kton(25 * COIN)
//		));
//		// not yet expired: promise for 12 months
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_2),
//			StakingBalance::Ring(50 * COIN)
//		));
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_2),
//			StakingBalance::Kton(25 * COIN)
//		));
//		ring_pool -= 50 * COIN;
//		kton_pool -= 50 * COIN;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//		assert_eq!(Staking::kton_pool(), kton_pool);
//
//		// claim: 50Ring
//		assert_ok!(Staking::claim_deposits_with_punish(
//			Origin::signed(controller_2),
//			(promise_month * MONTH_IN_SECONDS) as u64,
//		));
//		// unbond deposit items: 12.5Ring
//		Timestamp::set_timestamp((promise_month * MONTH_IN_SECONDS) as u64);
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_2),
//			StakingBalance::Ring(125 * COIN / 10),
//		));
//		ring_pool -= 125 * COIN / 10;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//
//		// slash: 37.5Ring 50Kton
//		<Stakers<Test>>::insert(
//			&stash_1,
//			Exposure {
//				total: 1,
//				own: 1,
//				others: vec![],
//			},
//		);
//		<Stakers<Test>>::insert(
//			&stash_2,
//			Exposure {
//				total: 1,
//				own: 1,
//				others: vec![],
//			},
//		);
//		// FIXME: slash strategy
//		let _ = Staking::slash_validator(
//			&stash_1,
//			ExtendedBalance::max_value(),
//			&Staking::stakers(&stash_1),
//			&mut vec![],
//		);
//		// FIXME: slash strategy
//		let _ = Staking::slash_validator(
//			&stash_2,
//			ExtendedBalance::max_value(),
//			&Staking::stakers(&stash_2),
//			&mut vec![],
//		);
//		ring_pool -= 375 * COIN / 10;
//		kton_pool -= 50 * COIN;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//		assert_eq!(Staking::kton_pool(), kton_pool);
//	});
//}
//
//#[test]
//fn unbond_over_max_unbondings_chunks_should_fail() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		gen_paired_account!(stash(123), controller(456));
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Ring(COIN),
//			RewardDestination::Stash,
//			0,
//		));
//
//		for ts in 0..MAX_UNLOCKING_CHUNKS {
//			Timestamp::set_timestamp(ts as u64);
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(1)));
//		}
//
//		assert_err!(
//			Staking::unbond(Origin::signed(controller), StakingBalance::Ring(1)),
//			"can not schedule more unlock chunks",
//		);
//	});
//}
//
//#[test]
//fn promise_extra_should_not_remove_unexpired_items() {
//	ExtBuilder::default().build().execute_with(|| {
//		gen_paired_account!(stash(123), controller(456), promise_month(12));
//		let expired_items_len = 3;
//		let expiry_date = (promise_month * MONTH_IN_SECONDS) as u64;
//
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(stash),
//			StakingBalance::Ring(5 * COIN),
//			0,
//		));
//		for _ in 0..expired_items_len {
//			assert_ok!(Staking::deposit_extra(Origin::signed(controller), COIN, promise_month));
//		}
//
//		Timestamp::set_timestamp(expiry_date - 1);
//		assert_ok!(Staking::deposit_extra(
//			Origin::signed(controller),
//			2 * COIN,
//			promise_month,
//		));
//		assert_eq!(
//			Staking::ledger(&controller).unwrap().deposit_items.len(),
//			2 + expired_items_len,
//		);
//
//		Timestamp::set_timestamp(expiry_date);
//		assert_ok!(Staking::deposit_extra(
//			Origin::signed(controller),
//			2 * COIN,
//			promise_month,
//		));
//		assert_eq!(Staking::ledger(&controller).unwrap().deposit_items.len(), 2);
//	});
//}
//
//#[test]
//fn unbond_zero() {
//	ExtBuilder::default().build().execute_with(|| {
//		gen_paired_account!(stash(123), controller(456), promise_month(12));
//		let ledger = Staking::ledger(&controller).unwrap();
//
//		Timestamp::set_timestamp((promise_month * MONTH_IN_SECONDS) as u64);
//		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Ring(0)));
//		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Kton(0)));
//		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
//	});
//}
//
//// bond 10_000 Ring for 12 months, gain 1 Kton
//// bond extra 10_000 Ring for 36 months, gain 3 Kton
//// bond extra 1 Kton
//// nominate
//// unlock the 12 months deposit item with punish
//// lost 3 Kton and 10_000 Ring's power for nominate
//#[test]
//fn yakio_q1() {
//	ExtBuilder::default().build().execute_with(|| {
//		let (stash, controller) = (777, 888);
//		let _ = Ring::deposit_creating(&stash, 20_000);
//
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Ring(10_000),
//			RewardDestination::Stash,
//			12,
//		));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(stash),
//			StakingBalance::Ring(10_000),
//			36,
//		));
//		assert_eq!(Kton::free_balance(&stash), 4);
//
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(1), 36));
//		assert_eq!(Staking::ledger(&controller).unwrap().active_kton, 1);
//
//		assert_ok!(Staking::nominate(Origin::signed(controller), vec![controller]));
//
//		assert_ok!(Staking::claim_deposits_with_punish(
//			Origin::signed(controller),
//			(12 * MONTH_IN_SECONDS) as u64,
//		));
//		assert_eq!(Kton::free_balance(&stash), 1);
//
//		let ledger = Staking::ledger(&controller).unwrap();
//		// not enough Kton to unbond
//		assert_ok!(Staking::claim_deposits_with_punish(
//			Origin::signed(controller),
//			(36 * MONTH_IN_SECONDS) as u64,
//		));
//		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
//	});
//}
//
//// how to balance the power and calculate the reward if some validators have been chilled
//#[test]
//fn yakio_q2() {
//	fn run(with_new_era: bool) -> Balance {
//		let mut balance = 0;
//		ExtBuilder::default().build().execute_with(|| {
//			gen_paired_account!(validator_1_stash(123), validator_1_controller(456), 0);
//			gen_paired_account!(validator_2_stash(234), validator_2_controller(567), 0);
//			gen_paired_account!(nominator_stash(345), nominator_controller(678), 0);
//
//			assert_ok!(Staking::validate(
//				Origin::signed(validator_1_controller),
//				ValidatorPrefs {
//					node_name: vec![0; 8],
//					..Default::default()
//				},
//			));
//			assert_ok!(Staking::validate(
//				Origin::signed(validator_2_controller),
//				ValidatorPrefs {
//					node_name: vec![1; 8],
//					..Default::default()
//				},
//			));
//			assert_ok!(Staking::nominate(
//				Origin::signed(nominator_controller),
//				vec![validator_1_stash, validator_2_stash],
//			));
//
//			start_era(1);
//			assert_ok!(Staking::chill(Origin::signed(validator_1_controller)));
//			// assert_ok!(Staking::chill(Origin::signed(validator_2_controller)));
//			if with_new_era {
//				start_era(2);
//			}
//			let _ = Staking::reward_validator(&validator_1_stash, 1000 * COIN);
//			let _ = Staking::reward_validator(&validator_2_stash, 1000 * COIN);
//
//			balance = Ring::free_balance(&nominator_stash);
//		});
//
//		balance
//	}
//
//	let free_balance = run(false);
//	let free_balance_with_new_era = run(true);
//
//	assert_ne!(free_balance, 0);
//	assert_ne!(free_balance_with_new_era, 0);
//	assert!(free_balance > free_balance_with_new_era);
//}
//
//#[test]
//fn xavier_q1() {
//	ExtBuilder::default().build().execute_with(|| {
//		let stash = 123;
//		let controller = 456;
//		Kton::deposit_creating(&stash, 10);
//
//		Timestamp::set_timestamp(0);
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Kton(5),
//			RewardDestination::Stash,
//			0,
//		));
//		assert_eq!(Timestamp::get(), 0);
//		assert_eq!(Kton::free_balance(stash), 10);
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 5,
//					unbondings: vec![],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Init - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Ok Init - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!();
//
//		Timestamp::set_timestamp(1);
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(5), 0));
//		assert_eq!(Timestamp::get(), 1);
//		assert_eq!(Kton::free_balance(stash), 10);
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 10,
//					unbondings: vec![],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Bond Extra - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Ok Bond Extra - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!();
//
//		let unbond_start = 2;
//		Timestamp::set_timestamp(unbond_start);
//		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Kton(9)));
//		assert_eq!(Timestamp::get(), 2);
//		assert_eq!(Kton::free_balance(stash), 10);
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 1,
//					unbondings: vec![NormalLock {
//						amount: 9,
//						until: BondingDuration::get() + unbond_start,
//					}],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Unbond - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Ok Unbond - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!();
//
//		assert_err!(
//			Kton::transfer(Origin::signed(stash), controller, 1),
//			"account liquidity restrictions prevent withdrawal",
//		);
//		//		println!("Locking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Locking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!();
//
//		Timestamp::set_timestamp(BondingDuration::get() + unbond_start);
//		assert_ok!(Kton::transfer(Origin::signed(stash), controller, 1));
//		//		println!("Unlocking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Unlocking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!(
//		//			"Unlocking Transfer - Kton StakingLedger: {:#?}",
//		//			Staking::ledger(&controller)
//		//		);
//		//		println!();
//		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start);
//		assert_eq!(Kton::free_balance(stash), 9);
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 1,
//					unbondings: vec![NormalLock {
//						amount: 9,
//						until: BondingDuration::get() + unbond_start,
//					}],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//
//		Kton::deposit_creating(&stash, 20);
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(19), 0));
//		assert_eq!(Kton::free_balance(stash), 29);
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 20,
//					unbondings: vec![NormalLock {
//						amount: 9,
//						until: BondingDuration::get() + unbond_start,
//					}],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		assert_eq!(
//			Staking::ledger(&controller).unwrap(),
//			StakingLedger {
//				stash: 123,
//				active_ring: 0,
//				active_deposit_ring: 0,
//				active_kton: 20,
//				deposit_items: vec![],
//				ring_staking_lock: Default::default(),
//				kton_staking_lock: StakingLock {
//					staking_amount: 20,
//					unbondings: vec![NormalLock {
//						amount: 9,
//						until: BondingDuration::get() + unbond_start,
//					}],
//				},
//			}
//		);
//		//		println!("Unlocking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Unlocking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!(
//		//			"Unlocking Transfer - Kton StakingLedger: {:#?}",
//		//			Staking::ledger(&controller)
//		//		);
//		//		println!();
//	});
//
//	ExtBuilder::default().build().execute_with(|| {
//		let stash = 123;
//		let controller = 456;
//		let _ = Ring::deposit_creating(&stash, 10);
//
//		Timestamp::set_timestamp(0);
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Ring(5),
//			RewardDestination::Stash,
//			0,
//		));
//		assert_eq!(Timestamp::get(), 0);
//		assert_eq!(Ring::free_balance(stash), 10);
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 5,
//					unbondings: vec![],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Init - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Ok Init - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!();
//
//		Timestamp::set_timestamp(1);
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(5), 0));
//		assert_eq!(Timestamp::get(), 1);
//		assert_eq!(Ring::free_balance(stash), 10);
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 10,
//					unbondings: vec![],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Bond Extra - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Ok Bond Extra - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!();
//
//		let unbond_start = 2;
//		Timestamp::set_timestamp(unbond_start);
//		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(9)));
//		assert_eq!(Timestamp::get(), 2);
//		assert_eq!(Ring::free_balance(stash), 10);
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 1,
//					unbondings: vec![NormalLock {
//						amount: 9,
//						until: BondingDuration::get() + unbond_start,
//					}],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Unbond - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Ok Unbond - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!();
//
//		assert_err!(
//			Ring::transfer(Origin::signed(stash), controller, 1),
//			"account liquidity restrictions prevent withdrawal",
//		);
//		//		println!("Locking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Locking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!();
//
//		Timestamp::set_timestamp(BondingDuration::get() + unbond_start);
//		assert_ok!(Ring::transfer(Origin::signed(stash), controller, 1));
//		//		println!("Unlocking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Unlocking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!(
//		//			"Unlocking Transfer - Ring StakingLedger: {:#?}",
//		//			Staking::ledger(&controller)
//		//		);
//		//		println!();
//		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start);
//		assert_eq!(Ring::free_balance(stash), 9);
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 1,
//					unbondings: vec![NormalLock {
//						amount: 9,
//						until: BondingDuration::get() + unbond_start,
//					}],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//
//		let _ = Ring::deposit_creating(&stash, 20);
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(19), 0));
//		assert_eq!(Ring::free_balance(stash), 29);
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 20,
//					unbondings: vec![NormalLock {
//						amount: 9,
//						until: BondingDuration::get() + unbond_start,
//					}],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		assert_eq!(
//			Staking::ledger(&controller).unwrap(),
//			StakingLedger {
//				stash: 123,
//				active_ring: 20,
//				active_deposit_ring: 0,
//				active_kton: 0,
//				deposit_items: vec![],
//				ring_staking_lock: StakingLock {
//					staking_amount: 20,
//					unbondings: vec![NormalLock {
//						amount: 9,
//						until: BondingDuration::get() + unbond_start,
//					}],
//				},
//				kton_staking_lock: Default::default(),
//			}
//		);
//		//		println!("Unlocking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Unlocking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!(
//		//			"Unlocking Transfer - Ring StakingLedger: {:#?}",
//		//			Staking::ledger(&controller)
//		//		);
//		//		println!();
//	});
//}
//
//#[test]
//fn xavier_q2() {
//	ExtBuilder::default().build().execute_with(|| {
//		let stash = 123;
//		let controller = 456;
//		Kton::deposit_creating(&stash, 10);
//
//		Timestamp::set_timestamp(1);
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Kton(5),
//			RewardDestination::Stash,
//			0,
//		));
//		assert_eq!(Kton::free_balance(stash), 10);
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 5,
//					unbondings: vec![],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Init - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Ok Init - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!();
//
//		Timestamp::set_timestamp(1);
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(4), 0));
//		assert_eq!(Timestamp::get(), 1);
//		assert_eq!(Kton::free_balance(stash), 10);
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 9,
//					unbondings: vec![],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Bond Extra - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Ok Bond Extra - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!();
//
//		let (unbond_start_1, unbond_value_1) = (2, 2);
//		Timestamp::set_timestamp(unbond_start_1);
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller),
//			StakingBalance::Kton(unbond_value_1),
//		));
//		assert_eq!(Timestamp::get(), unbond_start_1);
//		assert_eq!(Kton::free_balance(stash), 10);
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 7,
//					unbondings: vec![NormalLock {
//						amount: 2,
//						until: BondingDuration::get() + unbond_start_1,
//					}],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Unbond - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Ok Unbond - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!();
//
//		let (unbond_start_2, unbond_value_2) = (3, 6);
//		Timestamp::set_timestamp(unbond_start_2);
//		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Kton(6)));
//		assert_eq!(Timestamp::get(), unbond_start_2);
//		assert_eq!(Kton::free_balance(stash), 10);
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 1,
//					unbondings: vec![
//						NormalLock {
//							amount: 2,
//							until: BondingDuration::get() + unbond_start_1,
//						},
//						NormalLock {
//							amount: 6,
//							until: BondingDuration::get() + unbond_start_2,
//						}
//					],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Unbond - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Ok Unbond - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!();
//
//		assert_err!(
//			Kton::transfer(Origin::signed(stash), controller, unbond_value_1),
//			"account liquidity restrictions prevent withdrawal",
//		);
//		//		println!("Locking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Locking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!();
//
//		assert_ok!(Kton::transfer(Origin::signed(stash), controller, unbond_value_1 - 1));
//		assert_eq!(Kton::free_balance(stash), 9);
//		//		println!("Normal Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Normal Transfer - Kton Locks: {:#?}", Kton::locks(stash));
//
//		Timestamp::set_timestamp(BondingDuration::get() + unbond_start_1);
//		assert_err!(
//			Kton::transfer(Origin::signed(stash), controller, unbond_value_1 + 1),
//			"account liquidity restrictions prevent withdrawal",
//		);
//		//		println!("Locking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Locking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
//		//		println!();
//		assert_ok!(Kton::transfer(Origin::signed(stash), controller, unbond_value_1));
//		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start_1);
//		assert_eq!(Kton::free_balance(stash), 7);
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 1,
//					unbondings: vec![
//						NormalLock {
//							amount: 2,
//							until: BondingDuration::get() + unbond_start_1,
//						},
//						NormalLock {
//							amount: 6,
//							until: BondingDuration::get() + unbond_start_2,
//						}
//					],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Unlocking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Unlocking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
//
//		Timestamp::set_timestamp(BondingDuration::get() + unbond_start_2);
//		assert_ok!(Kton::transfer(Origin::signed(stash), controller, unbond_value_2));
//		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start_2);
//		assert_eq!(Kton::free_balance(stash), 1);
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 1,
//					unbondings: vec![
//						NormalLock {
//							amount: 2,
//							until: BondingDuration::get() + unbond_start_1,
//						},
//						NormalLock {
//							amount: 6,
//							until: BondingDuration::get() + unbond_start_2,
//						}
//					],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Unlocking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
//		//		println!("Unlocking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
//
//		Kton::deposit_creating(&stash, 1);
//		//		println!("Staking Ledger: {:#?}", Staking::ledger(controller).unwrap());
//		assert_eq!(Kton::free_balance(stash), 2);
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(1), 0));
//		assert_eq!(
//			Kton::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 2,
//					unbondings: vec![
//						NormalLock {
//							amount: 2,
//							until: BondingDuration::get() + unbond_start_1,
//						},
//						NormalLock {
//							amount: 6,
//							until: BondingDuration::get() + unbond_start_2,
//						}
//					],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//	});
//
//	ExtBuilder::default().build().execute_with(|| {
//		let stash = 123;
//		let controller = 456;
//		let _ = Ring::deposit_creating(&stash, 10);
//
//		Timestamp::set_timestamp(1);
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Ring(5),
//			RewardDestination::Stash,
//			0,
//		));
//		assert_eq!(Ring::free_balance(stash), 10);
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 5,
//					unbondings: vec![],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Init - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Ok Init - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!();
//
//		Timestamp::set_timestamp(1);
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(4), 0));
//		assert_eq!(Timestamp::get(), 1);
//		assert_eq!(Ring::free_balance(stash), 10);
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 9,
//					unbondings: vec![],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Bond Extra - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Ok Bond Extra - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!();
//
//		let (unbond_start_1, unbond_value_1) = (2, 2);
//		Timestamp::set_timestamp(unbond_start_1);
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller),
//			StakingBalance::Ring(unbond_value_1)
//		));
//		assert_eq!(Timestamp::get(), unbond_start_1);
//		assert_eq!(Ring::free_balance(stash), 10);
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 7,
//					unbondings: vec![NormalLock {
//						amount: 2,
//						until: BondingDuration::get() + unbond_start_1,
//					},],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Unbond - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Ok Unbond - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!();
//
//		let (unbond_start_2, unbond_value_2) = (3, 6);
//		Timestamp::set_timestamp(unbond_start_2);
//		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(6)));
//		assert_eq!(Timestamp::get(), unbond_start_2);
//		assert_eq!(Ring::free_balance(stash), 10);
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 1,
//					unbondings: vec![
//						NormalLock {
//							amount: 2,
//							until: BondingDuration::get() + unbond_start_1,
//						},
//						NormalLock {
//							amount: 6,
//							until: BondingDuration::get() + unbond_start_2,
//						}
//					],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Ok Unbond - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Ok Unbond - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!();
//
//		assert_err!(
//			Ring::transfer(Origin::signed(stash), controller, unbond_value_1),
//			"account liquidity restrictions prevent withdrawal",
//		);
//		//		println!("Locking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Locking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!();
//
//		assert_ok!(Ring::transfer(Origin::signed(stash), controller, unbond_value_1 - 1));
//		assert_eq!(Ring::free_balance(stash), 9);
//		//		println!("Normal Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Normal Transfer - Ring Locks: {:#?}", Ring::locks(stash));
//
//		Timestamp::set_timestamp(BondingDuration::get() + unbond_start_1);
//		assert_err!(
//			Ring::transfer(Origin::signed(stash), controller, unbond_value_1 + 1),
//			"account liquidity restrictions prevent withdrawal",
//		);
//		//		println!("Locking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Locking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
//		//		println!();
//		assert_ok!(Ring::transfer(Origin::signed(stash), controller, unbond_value_1));
//		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start_1);
//		assert_eq!(Ring::free_balance(stash), 7);
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 1,
//					unbondings: vec![
//						NormalLock {
//							amount: 2,
//							until: BondingDuration::get() + unbond_start_1,
//						},
//						NormalLock {
//							amount: 6,
//							until: BondingDuration::get() + unbond_start_2,
//						}
//					],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Unlocking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Unlocking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
//
//		Timestamp::set_timestamp(BondingDuration::get() + unbond_start_2);
//		assert_ok!(Ring::transfer(Origin::signed(stash), controller, unbond_value_2));
//		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start_2);
//		assert_eq!(Ring::free_balance(stash), 1);
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 1,
//					unbondings: vec![
//						NormalLock {
//							amount: 2,
//							until: BondingDuration::get() + unbond_start_1,
//						},
//						NormalLock {
//							amount: 6,
//							until: BondingDuration::get() + unbond_start_2,
//						}
//					],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//		//		println!("Unlocking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
//		//		println!("Unlocking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
//
//		let _ = Ring::deposit_creating(&stash, 1);
//		//		println!("Staking Ledger: {:#?}", Staking::ledger(controller).unwrap());
//		assert_eq!(Ring::free_balance(stash), 2);
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(1), 0));
//		assert_eq!(
//			Ring::locks(stash),
//			vec![BalanceLock {
//				id: STAKING_ID,
//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
//					staking_amount: 2,
//					unbondings: vec![
//						NormalLock {
//							amount: 2,
//							until: BondingDuration::get() + unbond_start_1,
//						},
//						NormalLock {
//							amount: 6,
//							until: BondingDuration::get() + unbond_start_2,
//						}
//					],
//				}),
//				reasons: WithdrawReasons::all(),
//			}]
//		);
//	});
//}
//
//#[test]
//fn xavier_q3() {
//	ExtBuilder::default().build().execute_with(|| {
//		let stash = 123;
//		let controller = 456;
//		Kton::deposit_creating(&stash, 10);
//
//		Timestamp::set_timestamp(1);
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Kton(5),
//			RewardDestination::Stash,
//			0,
//		));
//		assert_eq!(Timestamp::get(), 1);
//		assert_eq!(
//			Staking::ledger(&controller).unwrap(),
//			StakingLedger {
//				stash: 123,
//				active_ring: 0,
//				active_deposit_ring: 0,
//				active_kton: 5,
//				deposit_items: vec![],
//				ring_staking_lock: Default::default(),
//				kton_staking_lock: StakingLock {
//					staking_amount: 5,
//					unbondings: vec![],
//				},
//			}
//		);
//		//		println!("Locks: {:#?}", Kton::locks(stash));
//		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
//		//		println!();
//
//		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Kton(5)));
//		assert_eq!(
//			Staking::ledger(&controller).unwrap(),
//			StakingLedger {
//				stash: 123,
//				active_ring: 0,
//				active_deposit_ring: 0,
//				active_kton: 0,
//				deposit_items: vec![],
//				ring_staking_lock: Default::default(),
//				kton_staking_lock: StakingLock {
//					staking_amount: 0,
//					unbondings: vec![NormalLock { amount: 5, until: 61 }],
//				},
//			}
//		);
//		//		println!("Locks: {:#?}", Kton::locks(stash));
//		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
//		//		println!();
//
//		Timestamp::set_timestamp(61);
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(1), 0));
//		assert_eq!(Timestamp::get(), 61);
//		assert_eq!(
//			Staking::ledger(&controller).unwrap(),
//			StakingLedger {
//				stash: 123,
//				active_ring: 0,
//				active_deposit_ring: 0,
//				active_kton: 1,
//				deposit_items: vec![],
//				ring_staking_lock: Default::default(),
//				kton_staking_lock: StakingLock {
//					staking_amount: 1,
//					unbondings: vec![NormalLock { amount: 5, until: 61 }],
//				},
//			}
//		);
//		//		println!("Locks: {:#?}", Kton::locks(stash));
//		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
//		//		println!();
//	});
//
//	ExtBuilder::default().build().execute_with(|| {
//		let stash = 123;
//		let controller = 456;
//		let _ = Ring::deposit_creating(&stash, 10);
//
//		Timestamp::set_timestamp(1);
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Ring(5),
//			RewardDestination::Stash,
//			0,
//		));
//		assert_eq!(Timestamp::get(), 1);
//		assert_eq!(
//			Staking::ledger(&controller).unwrap(),
//			StakingLedger {
//				stash: 123,
//				active_ring: 5,
//				active_deposit_ring: 0,
//				active_kton: 0,
//				deposit_items: vec![],
//				ring_staking_lock: StakingLock {
//					staking_amount: 5,
//					unbondings: vec![],
//				},
//				kton_staking_lock: Default::default(),
//			}
//		);
//		//		println!("Locks: {:#?}", Ring::locks(stash));
//		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
//		//		println!();
//
//		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(5)));
//		assert_eq!(
//			Staking::ledger(&controller).unwrap(),
//			StakingLedger {
//				stash: 123,
//				active_ring: 0,
//				active_deposit_ring: 0,
//				active_kton: 0,
//				deposit_items: vec![],
//				ring_staking_lock: StakingLock {
//					staking_amount: 0,
//					unbondings: vec![NormalLock { amount: 5, until: 61 }],
//				},
//				kton_staking_lock: Default::default(),
//			}
//		);
//		//		println!("Locks: {:#?}", Ring::locks(stash));
//		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
//		//		println!();
//
//		Timestamp::set_timestamp(61);
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(1), 0));
//		assert_eq!(Timestamp::get(), 61);
//		assert_eq!(
//			Staking::ledger(&controller).unwrap(),
//			StakingLedger {
//				stash: 123,
//				active_ring: 1,
//				active_deposit_ring: 0,
//				active_kton: 0,
//				deposit_items: vec![],
//				ring_staking_lock: StakingLock {
//					staking_amount: 1,
//					unbondings: vec![NormalLock { amount: 5, until: 61 }],
//				},
//				kton_staking_lock: Default::default(),
//			}
//		);
//		//		println!("Locks: {:#?}", Ring::locks(stash));
//		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
//		//		println!();
//	});
//}
