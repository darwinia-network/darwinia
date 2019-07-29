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

pub trait OnMinted<Balance> {
   fn on_minted(value: Balance);
}

pub trait OnAccountBalanceChanged<AccountId, Balance> {
    fn on_changed(who: &AccountId, old: Balance, new: Balance);
}

