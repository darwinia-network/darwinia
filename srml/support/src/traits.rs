
extern crate parity_codec as codec;
extern crate sr_std as rstd;

use codec::{Codec, Encode, Decode};
use srml_support::dispatch::Result;
use srml_support::traits::Imbalance;
use sr_primitives::traits::{
Zero, SimpleArithmetic, As, StaticLookup, Member, CheckedAdd, CheckedSub,
MaybeSerializeDebug, Saturating
};
use rstd::{prelude::*, result};

// general interface
pub trait SystemCurrency<AccountId> {
    // ring
    type CurrencyOf: SimpleArithmetic + Codec + Copy + MaybeSerializeDebug + Default;
    type PositiveImbalance: Imbalance<Self::CurrencyOf, Opposite=Self::NegativeImbalance>;
    type NegativeImbalance: Imbalance<Self::CurrencyOf, Opposite=Self::PositiveImbalance>;

    fn reward_can_withdraw(who: &AccountId) -> i128;

    fn system_withdraw(who: &AccountId, value: Self::CurrencyOf) -> result::Result<(Self::NegativeImbalance, Self::NegativeImbalance), &'static str>;

    fn system_refund(who: &AccountId, value: Self::CurrencyOf, system_imbalance: Self::NegativeImbalance, acc_imbalance: Self::NegativeImbalance);
}