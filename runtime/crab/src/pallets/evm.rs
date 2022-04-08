// --- core ---
use core::marker::PhantomData;
// --- paritytech ---
use bp_messages::LaneId;
use fp_evm::{Context, Precompile, PrecompileResult, PrecompileSet};
use frame_support::{
	dispatch::Dispatchable,
	traits::{FindAuthor, PalletInfoAccess},
	weights::{GetDispatchInfo, PostDispatchInfo},
	ConsensusEngineId,
};
use pallet_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
use sp_core::{crypto::Public, H160, U256};
// --- darwinia-network ---
use crate::{messages::darwinia_message::ToDarwiniaMessagePayload, *};
use darwinia_ethereum::{
	account_basic::{DvmAccountBasic, KtonRemainBalance, RingRemainBalance},
	EthereumBlockHashMapping,
};
use darwinia_evm::{runner::stack::Runner, Config, EVMCurrencyAdapter, EnsureAddressTruncated};
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
		let payload = ToDarwiniaMessagePayload::decode(&mut payload.as_slice())
			.map_err(|_| "decode darwinia payload failed")?;
		let call: Call = match message_pallet_index {
			_ if message_pallet_index as usize
				== <BridgeDarwiniaMessages as PalletInfoAccess>::index() =>
			{
				BridgeMessagesCall::<Runtime, WithDarwiniaMessages>::send_message {
					lane_id,
					payload,
					delivery_and_dispatch_fee: fee.saturated_into(),
				}
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
impl<R> CrabPrecompiles<R>
where
	R: darwinia_evm::Config,
{
	pub fn new() -> Self {
		Self(Default::default())
	}
	pub fn used_addresses() -> sp_std::vec::Vec<H160> {
		sp_std::vec![1, 2, 3, 4, 21, 23, 24, 25]
			.into_iter()
			.map(|x| addr(x))
			.collect()
	}
}

impl<R> PrecompileSet for CrabPrecompiles<R>
where
	Transfer<R>: Precompile,
	EthereumBridge<R>: Precompile,
	Sub2SubBridge<R, ToPangoroMessageSender>: Precompile,
	Dispatch<R>: Precompile,
	R: darwinia_evm::Config,
{
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<PrecompileResult> {
		match address {
			// Ethereum precompiles
			a if a == addr(1) => Some(ECRecover::execute(input, target_gas, context, is_static)),
			a if a == addr(2) => Some(Sha256::execute(input, target_gas, context, is_static)),
			a if a == addr(3) => Some(Ripemd160::execute(input, target_gas, context, is_static)),
			a if a == addr(4) => Some(Identity::execute(input, target_gas, context, is_static)),
			// Darwinia precompiles
			a if a == addr(21) => Some(<Transfer<R>>::execute(
				input, target_gas, context, is_static,
			)),
			a if a == addr(24) => Some(<Sub2SubBridge<R, ToDarwiniaMessageSender>>::execute(
				input, target_gas, context, is_static,
			)),
			a if a == addr(25) => Some(<Dispatch<R>>::execute(
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
		U256::from(10 * GWEI)
	}
}

frame_support::parameter_types! {
	pub const ChainId: u64 = 44;
	pub BlockGasLimit: U256 = u32::MAX.into();
	pub PrecompilesValue: PangolinPrecompiles<Runtime> = PangolinPrecompiles::<_>::new();
}

impl Config for Runtime {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressTruncated<Self::AccountId>;
	type IntoAccountId = ConcatConverter<Self::AccountId>;
	type FindAuthor = EthereumFindAuthor<Babe>;
	type BlockHashMapping = EthereumBlockHashMapping<Self>;
	type Event = Event;
	type PrecompilesType = CrabPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type RingAccountBasic = DvmAccountBasic<Self, Ring, RingRemainBalance>;
	type KtonAccountBasic = DvmAccountBasic<Self, Kton, KtonRemainBalance>;
	type Runner = Runner<Self>;
	type OnChargeTransaction = EVMCurrencyAdapter;
}

fn addr(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
