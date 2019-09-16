/// utility in staking
use crate::{EraIndex, KtonBalanceOf, Module, RingBalanceOf, Trait};
use primitives::traits::{IntegerSquareRoot, SaturatedConversion};
use rstd::convert::TryInto;
use srml_support::traits::{Currency, Get};
use substrate_primitives::U256;

//change when new epoch
// the total reward per era
pub fn compute_current_era_reward<T: Trait + 'static>() -> RingBalanceOf<T> {
	let eras_per_epoch: RingBalanceOf<T> = <T::ErasPerEpoch as Get<EraIndex>>::get().into();
	let current_epoch: u32 = <Module<T>>::epoch_index();
	let total_left: u128 = (T::Cap::get() - T::Ring::total_issuance()).saturated_into::<u128>();
	let surplus = total_left
		- total_left * 99_u128.pow(current_epoch.integer_sqrt())
			/ 100_u128.pow(current_epoch.integer_sqrt());
	let surplus: RingBalanceOf<T> = <RingBalanceOf<T>>::saturated_from::<u128>(surplus);
	(surplus / eras_per_epoch)
}

// consistent with the formula in smart contract in evolution land which can be found in
// https://github.com/evolutionlandorg/bank/blob/master/contracts/GringottsBank.sol#L280
pub fn compute_kton_return<T: Trait + 'static>(
	value: RingBalanceOf<T>,
	months: u32,
) -> KtonBalanceOf<T> {
	let value = value.saturated_into::<u64>();
	let no = U256::from(67).pow(U256::from(months));
	let de = U256::from(66).pow(U256::from(months));

	let quotient = no / de;
	let remainder = no % de;
	let res = U256::from(value)
		* (U256::from(1000) * (quotient - 1) + U256::from(1000) * remainder / de)
		/ U256::from(1_970_000);
	res.as_u128().try_into().unwrap_or_default()
}
