// --- core ---
use core::marker::PhantomData;
// --- paritytech ---
use fp_evm::{Context, ExitRevert, Precompile, PrecompileFailure, PrecompileResult, PrecompileSet};
use frame_support::{
	pallet_prelude::Weight, traits::FindAuthor, ConsensusEngineId, StorageHasher, Twox128,
};
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
use pallet_session::FindAccountFromAuthorIndex;
use sp_core::{crypto::Public, H160, U256};
// --- darwinia-network ---
use crate::*;
use darwinia_ethereum::{
	adapter::{CurrencyAdapter, KtonRemainBalance, RingRemainBalance},
	EthereumBlockHashMapping,
};
use darwinia_evm::{
	runner::stack::Runner, Config, EVMCurrencyAdapter, EnsureAddressTruncated, GasWeightMapping,
};
use darwinia_evm_precompile_dispatch::Dispatch;
use darwinia_evm_precompile_kton::{Erc20Metadata, KtonERC20};
use darwinia_evm_precompile_state_storage::{StateStorage, StorageFilterT};
use darwinia_support::evm::ConcatConverter;

pub struct EthereumFindAuthor<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for EthereumFindAuthor<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		F::find_author(digests).map(|author_index| {
			let authority_id = Babe::authorities()[author_index as usize].clone();

			H160::from_slice(&authority_id.0.to_raw_vec()[4..24])
		})
	}
}

pub struct StorageFilter;
impl StorageFilterT for StorageFilter {
	fn allow(prefix: &[u8]) -> bool {
		prefix != Twox128::hash(b"EVM") && prefix != Twox128::hash(b"Ethereum")
	}
}

struct KtonERC20MetaData;
impl Erc20Metadata for KtonERC20MetaData {
	fn name() -> &'static str {
		"KTON ERC20"
	}

	fn symbol() -> &'static str {
		"KTON"
	}

	fn decimals() -> u8 {
		18
	}
}

pub struct DarwiniaPrecompiles<R>(PhantomData<R>);
impl<R> DarwiniaPrecompiles<R>
where
	R: darwinia_ethereum::Config,
{
	pub fn new() -> Self {
		Self(Default::default())
	}

	pub fn used_addresses() -> [H160; 12] {
		[
			addr(1),
			addr(2),
			addr(3),
			addr(4),
			addr(5),
			addr(6),
			addr(7),
			addr(8),
			addr(9),
			addr(1024),
			addr(1025),
			addr(1026),
		]
	}
}

impl<R> PrecompileSet for DarwiniaPrecompiles<R>
where
	Dispatch<R>: Precompile,
	R: darwinia_ethereum::Config,
	StateStorage<R, StorageFilter>: Precompile,
{
	fn execute(
		&self,
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<PrecompileResult> {
		// Filter known precompile addresses except Ethereum officials
		if self.is_precompile(address) && address > addr(9) && address != context.address {
			return Some(Err(PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: b"cannot be called with DELEGATECALL or CALLCODE".to_vec(),
				cost: 0,
			}));
		};

		match address {
			// Ethereum precompiles:
			a if a == addr(1) => Some(ECRecover::execute(input, target_gas, context, is_static)),
			a if a == addr(2) => Some(Sha256::execute(input, target_gas, context, is_static)),
			a if a == addr(3) => Some(Ripemd160::execute(input, target_gas, context, is_static)),
			a if a == addr(4) => Some(Identity::execute(input, target_gas, context, is_static)),
			a if a == addr(5) => Some(Modexp::execute(input, target_gas, context, is_static)),
			a if a == addr(6) => Some(Bn128Add::execute(input, target_gas, context, is_static)),
			a if a == addr(7) => Some(Bn128Mul::execute(input, target_gas, context, is_static)),
			a if a == addr(8) => Some(Bn128Pairing::execute(input, target_gas, context, is_static)),
			a if a == addr(9) => Some(Blake2F::execute(input, target_gas, context, is_static)),
			// Darwinia precompiles: 1024+ for stable precompiles.
			a if a == addr(1024) => Some(<StateStorage<R, StorageFilter>>::execute(
				input, target_gas, context, is_static,
			)),
			a if a == addr(1025) =>
				Some(<Dispatch<R>>::execute(input, target_gas, context, is_static)),
			a if a == addr(1026) => Some(<KtonERC20<R, KtonERC20MetaData>>::execute(
				input, target_gas, context, is_static,
			)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		Self::used_addresses().contains(&address)
	}
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> U256 {
		U256::from(GWEI)
	}
}

pub struct FixedGasWeightMapping;
impl GasWeightMapping for FixedGasWeightMapping {
	fn gas_to_weight(gas: u64) -> Weight {
		gas.saturating_mul(WEIGHT_PER_GAS)
	}

	fn weight_to_gas(weight: Weight) -> u64 {
		weight.wrapping_div(WEIGHT_PER_GAS)
	}
}

fn addr(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}

frame_support::parameter_types! {
	pub const ChainId: u64 = 46;
	pub BlockGasLimit: U256 = U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT / WEIGHT_PER_GAS);
	pub PrecompilesValue: DarwiniaPrecompiles<Runtime> = DarwiniaPrecompiles::<_>::new();
}

impl Config for Runtime {
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressTruncated<Self::AccountId>;
	type ChainId = ChainId;
	type Event = Event;
	type FeeCalculator = FixedGasPrice;
	type FindAuthor = EthereumFindAuthor<Babe>;
	type GasWeightMapping = FixedGasWeightMapping;
	type IntoAccountId = ConcatConverter<Self::AccountId>;
	type KtonBalanceAdapter = CurrencyAdapter<Self, Kton, KtonRemainBalance>;
	type OnChargeTransaction = EVMCurrencyAdapter<FindAccountFromAuthorIndex<Self, Babe>>;
	type PrecompilesType = DarwiniaPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type RingBalanceAdapter = CurrencyAdapter<Self, Ring, RingRemainBalance>;
	type Runner = Runner<Self>;
}
