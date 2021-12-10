// --- core ---
use core::marker::PhantomData;
// --- crates.io ---
use evm::{executor::PrecompileOutput, Context, ExitError};
// --- paritytech ---
use bp_messages::LaneId;
use frame_support::{
	dispatch::Dispatchable,
	traits::{FindAuthor, PalletInfoAccess},
	weights::{GetDispatchInfo, PostDispatchInfo},
	ConsensusEngineId,
};
use sp_core::{crypto::Public, H160, U256};
// --- darwinia-network ---
use crate::{messages::darwinia_message::ToDarwiniaMessagePayload, *};
use darwinia_evm::{runner::stack::Runner, Config, EnsureAddressTruncated, FeeCalculator};
use darwinia_evm_precompile_bridge_s2s::Sub2SubBridge;
use darwinia_evm_precompile_dispatch::Dispatch;
use darwinia_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
use darwinia_evm_precompile_transfer::Transfer;
use darwinia_support::{
	evm::ConcatConverter,
	s2s::{LatestMessageNoncer, RelayMessageSender},
};
use dp_evm::{Precompile, PrecompileSet};
use dvm_ethereum::{
	account_basic::{DvmAccountBasic, KtonRemainBalance, RingRemainBalance},
	EthereumBlockHashMapping,
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
		let payload = ToDarwiniaMessagePayload::decode(&mut payload.as_slice())
			.map_err(|_| "decode darwinia payload failed")?;
		let call: Call = match message_pallet_index {
			_ if message_pallet_index as usize
				== <BridgeDarwiniaMessages as PalletInfoAccess>::index() =>
			{
				BridgeMessagesCall::<Runtime, WithDarwiniaMessages>::send_message(
					lane_id,
					payload,
					fee.saturated_into(),
				)
				.into()
			}
			_ => {
				return Err("invalid pallet index".into());
			}
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
impl<R> PrecompileSet for CrabPrecompiles<R>
where
	R: from_substrate_issuing::Config,
	R: darwinia_evm::Config,
	R::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Encode + Decode,
	<R::Call as Dispatchable>::Origin: From<Option<R::AccountId>>,
	R::Call: From<from_substrate_issuing::Call<R>>,
{
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
			_ if address == addr(24) => Some(<Sub2SubBridge<R, ToDarwiniaMessageSender>>::execute(
				input, target_gas, context,
			)),
			_ if address == addr(25) => Some(<Dispatch<R>>::execute(input, target_gas, context)),
			_ => None,
		}
	}
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> U256 {
		U256::from(10 * GWEI)
	}
}

frame_support::parameter_types! {
	pub const ChainId: u64 = 44;
	pub BlockGasLimit: U256 = u32::MAX.into();
}

impl Config for Runtime {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressTruncated<Self::AccountId>;
	type IntoAccountId = ConcatConverter<Self::AccountId>;
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
