// --- substrate ---
use pallet_transaction_payment::{Config, CurrencyAdapter};
// --- darwinia ---
use crate::*;

frame_support::parameter_types! {
	pub const TransactionByteFee: Balance = 50 * MICRO;
}
impl Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees<Self>>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}
