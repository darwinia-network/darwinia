//! Balances pallet benchmarking.

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, Dispatchable};

use crate::Module as Balances;

const SEED: u32 = 0;
const MAX_EXISTENTIAL_DEPOSIT: u32 = 1000;
const MAX_USER_INDEX: u32 = 1000;

benchmarks! {
	_ {
		let e in 2 .. MAX_EXISTENTIAL_DEPOSIT => ();
		let u in 1 .. MAX_USER_INDEX => ();
	}

	// Benchmark `transfer` extrinsic with the worst possible conditions:
	// * Transfer will kill the sender account.
	// * Transfer will create the recipient account.
	transfer {
		let u in ...;
		let e in ...;

		let existential_deposit = T::ExistentialDeposit::get();
		let caller = account("caller", u, SEED);

		// Give some multiple of the existential deposit + creation fee + transfer fee
		let balance = existential_deposit.saturating_mul(e.into());
		let _ = <Balances<T> as Currency<_>>::make_free_balance_be(&caller, balance);

		// Transfer `e - 1` existential deposits + 1 unit, which guarantees to create one account, and reap this user.
		let recipient = account("recipient", u, SEED);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient);
		let transfer_amount = existential_deposit.saturating_mul((e - 1).into()) + 1.into();
	}: _(RawOrigin::Signed(caller), recipient_lookup, transfer_amount)

	// Benchmark `transfer` with the best possible condition:
	// * Both accounts exist and will continue to exist.
	transfer_best_case {
		let u in ...;
		let e in ...;

		let caller = account("caller", u, SEED);
		let recipient: T::AccountId = account("recipient", u, SEED);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());

		// Give the sender account max funds for transfer (their account will never reasonably be killed).
		let _ = <Balances<T> as Currency<_>>::make_free_balance_be(&caller, T::Balance::max_value());

		// Give the recipient account existential deposit (thus their account already exists).
		let existential_deposit = T::ExistentialDeposit::get();
		let _ = <Balances<T> as Currency<_>>::make_free_balance_be(&recipient, existential_deposit);
		let transfer_amount = existential_deposit.saturating_mul(e.into());
	}: transfer(RawOrigin::Signed(caller), recipient_lookup, transfer_amount)

	// Benchmark `transfer_keep_alive` with the worst possible condition:
	// * The recipient account is created.
	transfer_keep_alive {
		let u in ...;
		let e in ...;

		let caller = account("caller", u, SEED);
		let recipient = account("recipient", u, SEED);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient);

		// Give the sender account max funds, thus a transfer will not kill account.
		let _ = <Balances<T> as Currency<_>>::make_free_balance_be(&caller, T::Balance::max_value());
		let existential_deposit = T::ExistentialDeposit::get();
		let transfer_amount = existential_deposit.saturating_mul(e.into());
	}: _(RawOrigin::Signed(caller), recipient_lookup, transfer_amount)

	// Benchmark `set_balance` coming from ROOT account. This always creates an account.
	set_balance {
		let u in ...;
		let e in ...;

		let user: T::AccountId = account("user", u, SEED);
		let user_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(user.clone());

		// Give the user some initial balance.
		let existential_deposit = T::ExistentialDeposit::get();
		let balance_amount = existential_deposit.saturating_mul(e.into());
		let _ = <Balances<T> as Currency<_>>::make_free_balance_be(&user, balance_amount);
	}: _(RawOrigin::Root, user_lookup, balance_amount, balance_amount)

	// Benchmark `set_balance` coming from ROOT account. This always kills an account.
	set_balance_killing {
		let u in ...;
		let e in ...;

		let user: T::AccountId = account("user", u, SEED);
		let user_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(user.clone());

		// Give the user some initial balance.
		let existential_deposit = T::ExistentialDeposit::get();
		let balance_amount = existential_deposit.saturating_mul(e.into());
		let _ = <Balances<T> as Currency<_>>::make_free_balance_be(&user, balance_amount);
	}: set_balance(RawOrigin::Root, user_lookup, 0.into(), 0.into())
}
