// --- substrate ---
use sp_core::U256;
// --- darwinia ---
use crate::*;
use darwinia_evm::{
	runner::stack::Runner, ConcatAddressMapping, Config, EnsureAddressTruncated, FeeCalculator,
};
use dvm_ethereum::account_basic::DvmAccountBasic;
use dvm_ethereum::account_basic::{KtonRemainBalance, RingRemainBalance};

/// Fixed gas price.
pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> U256 {
		// Gas price is always one token per gas.
		1_000_000_000.into()
	}
}

pub type CrabPrecompiles<Runtime> = (
	darwinia_evm_precompile_simple::ECRecover, // 0x0000000000000000000000000000000000000001
	darwinia_evm_precompile_simple::Sha256,    // 0x0000000000000000000000000000000000000002
	darwinia_evm_precompile_simple::Ripemd160, // 0x0000000000000000000000000000000000000003
	darwinia_evm_precompile_simple::Identity,  // 0x0000000000000000000000000000000000000004
	darwinia_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000005
	darwinia_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000006
	darwinia_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000007
	darwinia_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000008
	darwinia_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000009
	darwinia_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000a
	darwinia_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000b
	darwinia_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000c
	darwinia_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000d
	darwinia_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000e
	darwinia_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000f
	darwinia_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000010
	darwinia_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000011
	darwinia_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000012
	darwinia_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000013
	darwinia_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000014
	darwinia_evm_precompile_withdraw::WithDraw<Runtime>, // 0x0000000000000000000000000000000000000015
);

frame_support::parameter_types! {
	pub const ChainId: u64 = 44;
	pub BlockGasLimit: U256 = U256::from(u32::max_value());
}
impl Config for Runtime {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressTruncated;
	type WithdrawOrigin = EnsureAddressTruncated;
	type AddressMapping = ConcatAddressMapping;
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
	type Event = Event;
	type Precompiles = CrabPrecompiles<Self>;
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type RingAccountBasic = DvmAccountBasic<Self, Ring, RingRemainBalance>;
	type KtonAccountBasic = DvmAccountBasic<Self, Kton, KtonRemainBalance>;
	type Runner = Runner<Self>;
}
