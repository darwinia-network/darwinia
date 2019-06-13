use rstd::prelude::*;
use crate::{Trait, RewardBalanceOf, Module};
use rstd::result;
use srml_support::traits::{Currency};
use primitives::traits::{Convert, Zero, One, As, StaticLookup, CheckedSub, Saturating, Bounded};

pub const YEAR: u64 = 31536000;


// change when new era
pub fn compute_next_session_reward<T: Trait>(
) -> Result<RewardBalanceOf<T>, &'static str> {
        //TODO: add decimal
        //TODO: add collection of eras as a minimum set for changing session_reward
        let epoch_length = <Module<T>>::era_per_epoch();
        let era_length = <Module<T>>::sessions_per_era();
        let session_number = era_length * epoch_length;
        let cap : RewardBalanceOf<T> = As::sa(10000000000_u64);
        let total_issuance_now = T::RewardCurrency::total_issuance();
        if let Some(surplus) = cap.checked_sub(&total_issuance_now) {
            Ok(surplus / (<RewardBalanceOf<T> as As<u64>>::sa(5_u64) * <RewardBalanceOf<T>>::sa(session_number.as_())))
        } else {
            return Err("too large.");
        }

}