
/// utility in staking
use crate::{Trait, ErasNums, Module, RingBalanceOf, KtonBalanceOf};
use srml_support::traits::{Currency, Get};
use primitives::traits::{ CheckedSub,SaturatedConversion, IntegerSquareRoot, Convert };
use substrate_primitives::U256;
use rstd::convert::TryInto;

//change when new epoch
// the total reward per era
pub fn compute_current_era_reward<T: Trait + 'static>() -> RingBalanceOf<T> {
    //TODO: add decimal
    //TODO: add collection of eras as a minimum set for changing session_reward
    let eras_per_epoch = <T::ErasPerEpoch as Get<ErasNums>>::get() as u128;
    let current_epoch: u32 = <Module<T>>::epoch_index().try_into().unwrap_or_default() as u32;
    let total_left: u128 = (T::Cap::get() - T::Ring::total_issuance()).try_into().unwrap_or_default() as u128;
    let surplus = U256::from(total_left) - U256::from(total_left * 99 * current_epoch.integer_sqrt() as u128) / U256::from(current_epoch.integer_sqrt() as u128 * 100);
    let surplus = surplus.as_u128();
    (surplus / eras_per_epoch).try_into().unwrap_or_default()
}


pub fn compute_kton_return<T: Trait + 'static>(value: RingBalanceOf<T>, months: u32) -> KtonBalanceOf<T> {
    let value = value.saturated_into::<u64>();
    let no = U256::from(67).pow(U256::from(months));
    let de = U256::from(66).pow(U256::from(months));

    let quotient = no / de;
    let remainder = no % de;
    let res = U256::from(value) * (U256::from(1000) * (quotient - 1) + U256::from(1000) * remainder / de) / U256::from(1970000);
    res.as_u128().try_into().unwrap_or_default()



}
