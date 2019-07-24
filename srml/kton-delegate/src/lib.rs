#![cfg_attr(not(feature = "std"), no_std)]

use rstd::result;
use srml_support::{decl_module, decl_storage};
use srml_support::dispatch::Result;
use srml_support::traits::{
    Currency, ExistenceRequirement,
    SignedImbalance, UpdateBalanceOutcome,
    WithdrawReason,
};
// use system::ensure_signed;
use dsupport::traits::OnAccountBalanceChanged;

pub trait Trait: balances::Trait<balances::Instance1> {
    type OnAccountBalanceChanged: OnAccountBalanceChanged<Self::AccountId, Self::Balance>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Kton {
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    }
}

impl<T: Trait> Currency<T::AccountId> for Module<T> {
    type Balance = T::Balance;
    type PositiveImbalance = balances::PositiveImbalance<T, balances::Instance1>;
    type NegativeImbalance = balances::NegativeImbalance<T, balances::Instance1>;

    fn total_balance(who: &T::AccountId) -> Self::Balance {
        <balances::Module<T, balances::Instance1>>::free_balance(who)
    }

    fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
        <balances::Module<T, balances::Instance1>>::can_slash(who, value)
    }

    fn total_issuance() -> Self::Balance {
        <balances::Module<T, balances::Instance1>>::total_issuance()
    }

    fn minimum_balance() -> Self::Balance {
        <balances::Module<T, balances::Instance1>>::minimum_balance()
    }

    fn free_balance(who: &T::AccountId) -> Self::Balance {
        <balances::Module<T, balances::Instance1>>::free_balance(who)
    }

    fn burn(amount: Self::Balance) -> Self::PositiveImbalance {
        <balances::Module<T, balances::Instance1>>::burn(amount)
    }

    fn issue(amount: Self::Balance) -> Self::NegativeImbalance {
        <balances::Module<T, balances::Instance1>>::issue(amount)
    }

    fn ensure_can_withdraw(
        who: &T::AccountId,
        amount: T::Balance,
        reason: WithdrawReason,
        new_balance: T::Balance,
        ) -> Result {
        <balances::Module<T, balances::Instance1>>::ensure_can_withdraw(who, amount, reason, new_balance)
    }


    fn transfer(transactor: &T::AccountId, dest: &T::AccountId, value: Self::Balance) -> Result {
        let old_from_balance = Self::free_balance(transactor);
        let old_to_balance   = Self::free_balance(dest);
        let new_from_balance = old_from_balance - value;
        let new_to_balance   = old_to_balance + value;
        T::OnAccountBalanceChanged::on_changed(transactor, old_from_balance, new_from_balance);
        T::OnAccountBalanceChanged::on_changed(dest, old_to_balance, new_to_balance);
        
        <balances::Module<T, balances::Instance1> as Currency<T::AccountId>>::transfer(transactor, dest, value)
    }

    fn withdraw(
        who: &T::AccountId,
        value: Self::Balance,
        reason: WithdrawReason,
        liveness: ExistenceRequirement,
        ) -> result::Result<Self::NegativeImbalance, &'static str> {
        let old_balance = Self::free_balance(who);
        let new_balance = old_balance - value;
        T::OnAccountBalanceChanged::on_changed(who, old_balance, new_balance);

        <balances::Module<T, balances::Instance1>>::withdraw(who, value, reason, liveness)
    }

    fn slash(
        who: &T::AccountId,
        value: Self::Balance
    ) -> (Self::NegativeImbalance, Self::Balance) {
        let old_balance = Self::free_balance(who);
        let new_balance = old_balance - value;
        T::OnAccountBalanceChanged::on_changed(who, old_balance, new_balance);

        <balances::Module<T, balances::Instance1>>::slash(who, value)
    }

    fn deposit_creating(
        who: &T::AccountId,
        value: Self::Balance,

        ) -> Self::PositiveImbalance {
        let old_balance = Self::free_balance(who);
        T::OnAccountBalanceChanged::on_changed(who, old_balance, value);

        <balances::Module<T, balances::Instance1>>::deposit_creating(who, value)
    }

    fn make_free_balance_be(who: &T::AccountId, balance: Self::Balance) -> (
        SignedImbalance<Self::Balance, Self::PositiveImbalance>,
        UpdateBalanceOutcome
    ) {
        <balances::Module<T, balances::Instance1>>::make_free_balance_be(who, balance)
    }

    fn deposit_into_existing(who: &T::AccountId, value: Self::Balance) -> 
        result::Result<Self::PositiveImbalance, &'static str> 
        {
            let old_balance = Self::free_balance(who);
            let new_balance = old_balance + value;
            T::OnAccountBalanceChanged::on_changed(who, old_balance, new_balance);
            <balances::Module<T, balances::Instance1>>::deposit_into_existing(who, value)
        }

}
