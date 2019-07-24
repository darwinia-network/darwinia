use codec::{Codec, Encode, Decode};
use sr_primitives::traits::{
    MaybeSerializeDebug, SimpleArithmetic
};
use rstd::{prelude::*, result};
use srml_support::traits::{Imbalance, Currency};

//pub trait LockRate {
//    //TODO： ugly to use u64, ready for hacking
//    //    type Balance: SimpleArithmetic + As<usize> + As<u64> + Codec + Copy + MaybeSerializeDebug + Default;
//
//    fn bill_lock_rate() -> Perbill;
//
//    fn update_total_lock(amount: u64, is_add: bool) -> Result;
//}

pub trait OnDilution<Balance> {
   fn on_dilution(minted: Balance);
}

pub trait OnAccountBalanceChanged<AccountId, Balance> {
    fn on_changed(who: &AccountId, old: Balance, new: Balance);
}
