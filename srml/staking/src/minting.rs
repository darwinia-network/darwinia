
use crate::{Trait, RewardBalanceOf, ErasNums};
use srml_support::traits::{Currency, Get};
use primitives::traits::CheckedSub;


// change when new epoch
// the total reward per era
pub fn compute_current_era_reward<T: Trait>() -> Result<RewardBalanceOf<T>, &'static str> {
    //TODO: add decimal
    //TODO: add collection of eras as a minimum set for changing session_reward
    let eras_per_epoch = <T::ErasPerEpoch as Get<ErasNums>>::get();
    let cap =  T::Cap::get();
    let total_issuance_now = T::RewardCurrency::total_issuance();
    if let Some(surplus) = cap.checked_sub(&total_issuance_now) {
        // mint 20% of the rest
        Ok(surplus / (5 * eras_per_epoch).into())
    } else {
        return Err("too large.");
    }

}

