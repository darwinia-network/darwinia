use rstd::prelude::*;
use crate::{Trait, RewardBalanceOf, Module};
use rstd::result;
use srml_support::traits::{Currency};
use primitives::traits::{Convert, Zero, One, As, StaticLookup, CheckedSub, Saturating, Bounded};
use kton::DECIMALS;

// change when new era
pub fn compute_next_session_reward<T: Trait>(validator_count: u64
) -> Result<RewardBalanceOf<T>, &'static str> {
        //TODO: add decimal
        //TODO: add collection of eras as a minimum set for changing session_reward
        let epoch_length = <Module<T>>::era_per_epoch();
        let era_length = <Module<T>>::sessions_per_era();
        let sessions_per_era = era_length * epoch_length;
        let cap =  <RewardBalanceOf<T>>::sa(10000000000 * DECIMALS);
        let total_issuance_now = T::RewardCurrency::total_issuance();
        if let Some(surplus) = cap.checked_sub(&total_issuance_now) {
            // mint 20% of the rest
            Ok(surplus / (<RewardBalanceOf<T> as As<u64>>::sa(5 * validator_count) * <RewardBalanceOf<T>>::sa(sessions_per_era.as_())))
        } else {
            return Err("too large.");
        }

}