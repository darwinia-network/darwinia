// --- paritytech ---
use sp_core::U256;
pub use sp_runtime::{Perbill, Permill};
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub IsActive: bool = true;
	pub DefaultBaseFeePerGas: U256 = U256::from(1_000_000_000);
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
	fn lower() -> Permill {
		Permill::zero()
	}

	fn ideal() -> Permill {
		Permill::from_parts(500_000)
	}

	fn upper() -> Permill {
		Permill::from_parts(1_000_000)
	}
}

impl pallet_base_fee::Config for Runtime {
	type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
	type Event = Event;
	type IsActive = IsActive;
	type Threshold = BaseFeeThreshold;
}
