use sp_core::U256;
use sp_runtime::{traits::SaturatedConversion, Perquintill};
use sp_std::convert::TryInto;

use crate::{KtonBalance, Power, RingBalance, Trait, TS};

// power is a mixture of ring and kton
// power = ring_ratio * POWER_COUNT / 2 + kton_ratio * POWER_COUNT / 2
pub fn compute_balance_power<S: TryInto<u128>>(active: S, pool: S) -> Power {
	const HALF_POWER_COUNT: Power = 1_000_000_000 / 2;

	Perquintill::from_rational_approximation(active.saturated_into::<Power>(), pool.saturated_into::<Power>().max(1))
		* HALF_POWER_COUNT
}

// TODO
pub fn compute_total_payout<N>() -> (N, N)
where
	N: Clone + Default,
{
	(Default::default(), Default::default())
}

// consistent with the formula in smart contract in evolution land which can be found in
// https://github.com/evolutionlandorg/bank/blob/master/contracts/GringottsBank.sol#L280
pub fn compute_kton_return<T: Trait>(value: RingBalance<T>, months: TS) -> KtonBalance<T> {
	let value = value.saturated_into::<u64>();
	let no = U256::from(67).pow(U256::from(months));
	let de = U256::from(66).pow(U256::from(months));

	let quotient = no / de;
	let remainder = no % de;
	let res = U256::from(value) * (U256::from(1000) * (quotient - 1) + U256::from(1000) * remainder / de)
		/ U256::from(1_970_000);
	res.as_u128().try_into().unwrap_or_default()
}
