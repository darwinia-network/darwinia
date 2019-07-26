#![cfg_attr(not(feature = "std"), no_std)]

use parity_codec::{Decode, Encode};
use primitives::traits::{
    CheckedSub, Zero, Bounded
};

use rstd::prelude::*;
use rstd::{result, convert::{ TryInto, TryFrom}};
use srml_support::{decl_event, decl_module, decl_storage, StorageMap, StorageValue, ensure};
use srml_support::traits::{
    Currency, ExistenceRequirement, Imbalance, LockableCurrency, LockIdentifier,
    WithdrawReason, WithdrawReasons,
};
use substrate_primitives::U256;
use system::ensure_signed;
use dsupport::traits::OnAccountBalanceChanged;
use dsupport::traits::OnMinted;

#[cfg(feature = "std")]
use runtime_io::with_storage;

mod mock;
mod tests;

const DEPOSIT_ID: LockIdentifier = *b"lockkton";

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BalanceLock<Balance, BlockNumber> {
    pub id: LockIdentifier,
    pub amount: Balance,
    pub until: BlockNumber,
    pub reasons: WithdrawReasons,
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct IndividualDeposit<Currency, Balance, Moment> {
    pub month: u32,
    pub start_at: Moment,
    pub value: Currency,
    pub balance: Balance,
    pub claimed: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Deposit<Currency, Balance, Moment> {
    pub total: Currency,
    pub deposit_list: Vec<IndividualDeposit<Currency, Balance, Moment>>,
}

type KtonBalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::Balance;
type RingBalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;

pub type RingNegativeImbalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
pub type RingPositiveImbalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

pub trait Trait: timestamp::Trait {
    type Kton: Currency<Self::AccountId>;
    type Ring: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;

    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T> where
    < T as system::Trait>::AccountId,
    Balance = KtonBalanceOf<T>,
    Currency = RingBalanceOf<T>,
    Moment = < T as timestamp::Trait>::Moment,
    {
        /// lock ring for getting kton
        NewDeposit(Moment, AccountId, Balance, Currency),

        /// Claim Reward
        RewardClaim(AccountId, Currency),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Reward {
        pub DepositLedger get(deposit_ledger): map T::AccountId => Option<Deposit<RingBalanceOf<T>, KtonBalanceOf<T>, T::Moment>>;

        pub SysAcc get(sys_acc) config(): T::AccountId;

        // reward you can get per kton
        pub RewardPerShare get(reward_per_share): RingBalanceOf<T>;

        // reward already paid to each ktoner
        pub RewardPaidOut get(reward_paid_out): map T::AccountId => i128;

        /// system revenue
        /// same to balance in ring
        /// TODO: it's ugly, ready for hacking
        pub SysRevenuePot get(system_revenue): map T::AccountId => RingBalanceOf<T>;


    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        pub fn deposit(origin, value: RingBalanceOf<T>, months: u32) {
            ensure!(!months.is_zero() && months <= 36, "months must be at least 1");
            let transactor = ensure_signed(origin)?;
            if <DepositLedger<T>>::exists(&transactor) {
                return Err("Already deposited.");
            }

            let free_currency = T::Ring::free_balance(&transactor);
            let value = value.min(free_currency);

            let now = <timestamp::Module<T>>::now();

            let kton_return = Self::compute_kton_balance(months, value).unwrap();

            let individual_deposit = IndividualDeposit {month: months, start_at: now.clone(), value: value, balance: kton_return, claimed: false};
            let deposit = Deposit {total: value, deposit_list: vec![individual_deposit]};

            Self::update_deposit(&transactor, &deposit);

            let _positive_imbalance = T::Kton::deposit_creating(&transactor, kton_return);
            Self::deposit_event(RawEvent::NewDeposit(now, transactor, kton_return, value));
        }

        fn deposit_extra(origin, additional_value: RingBalanceOf<T>, months: u32) {
            ensure!(!months.is_zero() && months <= 36, "months must be at least 1");
            let transactor = ensure_signed(origin)?;
            let mut deposit = Self::deposit_ledger(&transactor).ok_or("Use fn deposit instead.")?;

            let now = <timestamp::Module<T>>::now();
            let free_currency = T::Ring::free_balance(&transactor);

            if let Some(extra) = free_currency.checked_sub(&deposit.total) {
                let extra = extra.min(additional_value);
                deposit.total += extra;

                let kton_return = Self::compute_kton_balance(months, extra).unwrap();
                let individual_deposit = IndividualDeposit {month: months, start_at: now.clone(), value: extra.clone(), balance: kton_return, claimed: false};
                deposit.deposit_list.push(individual_deposit);
                Self::update_deposit(&transactor, &deposit);

                let _positive_imbalance = T::Kton::deposit_creating(&transactor, kton_return);
                Self::deposit_event(RawEvent::NewDeposit(now, transactor, kton_return, extra));
            }
        }

        pub fn claim_reward(origin) {
            let transactor = ensure_signed(origin)?;
            let value_can_withdraw = Self::reward_can_withdraw(&transactor);
            if !value_can_withdraw.is_zero() {
                Self::update_reward_paid_out(&transactor, value_can_withdraw, false);
                T::Ring::transfer(&Self::sys_acc(), &transactor, value_can_withdraw)?;
                Self::deposit_event(RawEvent::RewardClaim(transactor, value_can_withdraw));
            }
        }
    }
}

impl<T: Trait> Module<T> {
    fn update_deposit(who: &T::AccountId, deposit: &Deposit<RingBalanceOf<T>, KtonBalanceOf<T>, T::Moment>) {
        T::Ring::set_lock(
            DEPOSIT_ID,
            &who,
            deposit.total,
            // u32::max_value().into(),
            T::BlockNumber::max_value(),
            WithdrawReasons::all(),
        );
        <DepositLedger<T>>::insert(who, deposit);
    }

    fn compute_kton_balance(months: u32, value: RingBalanceOf<T>) -> Option<KtonBalanceOf<T>> {
        let months = months as u64;
        let value = value.try_into().unwrap_or_default() as u64;

        if !months.is_zero() {
            let no = U256::from(67_u128).pow(U256::from(months));
            let de = U256::from(66_u128).pow(U256::from(months));

            let quotient = no / de;
            let remainder = no % de;
            let res = U256::from(value) * (U256::from(1000) * (quotient - 1) + U256::from(1000) * remainder / de) / U256::from(1970000);

            Some(res.as_u64().try_into().unwrap_or_default())
        } else {
            None
        }
    }

    fn convert_to_paid_out(value: KtonBalanceOf<T>) -> RingBalanceOf<T> {
        let value: u64 = value.try_into().unwrap_or_default() as u64;
        let additional_reward_paid_out: RingBalanceOf<T> = Self::reward_per_share() * value.try_into().unwrap_or_default();
        additional_reward_paid_out
    }

    /// update one's reward_paid_out
    fn update_reward_paid_out(who: &T::AccountId, value: RingBalanceOf<T>, is_refund: bool) {
        let value = i128::from(value.try_into().unwrap_or_default() as u64);
        let reward_paid_out = Self::reward_paid_out(who);
        if is_refund {
            <RewardPaidOut<T>>::insert(who, reward_paid_out - value);
        } else {
            <RewardPaidOut<T>>::insert(who, reward_paid_out + value);
        }
    }

    fn reward_to_pot(value: RingBalanceOf<T>) {
        let sys_acc = Self::sys_acc();
        let _positive = T::Ring::deposit_creating(&sys_acc, value);

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
        let max_should_withdraw: u64 = max_should_withdraw.try_into().unwrap_or_default() as u64;
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

/// reward(ring minted)
impl<T: Trait> OnMinted<RingBalanceOf<T>> for Module<T> {
    fn on_minted(value: RingBalanceOf<T>) {
        Self::reward_to_pot(value);
    }
}

/// account kton balance changed
impl<T: Trait> OnAccountBalanceChanged<T::AccountId, KtonBalanceOf<T>> for Module<T> {
    fn on_changed(who: &T::AccountId, old: KtonBalanceOf<T>, new: KtonBalanceOf<T>) {
        // update reward paid out
        if old <= new {
            let additional_reward_paid_out = Self::convert_to_paid_out(new-old);
            Self::update_reward_paid_out(who, additional_reward_paid_out, false);
        } else {
            let additional_reward_paid_out = Self::convert_to_paid_out(old-new);
            Self::update_reward_paid_out(who, additional_reward_paid_out, true);
        }
    }
}
