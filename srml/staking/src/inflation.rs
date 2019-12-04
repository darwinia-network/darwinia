use rstd::convert::TryInto;
use sr_primitives::{
	traits::{IntegerSquareRoot, SaturatedConversion},
	Perbill,
};
use substrate_primitives::U256;

use super::{KtonBalanceOf, RingBalanceOf, TimeStamp, Trait};

//  1 - (99 / 100) ^ sqrt(year)
// <T: Trait + 'static>() -> RingBalanceOf<T>
pub fn compute_total_payout<T: Trait>(
	era_duration: TimeStamp,
	living_time: TimeStamp,
	total_left: u128,
) -> (RingBalanceOf<T>, RingBalanceOf<T>) {
	// Milliseconds per year for the Julian year (365.25 days).
	const MILLISECONDS_PER_YEAR: TimeStamp = ((36525 * 24 * 60 * 60) / 100) * 1000;

	let year: u32 = (living_time / MILLISECONDS_PER_YEAR + 1).saturated_into::<u32>();

	let portion = Perbill::from_rational_approximation(era_duration, MILLISECONDS_PER_YEAR);

	let maximum = portion * total_left;

	let maximum = maximum - maximum * 99_u128.pow(year.integer_sqrt()) / 100_u128.pow(year.integer_sqrt());

	let payout = maximum * 1; // TODO: add treasury ratio: Perbill::from_rational_approximation(npos_token_staked, total_tokens);

	let payout: RingBalanceOf<T> = <RingBalanceOf<T>>::saturated_from::<u128>(payout);

	let maximum: RingBalanceOf<T> = <RingBalanceOf<T>>::saturated_from::<u128>(maximum);

	(payout, maximum)
}

// consistent with the formula in smart contract in evolution land which can be found in
// https://github.com/evolutionlandorg/bank/blob/master/contracts/GringottsBank.sol#L280
pub fn compute_kton_return<T: Trait>(value: RingBalanceOf<T>, months: u32) -> KtonBalanceOf<T> {
	let value = value.saturated_into::<u64>();
	let no = U256::from(67).pow(U256::from(months));
	let de = U256::from(66).pow(U256::from(months));

	let quotient = no / de;
	let remainder = no % de;
	let res = U256::from(value) * (U256::from(1000) * (quotient - 1) + U256::from(1000) * remainder / de)
		/ U256::from(1_970_000);
	res.as_u128().try_into().unwrap_or_default()
}
