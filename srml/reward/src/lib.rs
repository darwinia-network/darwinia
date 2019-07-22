#![cfg_attr(not(feature = "std"), no_std)]

use parity_codec::{Codec, Decode, Encode};
use primitives::traits::{
    CheckedAdd, CheckedSub, MaybeSerializeDebug, Member, Saturating, SimpleArithmetic,
    StaticLookup, Zero, Bounded
};

use rstd::prelude::*;
use rstd::{cmp, result, convert::{ TryInto, TryFrom}};
use srml_support::{decl_event, decl_module, decl_storage, Parameter, StorageMap, StorageValue, ensure};
use srml_support::dispatch::Result;
use srml_support::traits::{
    Currency, ExistenceRequirement, Imbalance, LockableCurrency, LockIdentifier,
    OnUnbalanced, SignedImbalance, UpdateBalanceOutcome,
    WithdrawReason, WithdrawReasons,
};
use substrate_primitives::U256;
use system::ensure_signed;
use dsupport::traits::OnAccountBalanceChanged;
use dsupport::traits::OnDilution;

type KtonBalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::Balance;
type RingBalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;

pub type RingNegativeImbalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
pub type RingPositiveImbalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

pub trait Trait: timestamp::Trait {
    type Kton: Currency<Self::AccountId>;
    type Ring: Currency<Self::AccountId>;

    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T> where
    < T as system::Trait>::AccountId,
    Currency = RingBalanceOf<T>,
    {
        /// Claim Reward
        RewardClaim(AccountId, Currency),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Reward {
        // reward you can get per kton
        pub RewardPerShare get(reward_per_share): RingBalanceOf<T>;
        // reward already paid to each ktoner
        pub RewardPaidOut get(reward_paid_out): map T::AccountId => i128;

        pub SysAcc get(sys_acc) config(): T::AccountId;

        /// system revenue
        /// same to balance in ring
        /// TODO: it's ugly, ready for hacking
        pub SysRevenuePot get(system_revenue): map T::AccountId => RingBalanceOf<T>;
    }
    add_extra_genesis {}
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        pub fn claim_reward(origin) {
            let transactor = ensure_signed(origin)?;
            let value_can_withdraw = Self::reward_can_withdraw(&transactor);
            if !value_can_withdraw.is_zero() {
                Self::update_reward_paid_out(&transactor, value_can_withdraw);
                T::Ring::transfer(&Self::sys_acc(), &transactor, value_can_withdraw)?;
                Self::deposit_event(RawEvent::RewardClaim(transactor, value_can_withdraw));
            }
        }
    }
}

impl<T: Trait> Module<T> {

    fn convert_to_paid_out(value: KtonBalanceOf<T>) -> RingBalanceOf<T> {
        let value: u64 = value.try_into().unwrap_or_default() as u64;
        let additional_reward_paid_out: RingBalanceOf<T> = Self::reward_per_share() *  value.try_into().unwrap_or_default();
        additional_reward_paid_out
    }

    /// update one's reward_paid_out
    fn update_reward_paid_out(who: &T::AccountId, value: RingBalanceOf<T>) {
        let value = i128::from(value.try_into().unwrap_or_default() as u64);
        let reward_paid_out = Self::reward_paid_out(who);
        <RewardPaidOut<T>>::insert(who, reward_paid_out + value);
    }

    pub fn reward_to_pot(value: RingBalanceOf<T>) {
        let sys_acc = Self::sys_acc();
        let positive = T::Ring::deposit_creating(&sys_acc, value);

        // update reward-per-share
        let total_issuance: u64 = T::Kton::total_issuance().try_into().unwrap_or_default() as u64;
        //TODO: if kton total_issuance is super high
        // this will be zero
        let additional_reward_per_share = value / total_issuance.try_into().unwrap_or_default();
        <RewardPerShare<T>>::mutate(|r| *r += additional_reward_per_share);

        <SysRevenuePot<T>>::insert(&sys_acc, Self::system_revenue(&sys_acc) + value);
    }

    // PUB IMMUTABLE
    fn reward_can_withdraw(who: &T::AccountId) -> RingBalanceOf<T> {
        let free_balance = T::Kton::free_balance(who);
        let max_should_withdraw = Self::convert_to_paid_out(free_balance);
        let max_should_withdraw: u64  = max_should_withdraw.try_into().unwrap_or_default() as u64;
        let should_withdraw = i128::from(max_should_withdraw) - Self::reward_paid_out(who);
        if should_withdraw <= 0 {
            0.into()
        } else {
            u64::try_from(should_withdraw).unwrap_or_default().try_into().unwrap_or_default()
        }

    }

    /// pay system fee with reward
    fn withdraw_from_sys_reward(who: &T::AccountId, value: RingBalanceOf<T>)
        -> result::Result<(RingNegativeImbalanceOf<T>, RingNegativeImbalanceOf<T>), &'static str> {

            let can_withdraw_value = Self::reward_can_withdraw(who);

            let mut system_imbalance = RingNegativeImbalanceOf::<T>::zero();
            let mut acc_imbalance = RingNegativeImbalanceOf::<T>::zero();

            let withdraw_value = value.min(can_withdraw_value);

            if withdraw_value > 0.into() {
                let paid_out_new = match Self::reward_paid_out(who).checked_add(i128::from(withdraw_value.try_into().unwrap_or_default() as u64)) {
                    Some(v) => v,
                    None => return Err("wrong with paidout"),
                };

                <RewardPaidOut<T>>::insert(who, paid_out_new);
                system_imbalance = T::Ring::slash(&Self::sys_acc(), withdraw_value).0;
            }

            if value > withdraw_value {
                let new_value = value - withdraw_value;
                acc_imbalance = T::Ring::withdraw(
                    who,
                    new_value,
                    WithdrawReason::Fee,
                    ExistenceRequirement::KeepAlive)?;
            }

            Ok((system_imbalance, acc_imbalance))
        }

}

/// account kton balance changed
impl<T: Trait> OnAccountBalanceChanged<T::AccountId, KtonBalanceOf<T>> for Module<T> {
    fn on_changed(who: &T::AccountId, old: KtonBalanceOf<T>, new: KtonBalanceOf<T>) {
        // update reward paid out
        let additional_reward_paid_out = Self::convert_to_paid_out(new-old);
        Self::update_reward_paid_out(who, additional_reward_paid_out);
    }
}

/// reward(ring minted)
impl<T: Trait> OnDilution<RingBalanceOf<T>> for Module<T> {
    fn on_dilution(minted: RingBalanceOf<T>) {
        Self::reward_to_pot(minted);
    }
}
