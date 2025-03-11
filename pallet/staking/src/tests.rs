// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

// darwinia
use crate::{mock::*, *};
// polkadot-sdk
use frame_support::{assert_noop, assert_ok, traits::OnIdle};
use sp_runtime::DispatchError;

#[test]
fn get_top_collators_should_work() {
	const ZERO: [u8; 20] = [0; 20];

	let data = [
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 148, 244, 240, 74, 89, 79, 214, 144, 224,
		254, 164, 111, 40, 130, 165, 178, 97, 83, 167, 47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	];
	#[allow(deprecated)]
	let function = Function {
		name: "getTopCollators".to_owned(),
		inputs: vec![Param {
			name: "k".to_owned(),
			kind: ParamType::Uint(256),
			internal_type: None,
		}],
		outputs: vec![Param {
			name: "collators".to_owned(),
			kind: ParamType::Array(Box::new(ParamType::Address)),
			internal_type: None,
		}],
		constant: None,
		state_mutability: StateMutability::View,
	};
	let output = function
		.decode_output(&data)
		.map(|o| {
			let Some(Token::Array(addrs)) = o.into_iter().next() else { return Vec::new() };

			addrs
				.into_iter()
				.filter_map(|addr| match addr {
					Token::Address(addr) if addr.0 != ZERO => Some(addr.0),
					_ => None,
				})
				.collect()
		})
		.unwrap();

	assert_eq!(
		output,
		[
			// 0x94f4f04a594fd690e0fea46f2882a5b26153a72f.
			[
				148, 244, 240, 74, 89, 79, 214, 144, 224, 254, 164, 111, 40, 130, 165, 178, 97, 83,
				167, 47
			]
		]
	);
}

#[test]
fn elect_should_work() {
	ExtBuilder.build().execute_with(|| {
		NEXT_COLLATOR_ID.with(|v| *v.borrow_mut() = 4);

		assert_eq!(
			<Runtime as Config>::RingStaking::elect(<CollatorCount<Runtime>>::get()).unwrap(),
			[AccountId(4), AccountId(5), AccountId(6)]
		);
	});
}

#[test]
fn on_idle_allocate_ring_staking_reward_should_work() {
	ExtBuilder.build().execute_with(|| {
		(1..=512).for_each(|i| <PendingRewards<Runtime>>::insert(AccountId(i), 1));

		System::reset_events();
		<Staking as OnIdle<_>>::on_idle(0, Weight::zero().add_ref_time(5));
		assert_eq!(
			events().into_iter().filter(|e| matches!(e, Event::RewardAllocated { .. })).count(),
			5
		);

		System::reset_events();
		<Staking as OnIdle<_>>::on_idle(0, Weight::MAX);
		assert_eq!(
			events().into_iter().filter(|e| matches!(e, Event::RewardAllocated { .. })).count(),
			10
		);

		System::reset_events();
		while <PendingRewards<Runtime>>::iter().count() != 0 {
			<Staking as OnIdle<_>>::on_idle(0, Weight::MAX);
		}
		assert_eq!(
			events().into_iter().filter(|e| matches!(e, Event::RewardAllocated { .. })).count(),
			497
		);
	});
}

#[test]
fn on_new_session_should_work() {
	ExtBuilder.build().execute_with(|| {
		new_session();
		new_session();

		assert_eq!(Session::validators(), [AccountId(1), AccountId(2), AccountId(3)]);

		NEXT_COLLATOR_ID.with(|v| *v.borrow_mut() = 4);
		System::reset_events();
		new_session();
		new_session();

		assert_eq!(Session::validators(), [AccountId(4), AccountId(5), AccountId(6)]);
		// payout to collators * 2 session
		//
		// 3 * 2 = 6
		assert_eq!(
			events()
				.into_iter()
				.filter(|e| matches!(e, Event::RewardAllocated { .. }))
				.collect::<Vec<_>>(),
			[
				Event::RewardAllocated { who: AccountId(3), amount: 100 },
				Event::RewardAllocated { who: AccountId(1), amount: 200 },
				Event::RewardAllocated { who: AccountId(2), amount: 200 },
				Event::RewardAllocated { who: AccountId(3), amount: 100 },
				Event::RewardAllocated { who: AccountId(1), amount: 200 },
				Event::RewardAllocated { who: AccountId(2), amount: 200 }
			]
		);
		assert_eq!(
			(1..=3).map(|who| { Balances::free_balance(AccountId(who)) }).collect::<Vec<_>>(),
			[1100, 1100, 600]
		);

		NEXT_COLLATOR_ID.with(|v| *v.borrow_mut() = 7);
		System::reset_events();
		new_session();
		new_session();

		assert_eq!(Session::validators(), [AccountId(7), AccountId(8), AccountId(9)]);
		// payout to collators * 2 session
		//
		// 3 * 2 = 6
		assert_eq!(
			events()
				.into_iter()
				.filter(|e| matches!(e, Event::RewardAllocated { .. }))
				.collect::<Vec<_>>(),
			[
				Event::RewardAllocated { who: AccountId(6), amount: 100 },
				Event::RewardAllocated { who: AccountId(5), amount: 200 },
				Event::RewardAllocated { who: AccountId(4), amount: 200 },
				Event::RewardAllocated { who: AccountId(6), amount: 100 },
				Event::RewardAllocated { who: AccountId(5), amount: 200 },
				Event::RewardAllocated { who: AccountId(4), amount: 200 }
			]
		);
		assert_eq!(
			(1..=3).map(|who| { Balances::free_balance(AccountId(who)) }).collect::<Vec<_>>(),
			[1100, 1100, 600]
		);
	});
}

#[test]
fn allocate_ring_staking_reward_of_should_work() {
	ExtBuilder.build().execute_with(|| {
		let who = AccountId(1);

		assert_noop!(
			Staking::allocate_ring_staking_reward_of(RuntimeOrigin::signed(who), who),
			<Error<Runtime>>::NoReward
		);

		<PendingRewards<Runtime>>::insert(who, 1);
		System::reset_events();

		assert_ok!(Staking::allocate_ring_staking_reward_of(RuntimeOrigin::signed(who), who));
		assert_eq!(events(), [Event::RewardAllocated { who, amount: 1 }]);
	});
}

#[test]
fn set_collator_count_should_work() {
	ExtBuilder.build().execute_with(|| {
		assert_eq!(<CollatorCount<Runtime>>::get(), 3);
		assert_noop!(
			Staking::set_collator_count(RuntimeOrigin::signed(AccountId(1)), 1),
			DispatchError::BadOrigin
		);
		assert_ok!(Staking::set_collator_count(RuntimeOrigin::root(), 0));
		assert_eq!(<CollatorCount<Runtime>>::get(), 1);

		assert_ok!(Staking::set_collator_count(RuntimeOrigin::root(), 1));
		assert_eq!(<CollatorCount<Runtime>>::get(), 1);
	});
}

#[test]
fn set_ring_staking_contract_should_work() {
	ExtBuilder.build().execute_with(|| {
		assert_noop!(
			Staking::set_ring_staking_contract(RuntimeOrigin::signed(AccountId(1)), AccountId(1)),
			DispatchError::BadOrigin
		);

		assert_ok!(Staking::set_ring_staking_contract(RuntimeOrigin::root(), AccountId(1)));
		assert_eq!(<RingStakingContract<Runtime>>::get(), Some(AccountId(1)));
	});
}

#[test]
fn set_kton_staking_contract_should_work() {
	ExtBuilder.build().execute_with(|| {
		assert_noop!(
			Staking::set_kton_staking_contract(RuntimeOrigin::signed(AccountId(1)), AccountId(1)),
			DispatchError::BadOrigin
		);

		assert_ok!(Staking::set_kton_staking_contract(RuntimeOrigin::root(), AccountId(1)));
		assert_eq!(<KtonStakingContract<Runtime>>::get(), Some(AccountId(1)));
	});
}
