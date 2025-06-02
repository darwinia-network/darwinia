pub mod precompiles {
	pub const ADDR_EC_RECOVER: [u8; 20] = address_of(0x01);
	pub const ADDR_SHA256: [u8; 20] = address_of(0x02);
	pub const ADDR_RIPEMD160: [u8; 20] = address_of(0x03);
	pub const ADDR_IDENTITY: [u8; 20] = address_of(0x04);
	pub const ADDR_MODEXP: [u8; 20] = address_of(0x05);
	pub const ADDR_BN128_ADD: [u8; 20] = address_of(0x06);
	pub const ADDR_BN128_MUL: [u8; 20] = address_of(0x07);
	pub const ADDR_BN128_PAIRING: [u8; 20] = address_of(0x08);
	pub const ADDR_BLAKE2F: [u8; 20] = address_of(0x09);
	// https://github.com/ethereum/go-ethereum/blob/e56bbd77a44fc26550a862801690461e49e02503/core/vm/contracts.go#L131-L139.
	pub const ADDR_BLS12381_G1_ADD: [u8; 20] = address_of(0x0b);
	pub const ADDR_BLS12381_G1_MUL: [u8; 20] = address_of(0x0c);
	pub const ADDR_BLS12381_G1_MULTI_EXP: [u8; 20] = address_of(0x0d);
	pub const ADDR_BLS12381_G2_ADD: [u8; 20] = address_of(0x0e);
	pub const ADDR_BLS12381_G2_MUL: [u8; 20] = address_of(0x0f);
	pub const ADDR_BLS12381_G2_MULTI_EXP: [u8; 20] = address_of(0x10);
	pub const ADDR_BLS12381_PAIRING: [u8; 20] = address_of(0x11);
	pub const ADDR_BLS12381_MAP_G1: [u8; 20] = address_of(0x12);
	pub const ADDR_BLS12381_MAP_G2: [u8; 20] = address_of(0x13);
	// [0x400, 0x800) for stable precompiles.
	pub const ADDR_STATE_STORAGE: [u8; 20] = address_of(0x400);
	pub const ADDR_DISPATCH: [u8; 20] = address_of(0x401);
	// [0x402, 0x600) for assets precompiles.
	pub const ADDR_KTON: [u8; 20] = address_of(0x402);
	pub const ADDR_USDT: [u8; 20] = address_of(0x403);
	pub const ADDR_PINK: [u8; 20] = address_of(0x404);
	pub const ADDR_DOT: [u8; 20] = address_of(0x405);
	pub const ADDR_DEPOSIT_DEPRECATED: [u8; 20] = address_of(0x600);
	pub const ADDR_STAKING_DEPRECATED: [u8; 20] = address_of(0x601);
	pub const ADDR_CONVICTION_VOTING: [u8; 20] = address_of(0x602);
	// [0x800..) for the experimental precompiles.
	pub const ADDR_EXPERIMENTAL: [u8; 20] = address_of(0x800);

	pub const fn address_of(v: u64) -> [u8; 20] {
		[
			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
			((v >> 56) & 0xff) as u8,
			((v >> 48) & 0xff) as u8,
			((v >> 40) & 0xff) as u8,
			((v >> 32) & 0xff) as u8,
			((v >> 24) & 0xff) as u8,
			((v >> 16) & 0xff) as u8,
			((v >> 8) & 0xff) as u8,
			(v & 0xff) as u8,
		]
	}

	#[test]
	fn address_of_should_work() {
		// polkadot-sdk
		use sp_core::H160;

		fn non_const_address_of(v: u64) -> H160 {
			H160::from_low_u64_be(v)
		}

		for code in 0x01..=0x800 {
			assert_eq!(address_of(code), non_const_address_of(code).0);
		}
	}
}

// darwinia
use dc_primitives::*;
// polkadot-sdk
use sp_core::U256;
use sp_runtime::traits::AccountIdConversion;
use sp_std::vec;

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(75);
pub const MAXIMUM_BLOCK_WEIGHT: frame_support::weights::Weight =
	frame_support::weights::Weight::from_parts(
		frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
		cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
	);

// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(5);
const WEIGHT_MILLISECS_PER_BLOCK: u64 = 2_000;
const BLOCK_GAS_LIMIT: u64 = 20_000_000;

#[cfg(not(feature = "runtime-benchmarks"))]
const EXISTENTIAL_DEPOSIT: Balance = 0;
#[cfg(feature = "runtime-benchmarks")]
const EXISTENTIAL_DEPOSIT: Balance = 100;
frame_support::parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}
frame_support::parameter_types! {
	pub const MaxBalance: Balance = Balance::MAX;

	// Retry a scheduled item every 10 blocks (1 minute) until the preimage exists.
	pub const NoPreimagePostponement: Option<u32> = Some(10);

	pub const TreasuryPid: frame_support::PalletId = frame_support::PalletId(*b"da/trsry");

	pub const RelayOrigin: cumulus_primitives_core::AggregateMessageOrigin = cumulus_primitives_core::AggregateMessageOrigin::Parent;

	pub const ReservedXcmpWeight: frame_support::weights::Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: frame_support::weights::Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);

	// Disable gas based storage growth limit.
	pub const GasLimitStorageGrowthRatio: u64 = 0;

	pub RuntimeBlockLength: frame_system::limits::BlockLength =
		frame_system::limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights::builder()
		.base_block(frame_support::weights::constants::BlockExecutionWeight::get())
		.for_class(frame_support::dispatch::DispatchClass::all(), |weights| {
			weights.base_extrinsic = frame_support::weights::constants::ExtrinsicBaseWeight::get();
		})
		.for_class(frame_support::dispatch::DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(frame_support::dispatch::DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();

	pub MaximumSchedulerWeight: frame_support::weights::Weight = sp_runtime::Perbill::from_percent(80)
		* RuntimeBlockWeights::get().max_block;

	pub TreasuryAccount: AccountId = TreasuryPid::get().into_account_truncating();

	pub MaxProposalWeight: frame_support::weights::Weight = sp_runtime::Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;

	pub MessageQueueServiceWeight: frame_support::weights::Weight = sp_runtime::Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;

	pub BlockGasLimit: U256 = U256::from(BLOCK_GAS_LIMIT);
	// Restrict the POV size of the Ethereum transactions in the same way as weight limit.
	pub BlockPovSizeLimit: u64 = NORMAL_DISPATCH_RATIO * cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64;
	pub WeightPerGas: frame_support::weights::Weight = frame_support::weights::Weight::from_parts(
		fp_evm::weight_per_gas(BLOCK_GAS_LIMIT, NORMAL_DISPATCH_RATIO, WEIGHT_MILLISECS_PER_BLOCK),
		0
	);
	// FIXME: https://github.com/rust-lang/rust/issues/88581
	pub GasLimitPovSizeRatio: u64 = BLOCK_GAS_LIMIT.saturating_div(BlockPovSizeLimit::get()) + 1;
}

frame_support::ord_parameter_types! {
	pub const AssetCreator: AccountId = super::gov_origin::ROOT;
}
