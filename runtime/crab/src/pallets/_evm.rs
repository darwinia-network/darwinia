// --- core ---
use core::marker::PhantomData;
// --- crates.io ---
use evm::ExitRevert;
// --- paritytech ---
use fp_evm::{Context, Precompile, PrecompileResult, PrecompileSet, PrecompileFailure};
use frame_support::{
	pallet_prelude::Weight,
	traits::{FindAuthor, PalletInfoAccess},
	ConsensusEngineId,
};
use pallet_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
use pallet_session::FindAccountFromAuthorIndex;
use sp_core::{crypto::Public, H160, U256};
// --- darwinia-network ---
use crate::*;
use bp_messages::LaneId;
use darwinia_ethereum::{
	account_basic::{DvmAccountBasic, KtonRemainBalance, RingRemainBalance},
	EthereumBlockHashMapping,
};
use darwinia_evm::{
	runner::stack::Runner, Config, EVMCurrencyAdapter, EnsureAddressTruncated, GasWeightMapping,
};
use darwinia_evm_precompile_bridge_s2s::Sub2SubBridge;
use darwinia_evm_precompile_dispatch::Dispatch;
use darwinia_evm_precompile_transfer::Transfer;
use darwinia_support::{
	evm::ConcatConverter,
	s2s::{LatestMessageNoncer, RelayMessageSender},
};

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

pub struct ToDarwiniaMessageSender;
impl RelayMessageSender for ToDarwiniaMessageSender {
	fn encode_send_message(
		message_pallet_index: u32,
		lane_id: LaneId,
		payload: Vec<u8>,
		fee: u128,
	) -> Result<Vec<u8>, &'static str> {
		let payload = bm_darwinia::ToDarwiniaMessagePayload::decode(&mut payload.as_slice())
			.map_err(|_| "decode darwinia payload failed")?;
		let call: Call = match message_pallet_index {
			_ if message_pallet_index as usize
				== <BridgeDarwiniaMessages as PalletInfoAccess>::index() =>
				pallet_bridge_messages::Call::<Runtime, WithDarwiniaMessages>::send_message {
					lane_id,
					payload,
					delivery_and_dispatch_fee: fee.saturated_into(),
				}
				.into(),
			_ => {
				return Err("invalid pallet index".into());
			},
		};

		Ok(call.encode())
	}
}
impl LatestMessageNoncer for ToDarwiniaMessageSender {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> u64 {
		BridgeDarwiniaMessages::outbound_latest_generated_nonce(lane_id).into()
	}

	fn inbound_latest_received_nonce(lane_id: LaneId) -> u64 {
		BridgeDarwiniaMessages::inbound_latest_received_nonce(lane_id).into()
	}
}

pub struct CrabPrecompiles<R>(PhantomData<R>);
impl<R> CrabPrecompiles<R>
where
	R: darwinia_ethereum::Config,
{
	pub fn new() -> Self {
		Self(Default::default())
	}

	pub fn used_addresses() -> sp_std::vec::Vec<H160> {
		sp_std::vec![1, 2, 3, 4, 21, 24, 25].into_iter().map(|x| addr(x)).collect()
	}
}

impl<R> PrecompileSet for CrabPrecompiles<R>
where
	Transfer<R>: Precompile,
	Sub2SubBridge<R, ToDarwiniaMessageSender, bm_darwinia::ToDarwiniaOutboundPayLoad>: Precompile,
	Dispatch<R>: Precompile,
	R: darwinia_ethereum::Config,
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
		if self.is_precompile(address) && address > addr(4) && address != context.address {
			return Some(Err(PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: b"cannot be called with DELEGATECALL or CALLCODE".to_vec(),
				cost: 0,
			}));
		};

		match address {
			// Ethereum precompiles
			a if a == addr(1) => Some(ECRecover::execute(input, target_gas, context, is_static)),
			a if a == addr(2) => Some(Sha256::execute(input, target_gas, context, is_static)),
			a if a == addr(3) => Some(Ripemd160::execute(input, target_gas, context, is_static)),
			a if a == addr(4) => Some(Identity::execute(input, target_gas, context, is_static)),
			// Darwinia precompiles
			a if a == addr(21) =>
				Some(<Transfer<R>>::execute(input, target_gas, context, is_static)),
			a if a == addr(24) => Some(<Sub2SubBridge<
				R,
				ToDarwiniaMessageSender,
				bm_darwinia::ToDarwiniaOutboundPayLoad,
			>>::execute(input, target_gas, context, is_static)),
			a if a == addr(25) =>
				Some(<Dispatch<R>>::execute(input, target_gas, context, is_static)),
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
		U256::from(1 * GWEI)
	}
}

pub struct FixedGasWeightMapping;
impl GasWeightMapping for FixedGasWeightMapping {
	fn gas_to_weight(gas: u64) -> Weight {
		gas.saturating_mul(WEIGHT_PER_GAS)
	}

	fn weight_to_gas(weight: Weight) -> u64 {
		u64::try_from(weight.wrapping_div(WEIGHT_PER_GAS)).unwrap_or(u32::MAX as u64)
	}
}

frame_support::parameter_types! {
	pub const ChainId: u64 = 44;
	pub BlockGasLimit: U256 = (NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT / WEIGHT_PER_GAS).into();
	pub PrecompilesValue: CrabPrecompiles<Runtime> = CrabPrecompiles::<_>::new();
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
	type KtonAccountBasic = DvmAccountBasic<Self, Kton, KtonRemainBalance>;
	type OnChargeTransaction = EVMCurrencyAdapter<FindAccountFromAuthorIndex<Self, Babe>>;
	type PrecompilesType = CrabPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type RingAccountBasic = DvmAccountBasic<Self, Ring, RingRemainBalance>;
	type Runner = Runner<Self>;
}

fn addr(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
