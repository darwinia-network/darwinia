use super::*;
use mock::{ExtBuilder, Kton, Origin, System};
use runtime_io::with_externalities;
use srml_support::traits::{Currency, LockIdentifier, WithdrawReason, WithdrawReasons};
use srml_support::{assert_err, assert_noop, assert_ok};

const ID_1: LockIdentifier = *b"1       ";
const ID_2: LockIdentifier = *b"2       ";
const ID_3: LockIdentifier = *b"3       ";

#[test]
fn transfer_should_work() {
	with_externalities(
		&mut ExtBuilder::default().existential_deposit(0).build(),
		|| {
			//
			Kton::deposit_creating(&1001, 100);
			assert_err!(
				Kton::transfer(Origin::signed(1001), 1000, 500),
				"balance too low to send value"
			);
			assert_eq!(Kton::free_balance(&1000), 0);
		},
	);
}

#[test]
fn lock_should_work() {
	with_externalities(
		&mut ExtBuilder::default().existential_deposit(0).build(),
		|| {
			Kton::deposit_creating(&1001, 100);
			Kton::set_lock(ID_1, &1001, 90, u64::max_value(), WithdrawReasons::all());
			assert_err!(
				Kton::transfer(Origin::signed(1001), 1000, 20),
				"account liquidity restrictions prevent withdrawal"
			);
		},
	);
}

#[test]
fn lock_removal_should_work() {
	with_externalities(
		&mut ExtBuilder::default().existential_deposit(0).build(),
		|| {
			Kton::deposit_creating(&1001, 10);
			Kton::set_lock(
				ID_1,
				&1001,
				u64::max_value(),
				u64::max_value(),
				WithdrawReasons::all(),
			);
			Kton::remove_lock(ID_1, &1001);
			assert_ok!(<Kton as Currency<_>>::transfer(&1001, &1002, 1));
		},
	);
}

#[test]
fn lock_replacement_should_work() {
	with_externalities(
		&mut ExtBuilder::default().existential_deposit(0).build(),
		|| {
			Kton::deposit_creating(&1001, 10);
			Kton::set_lock(
				ID_1,
				&1001,
				u64::max_value(),
				u64::max_value(),
				WithdrawReasons::all(),
			);
			Kton::set_lock(ID_1, &1001, 5, u64::max_value(), WithdrawReasons::all());
			assert_ok!(<Kton as Currency<_>>::transfer(&1001, &1002, 1));
		},
	);
}

#[test]
fn double_locking_should_work() {
	with_externalities(
		&mut ExtBuilder::default().existential_deposit(0).build(),
		|| {
			Kton::deposit_creating(&1001, 10);
			Kton::set_lock(ID_1, &1001, 5, u64::max_value(), WithdrawReasons::all());
			Kton::set_lock(ID_2, &1001, 5, u64::max_value(), WithdrawReasons::all());
			assert_ok!(<Kton as Currency<_>>::transfer(&1001, &1002, 1));
		},
	);
}

#[test]
fn combination_locking_should_work() {
	with_externalities(
		&mut ExtBuilder::default().existential_deposit(0).build(),
		|| {
			Kton::deposit_creating(&1001, 10);
			Kton::set_lock(ID_1, &1001, u64::max_value(), 0, WithdrawReasons::none());
			Kton::set_lock(ID_2, &1001, 0, u64::max_value(), WithdrawReasons::none());
			Kton::set_lock(ID_3, &1001, 0, 0, WithdrawReasons::all());
			assert_ok!(<Kton as Currency<_>>::transfer(&1001, &1002, 1));
		},
	);
}

#[test]
fn lock_value_extension_should_work() {
	with_externalities(
		&mut ExtBuilder::default().existential_deposit(0).build(),
		|| {
			Kton::deposit_creating(&1001, 10);
			Kton::set_lock(ID_1, &1001, 5, u64::max_value(), WithdrawReasons::all());
			assert_noop!(
				<Kton as Currency<_>>::transfer(&1001, &1002, 6),
				"account liquidity restrictions prevent withdrawal"
			);
			Kton::extend_lock(ID_1, &1001, 2, u64::max_value(), WithdrawReasons::all());
			assert_noop!(
				<Kton as Currency<_>>::transfer(&1001, &1002, 6),
				"account liquidity restrictions prevent withdrawal"
			);
			Kton::extend_lock(ID_1, &1001, 8, u64::max_value(), WithdrawReasons::all());
			assert_noop!(
				<Kton as Currency<_>>::transfer(&1001, &1002, 3),
				"account liquidity restrictions prevent withdrawal"
			);
		},
	);
}

#[test]
fn lock_block_number_should_work() {
	with_externalities(
		&mut ExtBuilder::default().existential_deposit(0).build(),
		|| {
			Kton::deposit_creating(&1001, 10);
			Kton::set_lock(ID_1, &1001, 10, 2, WithdrawReasons::all());
			assert_noop!(
				<Kton as Currency<_>>::transfer(&1001, &1002, 1),
				"account liquidity restrictions prevent withdrawal"
			);

			System::set_block_number(2);
			assert_ok!(<Kton as Currency<_>>::transfer(&1001, &1002, 1));
		},
	);
}

#[test]
fn lock_block_number_extension_should_work() {
	with_externalities(
		&mut ExtBuilder::default().existential_deposit(0).build(),
		|| {
			Kton::deposit_creating(&1001, 10);
			Kton::set_lock(ID_1, &1001, 10, 2, WithdrawReasons::all());
			assert_noop!(
				<Kton as Currency<_>>::transfer(&1001, &1002, 6),
				"account liquidity restrictions prevent withdrawal"
			);
			Kton::extend_lock(ID_1, &1001, 10, 1, WithdrawReasons::all());
			assert_noop!(
				<Kton as Currency<_>>::transfer(&1001, &1002, 6),
				"account liquidity restrictions prevent withdrawal"
			);
			System::set_block_number(2);
			Kton::extend_lock(ID_1, &1001, 10, 8, WithdrawReasons::all());
			assert_noop!(
				<Kton as Currency<_>>::transfer(&1001, &1002, 3),
				"account liquidity restrictions prevent withdrawal"
			);
		},
	);
}

#[test]
fn lock_reasons_extension_should_work() {
	with_externalities(
		&mut ExtBuilder::default().existential_deposit(0).build(),
		|| {
			Kton::deposit_creating(&1001, 10);
			Kton::set_lock(ID_1, &1001, 10, 10, WithdrawReason::Transfer.into());
			assert_noop!(
				<Kton as Currency<_>>::transfer(&1001, &1002, 6),
				"account liquidity restrictions prevent withdrawal"
			);
			Kton::extend_lock(ID_1, &1001, 10, 10, WithdrawReasons::none());
			assert_noop!(
				<Kton as Currency<_>>::transfer(&1001, &1002, 6),
				"account liquidity restrictions prevent withdrawal"
			);
			Kton::extend_lock(ID_1, &1001, 10, 10, WithdrawReason::Reserve.into());
			assert_noop!(
				<Kton as Currency<_>>::transfer(&1001, &1002, 6),
				"account liquidity restrictions prevent withdrawal"
			);
		},
	);
}

#[test]
fn balance_works() {
	with_externalities(&mut ExtBuilder::default().build(), || {
		let _ = Kton::deposit_creating(&1001, 100);
		assert_eq!(Kton::free_balance(&1001), 100);
		assert_eq!(Kton::reserved_balance(&1001), 0);
		assert_eq!(Kton::total_balance(&1001), 100);
		assert_eq!(Kton::free_balance(&1002), 0);
		assert_eq!(Kton::reserved_balance(&1002), 0);
		assert_eq!(Kton::total_balance(&1002), 0);
	});
}

#[test]
fn balance_transfer_works() {
	with_externalities(&mut ExtBuilder::default().build(), || {
		let _ = Kton::deposit_creating(&1001, 111);
		assert_ok!(Kton::transfer(Some(1001).into(), 1002, 69));
		assert_eq!(Kton::total_balance(&1001), 42);
		assert_eq!(Kton::total_balance(&1002), 69);
	});
}
