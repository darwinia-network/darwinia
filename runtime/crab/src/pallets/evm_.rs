// --- crates.io ---
use evm::{executor::PrecompileOutput, Context, ExitError};
// --- paritytech ---
use frame_support::{traits::FindAuthor, ConsensusEngineId};
use sp_core::{crypto::Public, H160, U256};
use sp_std::marker::PhantomData;
// --- darwinia-network ---
use crate::*;
use darwinia_evm::{runner::stack::Runner, ConcatAddressMapping, Config, EnsureAddressTruncated};
use darwinia_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
use darwinia_evm_precompile_transfer::Transfer;
use dp_evm::{Precompile, PrecompileSet};
use dvm_ethereum::{
	account_basic::{DvmAccountBasic, KtonRemainBalance, RingRemainBalance},
	EthereumBlockHashMapping,
};

pub struct EthereumFindAuthor<F>(sp_std::marker::PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for EthereumFindAuthor<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		if let Some(author_index) = F::find_author(digests) {
			let authority_id = Babe::authorities()[author_index as usize].clone();
			return Some(H160::from_slice(&authority_id.0.to_raw_vec()[4..24]));
		}
		None
	}
}

pub struct CrabPrecompiles<R>(PhantomData<R>);
impl<R: darwinia_evm::Config> PrecompileSet for CrabPrecompiles<R> {
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> Option<Result<PrecompileOutput, ExitError>> {
		let addr = |n: u64| -> H160 { H160::from_low_u64_be(n) };

		match address {
			// Ethereum precompiles
			_ if address == addr(1) => Some(ECRecover::execute(input, target_gas, context)),
			_ if address == addr(2) => Some(Sha256::execute(input, target_gas, context)),
			_ if address == addr(3) => Some(Ripemd160::execute(input, target_gas, context)),
			_ if address == addr(4) => Some(Identity::execute(input, target_gas, context)),
			// Darwinia precompiles
			_ if address == addr(21) => Some(<Transfer<R>>::execute(input, target_gas, context)),
			_ => None,
		}
	}
}

frame_support::parameter_types! {
	pub const ChainId: u64 = 44;
	pub BlockGasLimit: U256 = U256::from(u32::max_value());
}

impl Config for Runtime {
	type FeeCalculator = DynamicFee;
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressTruncated<Self::AccountId>;
	type AddressMapping = ConcatAddressMapping<Self::AccountId>;
	type FindAuthor = EthereumFindAuthor<Babe>;
	type BlockHashMapping = EthereumBlockHashMapping<Self>;
	type Event = Event;
	type Precompiles = CrabPrecompiles<Self>;
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type RingAccountBasic = DvmAccountBasic<Self, Ring, RingRemainBalance>;
	type KtonAccountBasic = DvmAccountBasic<Self, Kton, KtonRemainBalance>;
	type Runner = Runner<Self>;
}
