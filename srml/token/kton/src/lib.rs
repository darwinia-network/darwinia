//! RING: system token of evolution land

#![cfg_attr(not(feature = "std"), no_std)]
extern crate parity_codec;
extern crate parity_codec_derive;
extern crate sr_primitives as primitives;
extern crate sr_std as rstd;
#[macro_use]
extern crate srml_support as support;
extern crate srml_system as system;
extern crate srml_timestamp as timestamp;
#[cfg(test)]
extern crate substrate_primitives;

use core::convert::TryFrom;
use evo_support::traits::SystemCurrency;
use parity_codec::{Codec, Decode, Encode, HasCompact};
#[cfg(feature = "std")]
use primitives::{Deserialize, Serialize};
use primitives::traits::{
    As, CheckedAdd, CheckedSub, MaybeSerializeDebug, Member, SimpleArithmetic,
    StaticLookup, Zero,
};
use rstd::{prelude::*, result, vec};
use runtime_io::print;
use substrate_primitives::U256;
use support::{decl_event, decl_module, decl_storage, Parameter, StorageMap, StorageValue};
use support::dispatch::Result;
use support::traits::{
    Currency, ExistenceRequirement, Imbalance, LockableCurrency, LockIdentifier, OnUnbalanced, SignedImbalance, WithdrawReason, WithdrawReasons};
use system::ensure_signed;

mod mock;
mod tests;

const DEPOSIT_ID: LockIdentifier = *b"lockkton";
const MONTH: u64 = 2592000;


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Revenue<CurrencyOf> {
    team: CurrencyOf,
    contribution: CurrencyOf,
    ktoner: CurrencyOf,
    lottery: CurrencyOf,
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct DepositInfo<Currency: HasCompact, Moment: HasCompact> {
    #[codec(compact)]
    pub month: Moment,
    #[codec(compact)]
    pub start_at: Moment,
    #[codec(compact)]
    pub value: Currency,
    pub unit_interest: u64,
    pub claimed: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Deposit<Currency: HasCompact, Moment: HasCompact> {
    #[codec(compact)]
    pub total_deposit: Currency,
    pub deposit_list: Vec<DepositInfo<Currency, Moment>>,
}


type CurrencyOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
pub type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
pub type PositiveImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

pub trait Trait: timestamp::Trait {
    /// The balance of an account.
    type Balance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64> + MaybeSerializeDebug;
    /// The token which
    type Currency: LockableCurrency<<Self as system::Trait>::AccountId, Moment=Self::Moment>;
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type SystemPayment: OnUnbalanced<NegativeImbalanceOf<Self>>;

    // recommend to keep it although unused til now
    type SystemRefund: OnUnbalanced<PositiveImbalanceOf<Self>>;
}

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
		<T as Trait>::Balance,
		Currency = CurrencyOf<T>,
		Moment = <T as timestamp::Trait>::Moment,
	{
	    /// lock ring for getting kton
	    /// Balance is for kton
	    /// Currency is for ring
		NewDeposit(u64, AccountId, Balance, Currency),
		/// Transfer succeeded (from, to, value, fees).
		TokenTransfer(AccountId, AccountId, Balance),

		WithdrawDeposit(AccountId, Currency, Moment, bool),
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

	    pub Init get(is_init): bool;

        pub Decimals get(decimals): u32;

	    pub UnitInterest get(unit_interest): u64;

	    pub DepositLedger get(deposit_ledger): map T::AccountId => Option<Deposit<CurrencyOf<T>, T::Moment>>;
		/// The total `units issued in the system.
		pub TotalIssuance get(total_issuance) : T::Balance;

		pub FreeBalance get(free_balance): map T::AccountId => T::Balance;

		pub ReservedBalance get(reserved_balance): map T::AccountId => T::Balance;

		pub Locks get(locks): map T::AccountId => Vec<BalanceLock<T::Balance, T::Moment>>;

		// reward you can get per kton
		pub RewardPerShare get(reward_per_share): CurrencyOf<T>;
		// reward already paid to each ktoner
		pub RewardPaidOut get(reward_paidout): map T::AccountId => i128;

		/// system revenue
		/// the id for evolution land is 42
		pub SysRevenue get(system_revenue): map T::AccountId => Revenue<CurrencyOf<T>>;

		pub SysAccount get(sys_account) config(): T::AccountId;

        // only for test
		pub Test: i128;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		pub fn init(origin) -> Result {
            ensure!(Self::is_init() == false, "Token already initialized.");

            let sender = ensure_signed(origin)?;

            <SysRevenue<T>>::insert(Self::sys_account(), Revenue::default());
            <Init<T>>::put(true);

            Ok(())
        }

		pub fn transfer(
			origin,
			dest: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::Balance
		) {
			let transactor = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			Self::transfer_internal(&transactor, &dest, value)?;
		}

        // @param duration - in MONTH
		pub fn deposit(origin, #[compact] value: CurrencyOf<T>, months: T::Moment) -> Result {
            let transactor = ensure_signed(origin)?;
            let free_balance = T::Currency::free_balance(&transactor);
            let value = value.min(free_balance);

            Self::update_deposit(transactor, value, months)?;

            Ok(())
		}

        fn withdraw(origin, months: T::Moment, #[compact] value: CurrencyOf<T>) -> Result {
            let transactor = ensure_signed(origin)?;
            Self::withdraw_deposit(transactor.clone(), months, value)?;

            Ok(())

        }

        pub fn transfer_to_system(origin, value: CurrencyOf<T>) -> Result {
            let transactor = ensure_signed(origin)?;

            // TODO: extend `WithdrawReason` to match system revenue model
            T::Currency::transfer(&transactor, &Self::sys_account(), value.clone())?;

            // re-balance
            Self::update_revenue(value)?;

            Ok(())

        }

        fn test(test_value: u64) -> Result {
            let test_value = i128::from(test_value);
            <Test<T>>::put(test_value);
            Ok(())
        }
    }

}


impl<T: Trait> Module<T> {


    // PUB MUTABLE

    pub fn update_revenue(value: CurrencyOf<T>) -> Result {
        let total_supply: u64 = Self::total_issuance().as_();
        let major = value.clone() * <CurrencyOf<T>>::sa(3) / <CurrencyOf<T>>::sa(10);
        let minor = value.clone() / <CurrencyOf<T>>::sa(10);

        let sys_account = Self::sys_account();

        let delta_reward_per_share = major.clone() / <CurrencyOf<T>>::sa(total_supply);

        // update reward_per_share
        <RewardPerShare<T>>::mutate(|r| *r += delta_reward_per_share);

        // update revenue
        let mut revenue = Self::system_revenue(&sys_account);
        revenue.contribution = major.clone();
        revenue.ktoner = major.clone();
        revenue.lottery = minor.clone();
        revenue.team = value - <CurrencyOf<T>>::sa(2) * major - minor;

        <SysRevenue<T>>::insert(&sys_account, revenue);

        Ok(())
    }


    // PRIVATE MUTABLES

    // IMPORTANT: we do not touch kton balance here
    // remember modify kton balance
    fn withdraw_kton_reward(who: &T::AccountId, value: T::Balance, dest: &T::AccountId) -> Result {
//        let free_balance = Self::free_balance(who);
//        let new_from_balance = match free_balance.checked_sub(&value) {
//            Some(b) => b,
//            None => return Err("from balance too low to receive value"),
//        };

        let reward_per_share = Self::reward_per_share();
        let update_paidout = <CurrencyOf<T> as As<u64>>::as_(reward_per_share) * <T::Balance as As<u64>>::as_(value);

        let paidout_who = Self::reward_paidout(who);
        <RewardPaidOut<T>>::insert(who, paidout_who - i128::from(update_paidout));

        // burn kton
        if dest == &T::AccountId::default() {
            <TotalIssuance<T>>::mutate(|t| *t = t.checked_sub(&value).unwrap());
            return Ok(());
        }

        // if dest is not default
        let paidout_dest = Self::reward_paidout(dest);
        <RewardPaidOut<T>>::insert(dest, paidout_dest + i128::from(update_paidout));

        Ok(())
    }

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

        // update reward
        Self::withdraw_kton_reward(transactor, value.clone(), &dest)?;
        Self::set_free_balance(transactor, new_from_balance);
        Self::set_free_balance(dest, new_to_balance);
        Self::deposit_event(RawEvent::TokenTransfer(transactor.clone(), dest.clone(), value));

        Ok(())
    }


    fn withdraw_deposit(who: T::AccountId, months: T::Moment, value: CurrencyOf<T>) -> Result {
        let now = timestamp::Module::<T>::get();
        let mut deposit = Self::deposit_ledger(&who).ok_or("the account has not deposited.")?;
        let mut deposit_info: DepositInfo<CurrencyOf<T>, T::Moment> = deposit.clone().deposit_list.into_iter()
            .find(|d| { d.value == value.clone() && d.month == months.clone() && d.claimed == false })
            .unwrap();
        // deposit token - ring
        let value = deposit_info.value;
        let duration = deposit_info.month.clone() * T::Moment::sa(MONTH);
        let due_time = deposit_info.start_at.clone() + duration;

        let able_to_claim = now >= due_time;

        if able_to_claim {
            deposit_info.claimed = true;
            deposit.total_deposit -= value;
        } else {
            let months_left = (now.clone() - due_time.clone()) / T::Moment::sa(MONTH);
            let kton_penalty = Self::compute_kton_balance(months_left, value.clone()) * T::Balance::sa(3);

            let free_balance = Self::free_balance(&who);
            let new_free_balance = match free_balance.checked_sub(&kton_penalty) {
                Some(b) => b,
                None => return Err("from balance too low to receive value"),
            };

            deposit_info.claimed = true;
            deposit.total_deposit -= value;
            // update reward
            Self::withdraw_kton_reward(&who, kton_penalty.clone(), &T::AccountId::default())?;
            // update kton balance
            Self::set_free_balance(&who, new_free_balance.clone());
            // update kton total issuance
            <TotalIssuance<T>>::mutate(|t| *t -= kton_penalty);
        }

        if deposit.total_deposit > <CurrencyOf<T>>::sa(0) {
            T::Currency::set_lock(DEPOSIT_ID, &who, deposit.total_deposit, T::Moment::sa(u64::max_value()), WithdrawReasons::all());
        }

        // update deposit
        <DepositLedger<T>>::insert(&who, deposit);

        Self::deposit_event(RawEvent::WithdrawDeposit(who, value, months, able_to_claim));
        Ok(())
    }

    // @param months - in month
    fn update_deposit(who: T::AccountId, value: CurrencyOf<T>, months: T::Moment) -> Result {
        let now = timestamp::Module::<T>::get();
        let unit_interest = Self::unit_interest();
        let deposit_info = DepositInfo { month: months.clone(), start_at: now, value: value, unit_interest: unit_interest, claimed: false };
        if <DepositLedger<T>>::exists(&who) {
            let mut deposit = Self::deposit_ledger(&who).ok_or("the account has not deposited")?;
            deposit.total_deposit += value;
            deposit.deposit_list.push(deposit_info);
            <DepositLedger<T>>::insert(&who, deposit);
        } else {
            <DepositLedger<T>>::insert(&who, Deposit { total_deposit: value, deposit_list: vec![deposit_info] });
        }

        let delta_balance = Self::compute_kton_balance(months.clone(), value);


        T::Currency::set_lock(DEPOSIT_ID, &who, Self::deposit_ledger(&who).unwrap().total_deposit, T::Moment::sa(u64::max_value()), WithdrawReasons::all());

        // update total_issuance
        <TotalIssuance<T>>::mutate(|t| *t += delta_balance);

        // update free_balance
        let old_free_balance = Self::free_balance(&who);
        let new_free_balance = match old_free_balance.checked_add(&delta_balance) {
            Some(b) => b,
            None => return Err("from balance too low to receive value"),
        };
        Self::set_free_balance(&who, new_free_balance);

        // update reward_paidout
        let reward_paid_old = Self::reward_paidout(&who); // i128
        let reward_per_share = i128::from(Self::reward_per_share().as_());
        let value1 = i128::from(delta_balance.as_());
        <RewardPaidOut<T>>::insert(&who, reward_paid_old + reward_per_share * value1);

        Self::deposit_event(RawEvent::NewDeposit(unit_interest, who.clone(), delta_balance, value.clone()));
        Ok(())
    }

    //TODO: check computation logic
    fn compute_kton_balance(months: T::Moment, value: CurrencyOf<T>) -> T::Balance {
        let exp_pre: u64 = <T::Moment>::as_(months.clone()); //12
        let exp = u32::try_from(exp_pre).unwrap();
        let value_pre: u64 = <CurrencyOf<T>>::as_(value.clone());
        let value = U256::from(value_pre);

        let no = U256::from(67_u128).pow(U256::from(exp)) * U256::exp10(6);
        let de = U256::from(66_u128).pow(U256::from(exp));

        let res = value * no / de;
        print(res.as_u64());
        let value = (res - U256::exp10(6) * value) / (U256::from(197) * U256::exp10(7));
        print(value.as_u64());
        T::Balance::sa(value.as_u64())
    }

    fn update_paidout(who: &T::AccountId, value: CurrencyOf<T>, is_refund: bool) {
        let value: i128 = i128::from(<CurrencyOf<T>>::as_(value));
        let reward_paidout = Self::reward_paidout(who);
        if is_refund {
            <RewardPaidOut<T>>::insert(who, reward_paidout - value);
        } else {
            <RewardPaidOut<T>>::insert(who, reward_paidout + value);
        }
    }

    fn set_free_balance(who: &T::AccountId, balance: T::Balance) {
        <FreeBalance<T>>::insert(who, balance);
    }
}

impl<T: Trait> SystemCurrency<T::AccountId> for Module<T> {
    type CurrencyOf = CurrencyOf<T>;
    type PositiveImbalance = PositiveImbalanceOf<T>;
    type NegativeImbalance = NegativeImbalanceOf<T>;

    fn reward_can_withdraw(who: &T::AccountId) -> i128 {
        let kton_balance: u64 = T::Balance::as_(Self::free_balance(who));
        let paid_out = Self::reward_paidout(who);
        let reward_per_share = Self::CurrencyOf::as_(Self::reward_per_share());
        let should_withdraw = i128::from(reward_per_share * kton_balance) - paid_out;
        should_withdraw
    }

    fn system_withdraw(
        who: &T::AccountId,
        value: Self::CurrencyOf,
    ) -> result::Result<(Self::NegativeImbalance, Self::NegativeImbalance), &'static str> {
        // can_withdraw_value must at least be 0.
        let can_withdraw_value = u64::try_from(Self::reward_can_withdraw(who)).unwrap_or_else(|_| Zero::zero());

        let mut system_imbalance = Self::NegativeImbalance::zero();
        let mut acc_imbalance = Self::NegativeImbalance::zero();

        let withdraw_value = value.min(Self::CurrencyOf::sa(can_withdraw_value));

        if withdraw_value > Self::CurrencyOf::sa(0) {
            let paidout_new = match Self::reward_paidout(who).checked_add(i128::from(Self::CurrencyOf::as_(withdraw_value.clone()))) {
                Some(v) => v,
                None => return Err("wrong with paidout."),
            };

            <RewardPaidOut<T>>::insert(who, paidout_new);
            system_imbalance = T::Currency::slash(&Self::sys_account(), withdraw_value).0;
        }

        if value > withdraw_value {
            let new_value = value - withdraw_value;
            acc_imbalance = T::Currency::withdraw(
                who,
                new_value,
                WithdrawReason::Fee,
                ExistenceRequirement::KeepAlive)?;
        }

        Ok((system_imbalance, acc_imbalance))
    }

    fn system_refund(
        who: &T::AccountId,
        value: Self::CurrencyOf,
        system_imbalance: Self::NegativeImbalance,
        acc_imbalance: Self::NegativeImbalance,
    ) {
        let acc_imbalance_in_currency = acc_imbalance.peek();
//        let (should_return_to_acc, _) = acc_imbalance.split(value.clone());
        let mut signed_imbalance = Self::PositiveImbalance::zero();

        let should_return_to_acc = value.min(acc_imbalance_in_currency);
        let free_balance = T::Currency::free_balance(who);
        // value first returns to acc
        signed_imbalance = T::Currency::deposit_creating(who, should_return_to_acc);


        // if there is surplus then return to the system
        if value > acc_imbalance_in_currency {
            let surplus = value - acc_imbalance_in_currency;
            Self::update_paidout(who, surplus, true);
            let sys_account = Self::sys_account();
            let system_refund_imbalance: Self::PositiveImbalance = T::Currency::deposit_creating(&sys_account, surplus);
            signed_imbalance.subsume(system_refund_imbalance);
        }

        let total_negative = acc_imbalance.merge(system_imbalance);

        if let Ok(imbalance) = total_negative.offset(signed_imbalance) {
            T::SystemPayment::on_unbalanced(imbalance);
        }


//        let free_balacne_old = T::Currency::free_balance(who);
//        let (signed_imbalance, _) = T::Currency::make_free_balance_be(who, free_balacne_old + value);
//
//        let imbalance = SignedImbalance::Negative(imbalance);
//
//        let signed_imbalance = signed_imbalance.merge(imbalance);
//
//        let total_imbalance = if let SignedImbalance::Negative(n) = signed_imbalance {
//            n
//        } else {
//            <NegativeImbalanceOf<T>>::zero()
//        };
//
//        T::SystemPayment::on_unbalanced(total_imbalance)
    }
}
