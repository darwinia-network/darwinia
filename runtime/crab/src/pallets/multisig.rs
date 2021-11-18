// --- paritytech ---
use pallet_multisig::Config;
// --- darwinia-network ---
use crate::{weights::pallet_multisig::WeightInfo, *};

// --- crates.io ---
use smallvec::smallvec;
// --- paritytech ---
use frame_support::weights::{
	WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
use sp_runtime::Perbill;
// --- darwinia-network ---
use common_primitives::Balance;
use darwinia_runtime_common::ExtrinsicBaseWeight;

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
		// in Crab, extrinsic base weight (smallest non-zero weight) is mapped to 100 MILLI:
		let p = 100 * MILLI;
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
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = crab_deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = crab_deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Ring;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = WeightInfo<Runtime>;
}
