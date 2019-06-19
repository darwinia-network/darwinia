use rstd::prelude::*;
use crate::{Trait, RewardBalanceOf, Module};
use rstd::result;
use srml_support::traits::{Currency};
use primitives::traits::{Convert, Zero, One, As, StaticLookup, CheckedSub, Saturating, Bounded};
use kton::DECIMALS;

// change when new epoch
// the total reward per era
pub fn compute_next_era_reward<T: Trait>() -> Result<RewardBalanceOf<T>, &'static str> {
        //TODO: add decimal
        //TODO: add collection of eras as a minimum set for changing session_reward
        let eras_per_epoch = <Module<T>>::era_per_epoch();
        let cap =  <RewardBalanceOf<T>>::sa(10000000000 * DECIMALS);
        let total_issuance_now = T::RewardCurrency::total_issuance();
        if let Some(surplus) = cap.checked_sub(&total_issuance_now) {
            // mint 20% of the rest
            Ok(surplus / <RewardBalanceOf<T>>::sa(5 * eras_per_epoch.as_()))
        } else {
            return Err("too large.");
        }

}

