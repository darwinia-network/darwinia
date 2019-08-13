
/// utility in staking
use crate::{Trait, ErasNums, RingBalanceOf, KtonBalanceOf};
use srml_support::traits::{Currency, Get};
use primitives::traits::{ CheckedSub,SaturatedConversion };
use substrate_primitives::U256;
use rstd::convert::TryInto;


//change when new epoch
// the total reward per era
pub fn compute_current_era_reward<T: Trait + 'static>() -> Result<RingBalanceOf<T>, &'static str> {
    //TODO: add decimal
    //TODO: add collection of eras as a minimum set for changing session_reward
    let eras_per_epoch = <T::ErasPerEpoch as Get<ErasNums>>::get();
    let cap =  T::Cap::get();
    let total_issuance_now = T::Ring::total_issuance();
    if let Some(surplus) = cap.checked_sub(&total_issuance_now) {
        // mint 20% of the rest
        Ok(surplus / (5 * eras_per_epoch).into())
    } else {
        return Err("too large.");
    }
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
