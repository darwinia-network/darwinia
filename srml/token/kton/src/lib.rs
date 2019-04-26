//! RING: system token of evolution land

#![cfg_attr(not(feature = "std"), no_std)]
extern crate parity_codec;
extern crate parity_codec_derive;


extern crate sr_primitives as primitives;
extern crate sr_std as rstd;
#[macro_use]
extern crate srml_support;
extern crate srml_system as system;
extern crate srml_timestamp as timestamp;
#[cfg(test)]
extern crate substrate_primitives;
#[cfg(feature = "std")]
use primitives::{Serialize, Deserialize};
use parity_codec::{Codec, Decode, Encode, HasCompact};
use primitives::traits::{
    As, CheckedAdd, CheckedSub, MaybeSerializeDebug, Member, SimpleArithmetic,
    StaticLookup, Zero,
};
use rstd::{prelude::*, vec};
use srml_support::{decl_event, decl_module, decl_storage, Parameter, StorageMap, StorageValue};
use srml_support::dispatch::Result;
use srml_support::traits::{
    Currency, LockableCurrency, LockIdentifier, WithdrawReasons};
use system::{ensure_signed};

const DEPOSIT_ID: LockIdentifier = *b"lockkton";
const Month: u64 = 2592000;


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct DepositInfo<Currency: Default, Moment: Default> {
    pub month: Moment,
    pub start_at: Moment,
    pub value: Currency,
    pub unit_interest: u64,
    pub claimed: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Deposit<Currency: Default, Moment: Default> {
    pub total_deposit: Currency,
    pub deposit_list: Vec<DepositInfo<Currency, Moment>>,
}

type CurrencyOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as system::Trait>::AccountId;

pub trait Trait: timestamp::Trait {
    /// The balance of an account.
    type Balance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64> + MaybeSerializeDebug;
    /// The token which
    type Currency: LockableCurrency<<Self as system::Trait>::AccountId, Moment=Self::Moment>;
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
		<T as Trait>::Balance,
		Currency = CurrencyOf<T>,
	{
	    /// lock ring for getting kton
	    /// Balance is for kton
	    /// Currency is for ring
		NewDeposit(u64, AccountId, Balance, Currency),
		/// Transfer succeeded (from, to, value, fees).
		TokenTransfer(AccountId, AccountId, Balance),
	}
);

/// Struct to encode the vesting schedule of an individual account.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct VestingSchedule<Balance> {
    /// Locked amount at genesis.
    pub offset: Balance,
    /// Amount that gets unlocked every block from genesis.
    pub per_block: Balance,
}

impl<Balance: SimpleArithmetic + Copy + As<u64>> VestingSchedule<Balance> {
    /// Amount locked at block `n`.
    pub fn locked_at<BlockNumber: As<u64>>(&self, n: BlockNumber) -> Balance {
        if let Some(x) = Balance::sa(n.as_()).checked_mul(&self.per_block) {
            self.offset.max(x) - x
        } else {
            Zero::zero()
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BalanceLock<Balance, BlockNumber> {
    pub id: LockIdentifier,
    pub amount: Balance,
    pub until: BlockNumber,
    pub reasons: WithdrawReasons,
}

decl_storage! {
	trait Store for Module<T: Trait> as KtonBalances {
	    pub UnitInterest get(unit_interest): u64;

	    pub DepositLedger get(deposit_ledger): map AccountIdOf<T> => Deposit<CurrencyOf<T>, T::Moment>;
		/// The total `units issued in the system.
		pub TotalIssuance get(total_issuance) : T::Balance;

		pub FreeBalance get(free_balance): map AccountIdOf<T> => T::Balance;

		pub ReservedBalance get(reserved_balance): map AccountIdOf<T> => T::Balance;

		pub Locks get(locks): map AccountIdOf<T> => Vec<BalanceLock<T::Balance, T::Moment>>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		pub fn transfer(
			origin,
			dest: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::Balance
		) {
			let transactor = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			Self::transfer_internal(&transactor, &dest, value)?;
		}

		fn deposit(origin, value: CurrencyOf<T>, duration: T::Moment) -> Result {
            let transactor = ensure_signed(origin)?;
            let free_balance = T::Currency::free_balance(&transactor);
            let value = value.min(free_balance);

            Self::update_deposit(transactor, value, duration);

            Ok(())
		}

        fn withdraw(origin, months: T::Moment, value: CurrencyOf<T>) -> Result {
            let transactor = ensure_signed(origin)?;
            Self::withdraw_deposit(transactor.clone(), months, value);
            Ok(())
        }
    }


}

impl<T: Trait> Module<T> {
    fn transfer_internal(transactor: &T::AccountId, dest: &T::AccountId, value: T::Balance) -> Result {
        let from_balance = Self::free_balance(transactor);
        let to_balance = Self::free_balance(dest);

        let new_from_balance = match from_balance.checked_sub(&value) {
            Some(b) => b,
            None => return Err("from balance too low to receive value"),
        };

        let new_to_balance = match to_balance.checked_add(&value) {
            Some(b) => b,
            None => return Err("destination balance too high to receive value"),
        };


        Self::deposit_event(RawEvent::TokenTransfer(transactor.clone(), dest.clone(), value));

        Ok(())
    }

    // PRIVATE MUTABLES


    fn withdraw_deposit(who: AccountIdOf<T>, months: T::Moment, value: CurrencyOf<T>) -> Result {
        let now = timestamp::Module::<T>::get();
        let mut deposit = Self::deposit_ledger(&who);
        let mut deposit_info : DepositInfo<CurrencyOf<T>, T::Moment> = deposit.deposit_list.into_iter()
            .find(|d| {d.value == value.clone() && d.month == months.clone() && d.claimed == false})
            .unwrap();
        // deposit token - ring
        let value = deposit_info.value;
        let duration = deposit_info.month.clone() * T::Moment::sa(Month);
        let due_time = deposit_info.start_at.clone() + duration;
        let total_deposit = deposit.total_deposit;

        if now >= due_time {
            deposit_info.claimed = true;
            T::Currency::set_lock(DEPOSIT_ID, &who, total_deposit - value.clone(), T::Moment::sa(u64::max_value()), WithdrawReasons::all());
        } else {
            let months_left = (now.clone() - due_time.clone()) / T::Moment::sa(Month);
            let kton_penalty = Self::compute_kton_balance(months_left, value.clone()) * T::Balance::sa(3);

            let free_balance = Self::free_balance(&who);
            let new_free_balance = match free_balance.checked_sub(&kton_penalty) {
                Some(b) => b,
                None => return Err("from balance too low to receive value"),
            };
            deposit_info.claimed = true;
            Self::set_free_balance(&who, new_free_balance.clone());
        }

        Ok(())
    }

    fn update_deposit(who: AccountIdOf<T>, value: CurrencyOf<T>, months: T::Moment) -> Result {
        let duration = months.clone() * T::Moment::sa(Month);
        let now = timestamp::Module::<T>::get();
        let unit_interest = Self::unit_interest();
        let deposit_info = DepositInfo { month: months.clone(), start_at: now, value: value, unit_interest: unit_interest, claimed: false };
        if <DepositLedger<T>>::exists(who.clone()) {
            let mut deposit = Self::deposit_ledger(&who);
            deposit.total_deposit += value;
            deposit.deposit_list.push(deposit_info.clone());
        } else {
            <DepositLedger<T>>::insert(&who, Deposit {total_deposit:value, deposit_list: vec![deposit_info] });
        }

        let new_balance = Self::compute_kton_balance(months.clone(), value.clone());

        let balance_now = Self::free_balance(&who);
        let new_balance = match new_balance.checked_add(&balance_now) {
            Some(b) => b,
            None => return Err("got overflow after adding a fee to value"),
        };

        T::Currency::set_lock(DEPOSIT_ID, &who, Self::deposit_ledger(&who).total_deposit, T::Moment::sa(u64::max_value()), WithdrawReasons::all());

        Self::set_free_balance(&who, new_balance);

        Self::deposit_event(RawEvent::NewDeposit(unit_interest, who.clone(), new_balance.clone(), value.clone()));
        Ok(())
    }

    fn compute_kton_balance(month: T::Moment, value: CurrencyOf<T>) -> T::Balance {
        let res = <CurrencyOf<T> as As<u64>>::as_(value.clone()) * 67_u64 ^ (<T::Moment as As<u64>>::as_(month.clone())) / (66_u64 ^ (<T::Moment as As<u64>>::as_(month)));
        let value = (res - <CurrencyOf<T> as As<u64>>::as_(value)) / 1970;
        T::Balance::sa(value)
    }

    fn set_free_balance(who: &AccountIdOf<T>, balance: T::Balance) {
        <FreeBalance<T>>::insert(who, balance);
    }
}
