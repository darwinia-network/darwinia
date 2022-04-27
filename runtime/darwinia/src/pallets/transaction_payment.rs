// --- crates.io ---
use smallvec::smallvec;
// --- paritytech ---
use frame_support::weights::{
	WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
use pallet_transaction_payment::{Config, CurrencyAdapter};
use sp_runtime::Perbill;
// --- darwinia-network ---
use crate::*;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - [0, MAXIMUM_BLOCK_WEIGHT]
///   - [Balance::min, Balance::max]
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;

	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in Darwinia, extrinsic base weight (smallest non-zero weight) is mapped to 10 MILLI:
		let p = 10 * MILLI;
		let q = Balance::from(ExtrinsicBaseWeight::get());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

frame_support::parameter_types! {
	pub const TransactionByteFee: Balance = 50 * MICRO;
	/// This value increases the priority of `Operational` transactions by adding
	/// a "virtual tip" that's equal to the `OperationalFeeMultiplier * final_fee`.
	pub const OperationalFeeMultiplier: u8 = 5;
}

impl Config for Runtime {
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OnChargeTransaction = CurrencyAdapter<Ring, DealWithFees<Self>>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
}
