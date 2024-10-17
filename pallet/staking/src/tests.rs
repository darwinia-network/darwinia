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
use frame_support::{assert_noop, assert_ok};
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
fn collator_caches_should_work() {
	ExtBuilder.build().execute_with(|| {
		assert!(call_on_cache!(<Previous<Runtime>>::get()).unwrap().is_empty());
		assert!(call_on_cache!(<Current<Runtime>>::get()).unwrap().is_empty());
		assert_eq!(
			call_on_cache!(<Next<Runtime>>::get()).unwrap(),
			vec![AccountId(1), AccountId(2), AccountId(3)]
		);
		assert_eq!(
			<CacheStates<Runtime>>::get(),
			(CacheState::Previous, CacheState::Current, CacheState::Next)
		);

		Staking::shift_cache_states();

		assert!(call_on_cache!(<Previous<Runtime>>::get()).unwrap().is_empty());
		assert_eq!(
			call_on_cache!(<Current<Runtime>>::get()).unwrap(),
			vec![AccountId(1), AccountId(2), AccountId(3)]
		);
		assert!(call_on_cache!(<Next<Runtime>>::get()).unwrap().is_empty());
		assert_eq!(
			<CacheStates<Runtime>>::get(),
			(CacheState::Next, CacheState::Previous, CacheState::Current)
		);

		Staking::shift_cache_states();

		assert_eq!(
			call_on_cache!(<Previous<Runtime>>::get()).unwrap(),
			vec![AccountId(1), AccountId(2), AccountId(3)]
		);
		assert!(call_on_cache!(<Current<Runtime>>::get()).unwrap().is_empty());
		assert!(call_on_cache!(<Next<Runtime>>::get()).unwrap().is_empty());
		assert_eq!(
			<CacheStates<Runtime>>::get(),
			(CacheState::Current, CacheState::Next, CacheState::Previous)
		);

		Staking::shift_cache_states();

		assert!(call_on_cache!(<Previous<Runtime>>::get()).unwrap().is_empty());
		assert!(call_on_cache!(<Current<Runtime>>::get()).unwrap().is_empty());
		assert_eq!(
			call_on_cache!(<Next<Runtime>>::get()).unwrap(),
			vec![AccountId(1), AccountId(2), AccountId(3)]
		);
		assert_eq!(
			<CacheStates<Runtime>>::get(),
			(CacheState::Previous, CacheState::Current, CacheState::Next)
		);
	});
}

#[test]
fn elect_should_work() {
	ExtBuilder.build().execute_with(|| {
		assert_eq!(
			call_on_cache!(<Next<Runtime>>::get()).unwrap(),
			vec![AccountId(1), AccountId(2), AccountId(3)]
		);

		NEXT_COLLATOR_ID.with(|v| *v.borrow_mut() = 4);
		new_session();

		assert_eq!(
			call_on_cache!(<Next<Runtime>>::get()).unwrap(),
			vec![AccountId(4), AccountId(5), AccountId(6)]
		);
	});
}

#[test]
fn auto_payout_should_work() {
	ExtBuilder.build().execute_with(|| {
		Efflux::block(1);

		(1..=3).for_each(|i| <PendingRewards<Runtime>>::insert(AccountId(i), i as Balance));

		System::reset_events();
		Efflux::block(1);
		dbg!(<PendingRewards<Runtime>>::iter().collect::<Vec<_>>());
		assert_eq!(events(), vec![Event::Payout { who: AccountId(2), amount: 2 }]);

		System::reset_events();
		Efflux::block(1);
		dbg!(<PendingRewards<Runtime>>::iter().collect::<Vec<_>>());
		assert_eq!(events(), vec![Event::Payout { who: AccountId(3), amount: 3 }]);

		System::reset_events();
		Efflux::block(1);
		dbg!(<PendingRewards<Runtime>>::iter().collect::<Vec<_>>());
		assert_eq!(events(), vec![Event::Payout { who: AccountId(1), amount: 1 }]);
	});
}

#[test]
fn on_new_session_should_work() {
	ExtBuilder.build().execute_with(|| {});
}

#[test]
fn unstake_all_for_should_work() {
	ExtBuilder.build().execute_with(|| {
		assert_noop!(
			Staking::unstake_all_for(RuntimeOrigin::signed(AccountId(1)), AccountId(1)),
			<Error<Runtime>>::NoRecord
		);

		<Ledgers<Runtime>>::insert(AccountId(1), Ledger { ring: 1, deposits: Default::default() });
		assert_ok!(Staking::unstake_all_for(RuntimeOrigin::signed(AccountId(1)), AccountId(1)));
	});
}

#[test]
fn payout_for_should_work() {
	ExtBuilder.build().execute_with(|| {});
}

#[test]
fn set_collator_count_should_work() {
	ExtBuilder.build().execute_with(|| {
		assert_noop!(
			Staking::set_collator_count(RuntimeOrigin::signed(AccountId(1)), 1),
			DispatchError::BadOrigin
		);
		assert_noop!(
			Staking::set_collator_count(RuntimeOrigin::root(), 0),
			<Error<Runtime>>::ZeroCollatorCount
		);

		assert_ok!(Staking::set_collator_count(RuntimeOrigin::root(), 1));
		assert_eq!(Staking::collator_count(), 1);
	});
}
