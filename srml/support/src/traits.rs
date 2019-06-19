
extern crate parity_codec as codec;
extern crate sr_std as rstd;

use codec::{Codec, Encode, Decode};
use srml_support::dispatch::Result;
use srml_support::traits::{Imbalance, Currency};
use sr_primitives::traits::{
Zero, SimpleArithmetic, As, StaticLookup, Member, CheckedAdd, CheckedSub,
MaybeSerializeDebug, Saturating
};
use rstd::{prelude::*, result};
use sr_primitives::Perbill;

// general interface
pub trait SystemCurrency<AccountId> {
    // ring
    type CurrencyOf: SimpleArithmetic + Codec + Copy + MaybeSerializeDebug + Default + As<u64>;
    type PositiveImbalance: Imbalance<Self::CurrencyOf, Opposite=Self::NegativeImbalance>;
    type NegativeImbalance: Imbalance<Self::CurrencyOf, Opposite=Self::PositiveImbalance>;

    fn reward_ktoner(value: Self::CurrencyOf) -> Result;

    fn reward_can_withdraw(who: &AccountId) -> i128;

    fn system_withdraw(who: &AccountId, value: Self::CurrencyOf) -> result::Result<(Self::NegativeImbalance, Self::NegativeImbalance), &'static str>;

    fn system_refund(who: &AccountId, value: Self::CurrencyOf, system_imbalance: Self::NegativeImbalance, acc_imbalance: Self::NegativeImbalance);
}

pub trait LockRate {
    //TODO： ugly to use u64, ready for hacking
    //    type Balance: SimpleArithmetic + As<usize> + As<u64> + Codec + Copy + MaybeSerializeDebug + Default;

    fn bill_lock_rate() -> Perbill;

    fn update_total_lock(amount: u64, is_add: bool) -> Result;
}

pub trait DarwiniaDilution<Balance> {
    fn on_dilution(treasury_income: Balance);
}
