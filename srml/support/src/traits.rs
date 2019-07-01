use codec::{Codec, Encode, Decode};
use sr_primitives::traits::{
    MaybeSerializeDebug, SimpleArithmetic
};
use rstd::{prelude::*, result};
use srml_support::traits::{Imbalance, Currency};

pub trait SystemCurrency<AccountId, Currency>
 where Currency: SimpleArithmetic + Codec + Copy + MaybeSerializeDebug + Default {
    // ring
    type PositiveImbalanceOf: Imbalance<Currency, Opposite=Self::NegativeImbalanceOf>;
    type NegativeImbalanceOf: Imbalance<Currency, Opposite=Self::PositiveImbalanceOf>;

    fn reward_to_pot(value: Currency);

    fn reward_can_withdraw(who: &AccountId) -> Currency;

    fn withdraw_from_sys_reward(who: &AccountId, value: Currency) -> result::Result<(Self::NegativeImbalanceOf, Self::NegativeImbalanceOf), &'static str>;

//    fn system_refund(who: &AccountId, value: Self::CurrencyOf, system_imbalance: Self::NegativeImbalanceOf, acc_imbalance: Self::NegativeImbalanceOf);
}

//pub trait LockRate {
//    //TODO： ugly to use u64, ready for hacking
//    //    type Balance: SimpleArithmetic + As<usize> + As<u64> + Codec + Copy + MaybeSerializeDebug + Default;
//
//    fn bill_lock_rate() -> Perbill;
//
//    fn update_total_lock(amount: u64, is_add: bool) -> Result;
//}
//
//pub trait DarwiniaDilution<Balance> {
//    fn on_dilution(treasury_income: Balance);
//}
