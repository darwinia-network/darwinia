#![cfg_attr(not(feature = "std"), no_std)]

use parity_codec::{Decode, Encode};
use primitives::traits::{
    CheckedSub, Zero, Bounded
};

use rstd::prelude::*;
use rstd::convert::TryInto;
use srml_support::{decl_event, decl_module, decl_storage, StorageMap, ensure};
use srml_support::traits::{
    Currency, LockableCurrency, LockIdentifier, WithdrawReasons,
};
use substrate_primitives::U256;
use system::ensure_signed;

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
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Gringotts {
        pub DepositLedger get(deposit_ledger): map T::AccountId => Option<Deposit<RingBalanceOf<T>, KtonBalanceOf<T>, T::Moment>>;
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

        pub fn deposit_extra(origin, additional_value: RingBalanceOf<T>, months: u32) {
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

    }
}

impl<T: Trait> Module<T> {

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

}

