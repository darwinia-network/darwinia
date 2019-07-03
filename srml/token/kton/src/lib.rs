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

// customed
use dsupport::traits::SystemCurrency;
mod imbalance;
use imbalance::{NegativeImbalance, PositiveImbalance};

mod mock;
mod tests;

const DEPOSIT_ID: LockIdentifier = *b"lockkton";
const DECIMALS: u64 = 1000000000;

/// Struct to encode the vesting schedule of an individual account.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct VestingSchedule<Balance> {
    /// Locked amount at genesis.
    pub offset: Balance,
    /// Amount that gets unlocked every block from genesis.
    pub per_block: Balance,
}

impl<Balance: SimpleArithmetic + Copy> VestingSchedule<Balance> {
    /// Amount locked at block `n`.
    pub fn locked_at<BlockNumber>(&self, n: BlockNumber) -> Balance
        where Balance: From<BlockNumber>
    {
        if let Some(x) = Balance::from(n).checked_mul(&self.per_block) {
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


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct IndividualDeposit<Currency, Moment> {
    pub month: Moment,
    pub start_at: Moment,
    pub value: Currency,
    pub claimed: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Deposit<Currency, Moment> {
    pub total: Currency,
    pub deposit_list: Vec<IndividualDeposit<Currency, Moment>>,
}

type CurrencyOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
pub type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
pub type PositiveImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

pub trait Trait: timestamp::Trait {
    type Balance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy +
    MaybeSerializeDebug + From<Self::BlockNumber>;

    type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;

    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    // kton
    type OnMinted: OnUnbalanced<PositiveImbalance<Self>>;
    type OnRemoval: OnUnbalanced<NegativeImbalance<Self>>;

    // ring
    type SystemRefund: OnUnbalanced<PositiveImbalanceOf<Self>>;
}

decl_event!(
    pub enum Event<T> where
        < T as system::Trait>::AccountId,
        < T as Trait>::Balance,
        Currency = CurrencyOf<T>,
        Moment = < T as timestamp::Trait>::Moment,
    {
        /// lock ring for getting kton
        /// Balance is for kton
        /// Currency is for ring
        NewDeposit(Moment, AccountId, Balance, Currency),
        /// Transfer succeeded (from, to, value, fees).
        TokenTransfer(AccountId, AccountId, Balance),
        /// Claim Reward
        RewardClaim(AccountId, Currency),
        WithdrawDeposit(AccountId, Currency, Moment, bool),
    }
);


decl_storage! {
	trait Store for Module<T: Trait> as KtonBalances {

	    pub DepositLedger get(deposit_ledger): map T::AccountId => Option<Deposit<CurrencyOf<T>, T::Moment>>;

		// reward you can get per kton
		pub RewardPerShare get(reward_per_share): CurrencyOf<T>;
		// reward already paid to each ktoner
		pub RewardPaidOut get(reward_paid_out): map T::AccountId => i128;

		pub SysAcc get(sys_acc) config(): T::AccountId;

		/// system revenue
		/// same to balance in ring
		/// TODO: it's ugly, ready for hacking
		pub SysRevenuePot get(system_revenue): map T::AccountId => CurrencyOf<T>;

        /// For Currency and LockableCurrency Trait
		/// The total `units issued in the system.
		// like `existential_deposit`, but always set to 0
		pub MinimumBalance get(minimum_balance): T::Balance = 0.into();

		pub TotalIssuance get(total_issuance) build(|config: &GenesisConfig<T>| {
			config.balances.iter().fold(Zero::zero(), |acc: T::Balance, &(_, n)| acc + n)
		}): T::Balance;

		pub FreeBalance get(free_balance) build(|config: &GenesisConfig<T>| config.balances.clone()): map T::AccountId => T::Balance;

		pub ReservedBalance get(reserved_balance): map T::AccountId => T::Balance;

		pub Locks get(locks): map T::AccountId => Vec<BalanceLock<T::Balance, T::BlockNumber>>;

		pub TotalLock get(total_lock): T::Balance;

		pub Vesting get(vesting) build(|config: &GenesisConfig<T>| {
			config.vesting.iter().filter_map(|&(ref who, begin, length)| {
				let begin = <T::Balance as From<T::BlockNumber>>::from(begin);
				let length = <T::Balance as From<T::BlockNumber>>::from(length);

				config.balances.iter()
					.find(|&&(ref w, _)| w == who)
					.map(|&(_, balance)| {
						// <= begin it should be >= balance
						// >= begin+length it should be <= 0

						let per_block = balance / length.max(primitives::traits::One::one());
						let offset = begin * per_block + balance;

						(who.clone(), VestingSchedule { offset, per_block })
					})
			}).collect::<Vec<_>>()
		}): map T::AccountId => Option<VestingSchedule<T::Balance>>;
	}
	add_extra_genesis {
		config(balances): Vec<(T::AccountId, T::Balance)>;
		config(vesting): Vec<(T::AccountId, T::BlockNumber, T::BlockNumber)>;
	}
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;


        fn deposit(origin, value: CurrencyOf<T>, months: T::Moment) {
            ensure!(!months.is_zero() && months <= 36.into(), "months must be at least 1");
            let transactor = ensure_signed(origin)?;
            if <DepositLedger<T>>::exists(&transactor) {
                return Err("Already deposited.");
            }

            let free_currency = T::Currency::free_balance(&transactor);
            let value = value.min(free_currency);

            let now = <timestamp::Module<T>>::now();

            let individual_deposit = IndividualDeposit {month: months.clone(), start_at: now.clone(), value: value, claimed: false};
            let deposit = Deposit {total: value, deposit_list: vec![individual_deposit]};

            Self::update_deposit(&transactor, &deposit);

            let kton_return = Self::compute_kton_balance(months, value).unwrap();
            let positive_imbalance = Self::deposit_creating(&transactor, kton_return);
            T::OnMinted::on_unbalanced(positive_imbalance);
            Self::deposit_event(RawEvent::NewDeposit(now, transactor, kton_return, value));
        }


        fn deposit_extra(origin, additional_value: CurrencyOf<T>, months: T::Moment) {
             ensure!(!months.is_zero() && months <= 36.into(), "months must be at least 1");
             let transactor = ensure_signed(origin)?;
             let mut deposit = Self::deposit_ledger(&transactor).ok_or("Use fn deposit instead.")?;

             let now = <timestamp::Module<T>>::now();
             let free_currency = T::Currency::free_balance(&transactor);

             if let Some(extra) = free_currency.checked_sub(&deposit.total) {
                 let extra = extra.min(additional_value);
                 deposit.total += extra;
                 let individual_deposit = IndividualDeposit {month: months.clone(), start_at: now.clone(), value: extra.clone(), claimed: false};
                 deposit.deposit_list.push(individual_deposit);
                 Self::update_deposit(&transactor, &deposit);

                 let kton_return = Self::compute_kton_balance(months, extra).unwrap();
                 let positive_imbalance = Self::deposit_creating(&transactor, kton_return);
                 T::OnMinted::on_unbalanced(positive_imbalance);
                 Self::deposit_event(RawEvent::NewDeposit(now, transactor, kton_return, extra));
             }
        }


        pub fn transfer(origin,
            dest: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::Balance
		) {
			let transactor = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;

            <Self as Currency<_>>::transfer(&transactor, &dest, value)?;

        }

        pub fn claim_reward(origin) {
            let transactor = ensure_signed(origin)?;
            let value_can_withdraw = Self::reward_can_withdraw(&transactor);
            if !value_can_withdraw.is_zero() {
                Self::update_reward_paid_out(&transactor, value_can_withdraw, false);
                T::Currency::transfer(&Self::sys_acc(), &transactor, value_can_withdraw);
                Self::deposit_event(RawEvent::RewardClaim(transactor, value_can_withdraw));
            }
        }


    }



}

impl<T: Trait> Module<T> {

    fn update_deposit(who: &T::AccountId, deposit: &Deposit<CurrencyOf<T>, T::Moment>) {
        T::Currency::set_lock(
            DEPOSIT_ID,
            &who,
            deposit.total,
            // u32::max_value().into(),
            T::BlockNumber::max_value(),
            WithdrawReasons::all()
        );
        <DepositLedger<T>>::insert(who, deposit);
    }


    fn convert_to_paid_out(value: T::Balance) -> CurrencyOf<T> {
        let value: u64 = value.try_into().unwrap_or_default() as u64;
        let additional_reward_paid_out: CurrencyOf<T> = Self::reward_per_share() *  value.try_into().unwrap_or_default();
        additional_reward_paid_out
    }

    fn compute_kton_balance(months: T::Moment, value: CurrencyOf<T>) -> Option<T::Balance> {
        let months = months.try_into().unwrap_or_default() as u64;
        let value = value.try_into().unwrap_or_default() as u64;
        if !months.is_zero() {
            let no = U256::from(67_u128).pow(U256::from(months.clone())) * U256::exp10(6);
            let de = U256::from(66_u128).pow(U256::from(months));

            let res: U256 = U256::from(value) * no / de;
            let value = (res - U256::exp10(6) * value) / (U256::from(197) * U256::exp10(7));
            Some(value.as_u64().try_into().unwrap_or_default())
        } else {
            None
        }

    }

    pub fn vesting_balance(who: &T::AccountId) -> T::Balance {
        if let Some(v) = Self::vesting(who) {
            Self::free_balance(who)
                .min(v.locked_at::<T::BlockNumber>(<system::Module<T>>::block_number()))
        } else {
            Zero::zero()
        }
    }


    // PRIVATE MUTABLE
    // NOTE: different from balacnes module
    fn set_free_balance(who: &T::AccountId, balance: T::Balance) -> UpdateBalanceOutcome {
        //TODO: check the value of balance, but no ensure!(...)
        <FreeBalance<T>>::insert(who, balance);
        UpdateBalanceOutcome::Updated
    }

    fn set_reserved_balance(who: &T::AccountId, balance: T::Balance) -> UpdateBalanceOutcome {
        <ReservedBalance<T>>::insert(who, balance);
        UpdateBalanceOutcome::Updated
    }

    /// update one's reward_paid_out
    /// is_refund true -, means giving out reward
    /// is_refund false +
    fn update_reward_paid_out(who: &T::AccountId, value: CurrencyOf<T>, is_refund: bool) {
        let value = i128::from(value.try_into().unwrap_or_default() as u64);
        let reward_paid_out = Self::reward_paid_out(who);
        if is_refund {
            <RewardPaidOut<T>>::insert(who, reward_paid_out - value);
        } else {
            <RewardPaidOut<T>>::insert(who, reward_paid_out + value);
        }
    }
}


impl<T: Trait> Currency<T::AccountId> for Module<T> {
    type Balance = T::Balance;
    type PositiveImbalance = PositiveImbalance<T>;
    type NegativeImbalance = NegativeImbalance<T>;

    fn total_balance(who: &T::AccountId) -> Self::Balance {
        Self::free_balance(who) + Self::reserved_balance(who)
    }

    fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
        Self::free_balance(who) >= value
    }

    fn total_issuance() -> Self::Balance {
        Self::total_issuance()
    }

    fn minimum_balance() -> Self::Balance {
        Self::minimum_balance()
    }

    fn free_balance(who: &T::AccountId) -> Self::Balance {
        <FreeBalance<T>>::get(who)
    }

    fn ensure_can_withdraw(
        who: &T::AccountId,
        _amount: T::Balance,
        reason: WithdrawReason,
        new_balance: T::Balance,
    ) -> Result {
        match reason {
            WithdrawReason::Reserve | WithdrawReason::Transfer if Self::vesting_balance(who) > new_balance =>
                return Err("vesting balance too high to send value"),
            _ => {}
        }
        let locks = Self::locks(who);
        if locks.is_empty() {
            return Ok(())
        }

        let now = <system::Module<T>>::block_number();
        if locks.into_iter()
            .all(|l|
                now >= l.until
                    || new_balance >= l.amount
                    || !l.reasons.contains(reason)
            )
        {
            Ok(())
        } else {
            Err("account liquidity restrictions prevent withdrawal")
        }
    }


    // TODO: add fee
    fn transfer(transactor: &T::AccountId, dest: &T::AccountId, value: Self::Balance) -> Result {

        let from_balance = Self::free_balance(transactor);
        let to_balance = Self::free_balance(dest);

        let new_from_balance = match from_balance.checked_sub(&value) {
            None => return Err("balance too low to send value"),
            Some(b) => b,
        };

        Self::ensure_can_withdraw(transactor, value, WithdrawReason::Transfer, new_from_balance)?;

        // NOTE: total stake being stored in the same type means that this could never overflow
        // but better to be safe than sorry.
        let new_to_balance = match to_balance.checked_add(&value) {
            Some(b) => b,
            None => return Err("destination balance too high to receive value"),
        };

        if transactor != dest {
            Self::set_free_balance(transactor, new_from_balance);
            Self::set_free_balance(dest, new_to_balance);
            // settle transactor reward
            let from_should_withdraw = Self::reward_can_withdraw(transactor);
            Self::update_reward_paid_out(transactor, from_should_withdraw, true);
            // settle dest reward
            let to_should_not_withdraw = Self::convert_to_paid_out(value);
            Self::update_reward_paid_out(dest, to_should_not_withdraw, false);
            Self::deposit_event(RawEvent:: TokenTransfer(transactor.clone(), dest.clone(), value));
        }

        Ok(())
    }


    fn withdraw(
        who: &T::AccountId,
        value: Self::Balance,
        reason: WithdrawReason,
        liveness: ExistenceRequirement,
    ) -> result::Result<Self::NegativeImbalance, &'static str> {
        if let Some(new_balance) = Self::free_balance(who).checked_sub(&value) {
            if liveness == ExistenceRequirement::KeepAlive && new_balance < Self::minimum_balance() {
                return Err("payment would kill account")
            }
            Self::ensure_can_withdraw(who, value, reason, new_balance)?;
            Self::set_free_balance(who, new_balance);
            Ok(NegativeImbalance::new(value))
        } else {
            Err("too few free funds in account")
        }

    }


    fn slash(
        who: &T::AccountId,
        value: Self::Balance
    ) -> (Self::NegativeImbalance, Self::Balance) {
        let free_balance = Self::free_balance(who);
        let free_slash = cmp::min(free_balance, value);
        Self::set_free_balance(who, free_balance - free_slash);
        let remaining_slash = value - free_slash;

        if !remaining_slash.is_zero() {
            let reserved_balance = Self::reserved_balance(who);
            let reserved_slash = cmp::min(reserved_balance, remaining_slash);
            Self::set_reserved_balance(who, reserved_balance - reserved_slash);
            (NegativeImbalance::new(free_slash + reserved_slash), remaining_slash - reserved_slash)
        } else {
            (NegativeImbalance::new(value), Zero::zero())
        }
    }

    fn deposit_into_existing(
        who: &T::AccountId,
        value: Self::Balance
    ) -> result::Result<Self::PositiveImbalance, &'static str> {
        if Self::total_balance(who).is_zero() {
            return Err("beneficiary account must pre-exist");
        }
        let additional_reward_paid_out = Self::convert_to_paid_out(value);
        Self::update_reward_paid_out(who, additional_reward_paid_out, false);
        Self::set_free_balance(who, Self::free_balance(who) + value);
        Ok(PositiveImbalance::new(value))
    }

    fn deposit_creating(
        who: &T::AccountId,
        value: Self::Balance,
    ) -> Self::PositiveImbalance {

        let (imbalance, _) = Self::make_free_balance_be(who, Self::free_balance(who) + value);

        if let SignedImbalance::Positive(p) = imbalance {
            p
        } else {
            // Impossible, but be defensive.
            Self::PositiveImbalance::zero()
        }
    }

    fn make_free_balance_be(who: &T::AccountId, balance: Self::Balance) -> (
        SignedImbalance<Self::Balance, Self::PositiveImbalance>,
        UpdateBalanceOutcome
    ) {
        let original = Self::free_balance(who);

        let imbalance = if original <= balance {
            // update reward paid out
            let additional_reward_paid_out = Self::convert_to_paid_out(balance);
            Self::update_reward_paid_out(who, additional_reward_paid_out, false);

            SignedImbalance::Positive(PositiveImbalance::new(balance - original))
        } else {
            SignedImbalance::Negative(NegativeImbalance::new(original - balance))
        };

        let outcome = {
            Self::set_free_balance(who, balance);
            UpdateBalanceOutcome::Updated
        };

        (imbalance, outcome)
    }

    fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
        <TotalIssuance<T>>::mutate(|issued|
            issued.checked_sub(&amount).unwrap_or_else(|| {
                amount = *issued;
                Zero::zero()
            })
        );
        PositiveImbalance::new(amount)
    }


    fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
        <TotalIssuance<T>>::mutate(|issued|
            *issued = issued.checked_add(&amount).unwrap_or_else(|| {
                amount = Self::Balance::max_value() - *issued;
                Self::Balance::max_value()
            })
        );
        NegativeImbalance::new(amount)
    }

}


impl<T: Trait> LockableCurrency<T::AccountId> for Module<T>
    where
        T::Balance: MaybeSerializeDebug
{
    type Moment = T::BlockNumber;

    fn set_lock(
        id: LockIdentifier,
        who: &T::AccountId,
        amount: T::Balance,
        until: T::BlockNumber,
        reasons: WithdrawReasons,
    ) {
        let now = <system::Module<T>>::block_number();
        let mut new_lock = Some(BalanceLock { id, amount, until, reasons });
        let mut locks = Self::locks(who).into_iter().filter_map(|l|
            if l.id == id {
                new_lock.take()
            } else if l.until > now {
                Some(l)
            } else {
                None
            }).collect::<Vec<_>>();
        if let Some(lock) = new_lock {
            locks.push(lock)
        }
        <Locks<T>>::insert(who, locks);
    }

    fn extend_lock(
        id: LockIdentifier,
        who: &T::AccountId,
        amount: T::Balance,
        until: T::BlockNumber,
        reasons: WithdrawReasons,
    ) {
        let now = <system::Module<T>>::block_number();
        let mut new_lock = Some(BalanceLock { id, amount, until, reasons });
        let mut locks = Self::locks(who).into_iter().filter_map(|l|
            if l.id == id {
                new_lock.take().map(|nl| {
                    BalanceLock {
                        id: l.id,
                        amount: l.amount.max(nl.amount),
                        until: l.until.max(nl.until),
                        reasons: l.reasons | nl.reasons,
                    }
                })
            } else if l.until > now {
                Some(l)
            } else {
                None
            }).collect::<Vec<_>>();
        if let Some(lock) = new_lock {
            locks.push(lock)
        }
        <Locks<T>>::insert(who, locks);
    }

    fn remove_lock(
        id: LockIdentifier,
        who: &T::AccountId,
    ) {
        let now = <system::Module<T>>::block_number();
        let locks = Self::locks(who).into_iter().filter_map(|l|
            if l.until > now && l.id != id {
                Some(l)
            } else {
                None
            }).collect::<Vec<_>>();
        <Locks<T>>::insert(who, locks);
    }
}

impl<T: Trait> SystemCurrency<T::AccountId, CurrencyOf<T>> for Module<T> {
    // all of ring
//    type CurrencyOf = CurrencyOf<T>;
    type PositiveImbalanceOf = PositiveImbalanceOf<T>;
    type NegativeImbalanceOf = NegativeImbalanceOf<T>;

    fn reward_to_pot(value: CurrencyOf<T>) {
        let sys_acc = Self::sys_acc();
        let positive = T::Currency::deposit_creating(&sys_acc, value);

        // update reward-per-share
        let total_issuance: u64 = Self::total_issuance().try_into().unwrap_or_default() as u64;
        //TODO: if kton total_issuance is super high
        // this will be zero
        let additional_reward_per_share = value / total_issuance.try_into().unwrap_or_default();
        <RewardPerShare<T>>::mutate(|r| *r += additional_reward_per_share);

        <SysRevenuePot<T>>::insert(&sys_acc, Self::system_revenue(&sys_acc) + value);

        // re-balance
        T::SystemRefund::on_unbalanced(positive);
    }


    // PUB IMMUTABLE
    fn reward_can_withdraw(who: &T::AccountId) -> CurrencyOf<T> {
        let free_balance = Self::free_balance(who);
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
    fn withdraw_from_sys_reward(
        who: &T::AccountId,
        value: CurrencyOf<T>)
        -> result::Result<(Self::NegativeImbalanceOf, Self::NegativeImbalanceOf), &'static str> {

        let can_withdraw_value = Self::reward_can_withdraw(who);

        let mut system_imbalance = Self::NegativeImbalanceOf::zero();
        let mut acc_imbalance = Self::NegativeImbalanceOf::zero();

        let withdraw_value = value.min(can_withdraw_value);

        if withdraw_value > 0.into() {
            let paid_out_new = match Self::reward_paid_out(who).checked_add(i128::from(withdraw_value.try_into().unwrap_or_default() as u64)) {
                Some(v) => v,
                None => return Err("wrong with paidout"),
            };

            <RewardPaidOut<T>>::insert(who, paid_out_new);
            system_imbalance = T::Currency::slash(&Self::sys_acc(), withdraw_value).0;
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


}

