// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! The Darwinia runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

/// Constant values used within the runtime.
pub mod constants;
pub use constants::{currency::*, fee::*, relay::*, time::*};

pub mod pallets;
pub use pallets::*;

#[cfg(feature = "std")]
pub mod genesis_loader {
	// --- std ---
	use std::fs::File;
	// --- crates ---
	use serde::{de::Error, Deserialize, Deserializer};
	// --- darwinia ---
	use crate::*;

	#[derive(Deserialize)]
	struct Account {
		target: String,
		#[serde(deserialize_with = "string_to_balance")]
		amount: Balance,
	}

	fn string_to_balance<'de, D>(deserializer: D) -> Result<Balance, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s: String = Deserialize::deserialize(deserializer)?;

		s.parse::<Balance>().map_err(Error::custom)
	}

	pub fn load_genesis_swap_from_file(path: &str) -> Result<Vec<(String, Balance)>, String> {
		serde_json::from_reader(File::open(path).map_err(|e| e.to_string())?)
			.map(|accounts: Vec<Account>| {
				accounts
					.into_iter()
					.map(|Account { target, amount }| (target, amount))
					.collect()
			})
			.map_err(|e| e.to_string())
	}
}

pub mod wasm {
	//! Make the WASM binary available.

	#[cfg(all(
		feature = "std",
		any(target_arch = "x86_64", target_arch = "x86", target_vendor = "apple")
	))]
	include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

	#[cfg(all(
		feature = "std",
		not(any(target_arch = "x86_64", target_arch = "x86", target_vendor = "apple"))
	))]
	pub const WASM_BINARY: &[u8] = include_bytes!("../../../wasm/darwinia_runtime.compact.wasm");
	#[cfg(all(
		feature = "std",
		not(any(target_arch = "x86_64", target_arch = "x86", target_vendor = "apple"))
	))]
	pub const WASM_BINARY_BLOATY: &[u8] = include_bytes!("../../../wasm/darwinia_runtime.wasm");

	#[cfg(feature = "std")]
	/// Wasm binary unwrapped. If built with `BUILD_DUMMY_WASM_BINARY`, the function panics.
	pub fn wasm_binary_unwrap() -> &'static [u8] {
		#[cfg(all(
			feature = "std",
			any(target_arch = "x86_64", target_arch = "x86", target_vendor = "apple")
		))]
		return WASM_BINARY.expect(
			"Development wasm binary is not available. This means the client is \
						built with `BUILD_DUMMY_WASM_BINARY` flag and it is only usable for \
						production chains. Please rebuild with the flag disabled.",
		);
		#[cfg(all(
			feature = "std",
			not(any(target_arch = "x86_64", target_arch = "x86", target_vendor = "apple"))
		))]
		return WASM_BINARY;
	}
}
pub use wasm::*;

/// Weights for pallets used in the runtime.
mod weights;

#[cfg(feature = "std")]
pub use darwinia_ethereum_relay::DagsMerkleRootsLoader;
#[cfg(feature = "std")]
pub use darwinia_staking::{Forcing, StakerStatus};

// --- crates ---
use codec::Encode;
// --- substrate ---
use frame_support::{
	traits::{KeyOwnerProofSystem, OnRuntimeUpgrade},
	weights::Weight,
	PalletId,
};
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_session::historical as pallet_session_historical;
use pallet_transaction_payment::FeeDetails;
use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo as TransactionPaymentRuntimeDispatchInfo;
use sp_api::impl_runtime_apis;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::{AllowedSlots, BabeEpochConfiguration};
use sp_core::OpaqueMetadata;
use sp_runtime::{
	generic,
	traits::{
		AccountIdLookup, BlakeTwo256, Block as BlockT, Extrinsic as ExtrinsicT, NumberFor,
		SaturatedConversion, StaticLookup, Verify,
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiAddress,
};
use sp_std::prelude::*;
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
// --- darwinia ---
use darwinia_balances_rpc_runtime_api::RuntimeDispatchInfo as BalancesRuntimeDispatchInfo;
use darwinia_header_mmr_rpc_runtime_api::RuntimeDispatchInfo as HeaderMMRRuntimeDispatchInfo;
use darwinia_primitives::*;
use darwinia_runtime_common::*;
use darwinia_staking_rpc_runtime_api::RuntimeDispatchInfo as StakingRuntimeDispatchInfo;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	darwinia_ethereum_relay::CheckEthereumRelayHeaderParcel<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Nonce, Call>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPallets,
	CustomOnRuntimeUpgrade,
>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

type Ring = Balances;

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: BabeEpochConfiguration = BabeEpochConfiguration {
	c: PRIMARY_PROBABILITY,
	allowed_slots: AllowedSlots::PrimaryAndSecondaryVRFSlots,
};

/// Runtime version (Darwinia).
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: sp_runtime::create_runtime_str!("Darwinia"),
	impl_name: sp_runtime::create_runtime_str!("Darwinia"),
	authoring_version: 0,
	spec_version: 1110,
	impl_version: 0,
	#[cfg(not(feature = "disable-runtime-api"))]
	apis: RUNTIME_API_VERSIONS,
	#[cfg(feature = "disable-runtime-api")]
	apis: sp_version::create_apis_vec![[]],
	transaction_version: 2,
};

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

frame_support::construct_runtime! {
	pub enum Runtime
	where
		Block = Block,
		NodeBlock = OpaqueBlock,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// Basic stuff; balances is uncallable initially.
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 1,

		// Must be before session.
		Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 2,

		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
		Balances: darwinia_balances::<Instance1>::{Pallet, Call, Storage, Config<T>, Event<T>} = 4,
		Kton: darwinia_balances::<Instance2>::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 6,

		// Consensus support.
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 7,
		ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 38,
		Staking: darwinia_staking::{Pallet, Call, Storage, Config<T>, Event<T>} = 8,
		Offences: pallet_offences::{Pallet, Call, Storage, Event} = 9,
		Historical: pallet_session_historical::{Pallet} = 10,
		Session: pallet_session::{Pallet, Call, Storage, Config<T>, Event} = 11,
		Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event, ValidateUnsigned} = 13,
		ImOnline: pallet_im_online::{Pallet, Call, Storage, Config<T>, Event<T>, ValidateUnsigned} = 14,
		AuthorityDiscovery: pallet_authority_discovery::{Pallet, Call, Config} = 15,
		DarwiniaHeaderMMR: darwinia_header_mmr::{Pallet, Call, Storage} = 35,

		// Governance stuff; uncallable initially.
		Democracy: darwinia_democracy::{Pallet, Call, Storage, Config, Event<T>} = 37,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Config<T>, Event<T>} = 16,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Config<T>, Event<T>} = 17,
		PhragmenElection: darwinia_elections_phragmen::{Pallet, Call, Storage, Config<T>, Event<T>} = 18,
		TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Config<T>, Event<T>} = 19,
		Treasury: darwinia_treasury::{Pallet, Call, Storage, Event<T>} = 20,

		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 27,

		// Vesting. Usable initially, but removed once all vesting is finished.
		Vesting: darwinia_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,

		// Utility module.
		Utility: pallet_utility::{Pallet, Call, Event} = 22,

		// Less simple identity module.
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 23,

		// Society module.
		Society: pallet_society::{Pallet, Call, Storage, Event<T>} = 24,

		// Social recovery module.
		Recovery: pallet_recovery::{Pallet, Call, Storage, Event<T>} = 25,

		// System scheduler.
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 26,

		// Proxy module. Late addition.
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 28,

		// Multisig module. Late addition.
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 29,

		// Crab bridge.
		CrabBacking: darwinia_crab_backing::{Pallet, Storage} = 30,

		// Ethereum bridge.
		EthereumRelay: darwinia_ethereum_relay::{Pallet, Call, Storage, Config<T>, Event<T>} = 32,
		EthereumBacking: darwinia_ethereum_backing::{Pallet, Call, Storage, Config<T>, Event<T>} = 31,
		EthereumRelayerGame: darwinia_relayer_game::<Instance1>::{Pallet, Storage} = 33,
		EthereumRelayAuthorities: darwinia_relay_authorities::<Instance1>::{Pallet, Call, Storage, Event<T>} = 36,

		// Tron bridge.
		TronBacking: darwinia_tron_backing::{Pallet, Storage, Config<T>} = 34,
	}
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		public: <Signature as Verify>::Signer,
		account: AccountId,
		nonce: <Runtime as frame_system::Config>::Index,
	) -> Option<(Call, <UncheckedExtrinsic as ExtrinsicT>::SignaturePayload)> {
		let period = BlockHashCountForDarwinia::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;

		let current_block = System::block_number()
			.saturated_into::<u64>()
			.saturating_sub(1);
		let tip = 0;
		let extra: SignedExtra = (
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			darwinia_ethereum_relay::CheckEthereumRelayHeaderParcel::<Runtime>::new(),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let (call, extra, _) = raw_payload.deconstruct();
		let address = <Runtime as frame_system::Config>::Lookup::unlookup(account);
		Some((call, (address, signature, extra)))
	}
}
impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}
impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	Call: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = Call;
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(
			data: sp_inherents::InherentData
		) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Grandpa::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			Historical::prove((fg_primitives::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(fg_primitives::OpaqueKeyOwnershipProof::new)
		}
	}

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
		fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
			// The choice of `c` parameter (where `1 - c` represents the
			// probability of a slot being empty), is done in accordance to the
			// slot duration and expected target block time, for safely
			// resisting network delays of maximum two seconds.
			// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
			sp_consensus_babe::BabeGenesisConfiguration {
				slot_duration: Babe::slot_duration(),
				epoch_length: EpochDuration::get(),
				c: BABE_GENESIS_EPOCH_CONFIG.c,
				genesis_authorities: Babe::authorities(),
				randomness: Babe::randomness(),
				allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
			}
		}

		fn current_epoch_start() -> sp_consensus_babe::Slot {
			Babe::current_epoch_start()
		}

		fn current_epoch() -> sp_consensus_babe::Epoch {
			Babe::current_epoch()
		}

		fn next_epoch() -> sp_consensus_babe::Epoch {
			Babe::next_epoch()
		}

		fn generate_key_ownership_proof(
			_slot: sp_consensus_babe::Slot,
			authority_id: sp_consensus_babe::AuthorityId,
		) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
			Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
			key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Babe::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}
	}
	impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityDiscoveryId> {
			AuthorityDiscovery::authorities()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic, len: u32
		) -> TransactionPaymentRuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl darwinia_balances_rpc_runtime_api::BalancesApi<Block, AccountId, Balance> for Runtime {
		fn usable_balance(
			instance: u8,
			account: AccountId
		) -> BalancesRuntimeDispatchInfo<Balance> {
			match instance {
				0 => Ring::usable_balance_rpc(account),
				1 => Kton::usable_balance_rpc(account),
				_ => Default::default()
			}
		}
	}

	impl darwinia_header_mmr_rpc_runtime_api::HeaderMMRApi<Block, Hash> for Runtime {
		fn gen_proof(
			block_number_of_member_leaf: u64,
			block_number_of_last_leaf: u64
		) -> HeaderMMRRuntimeDispatchInfo<Hash> {
			DarwiniaHeaderMMR::gen_proof_rpc(block_number_of_member_leaf, block_number_of_last_leaf )
		}
	}

	impl darwinia_staking_rpc_runtime_api::StakingApi<Block, AccountId, Power> for Runtime {
		fn power_of(account: AccountId) -> StakingRuntimeDispatchInfo<Power> {
			Staking::power_of_rpc(account)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> Result<(Weight, Weight), sp_runtime::RuntimeString> {
			log::info!("try-runtime::on_runtime_upgrade Darwinia.");
			let weight = Executive::try_runtime_upgrade()?;
			Ok((weight, RuntimeBlockWeights::get().max_block))
		}
	}
}

const PARCEL: &[u8] = &[
	202, 249, 79, 231, 204, 56, 160, 18, 49, 109, 186, 12, 193, 41, 111, 162, 171, 63, 180, 1, 170,
	206, 248, 25, 195, 154, 172, 147, 76, 41, 239, 52, 117, 47, 254, 96, 0, 0, 0, 0, 164, 103, 163,
	0, 0, 0, 0, 0, 251, 182, 27, 139, 152, 165, 159, 188, 75, 215, 156, 35, 33, 42, 221, 190, 250,
	235, 40, 159, 33, 105, 232, 137, 197, 28, 197, 96, 93, 5, 90, 84, 163, 251, 9, 90, 144, 163,
	61, 177, 143, 188, 242, 142, 134, 7, 63, 211, 50, 136, 251, 180, 29, 204, 77, 232, 222, 199,
	93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161,
	66, 253, 64, 212, 147, 71, 100, 216, 131, 1, 10, 6, 132, 103, 101, 116, 104, 136, 103, 111, 49,
	46, 49, 53, 46, 54, 133, 108, 105, 110, 117, 120, 252, 213, 242, 224, 177, 167, 40, 219, 178,
	17, 44, 33, 195, 117, 205, 254, 66, 85, 104, 73, 61, 222, 59, 183, 29, 3, 101, 9, 196, 4, 162,
	54, 39, 245, 64, 81, 8, 246, 91, 211, 100, 85, 221, 221, 242, 206, 50, 254, 43, 135, 133, 27,
	233, 127, 206, 62, 94, 255, 72, 99, 110, 229, 47, 30, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128,
	0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 16, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 4, 0, 0, 0, 0, 32, 64, 0, 0, 0, 64,
	8, 0, 0, 32, 32, 0, 0, 1, 0, 0, 0, 0, 0, 0, 64, 0, 0, 128, 0, 0, 0, 0, 0, 4, 0, 2, 0, 0, 128,
	1, 0, 0, 0, 0, 0, 8, 0, 8, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 64, 0, 0, 0, 128, 0, 0, 0, 0, 0, 129, 1, 0, 0, 8, 0, 0, 0, 64, 0, 32, 0, 0, 0, 0, 128, 0,
	0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 16, 0, 0, 32, 0, 0, 0, 32, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 32, 0, 0, 0, 32, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0,
	0, 73, 233, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 18, 122, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 78, 245, 134, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 8, 132, 160, 187, 22, 106, 67, 147, 147, 165, 98, 213, 199, 25, 115, 167, 227,
	241, 184, 123, 198, 187, 101, 177, 178, 82, 78, 132, 107, 2, 28, 108, 23, 10, 22, 36, 136, 238,
	46, 58, 148, 16, 64, 206, 225, 1, 235, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 157, 183, 53, 205, 190, 51, 116, 119, 211, 139, 112,
	217, 105, 152, 222, 203, 157, 142, 161, 215, 150, 205, 198, 201, 117, 70, 19, 41, 120, 219,
	102, 140, 17, 131, 172, 243, 106, 218, 92, 169, 62, 49, 230, 24, 231, 99, 44, 62, 210, 62, 221,
	243, 206, 191, 7, 126, 184, 104, 135, 61, 98, 18, 23, 154,
];

#[test]
fn assert_encoded_eq() {
	darwinia_ethereum_relay::migration::assert_encoded_eq(
		r#"{
			"header": {
				"baseFeePerGas": "0xeb",
				"difficulty": "0x4186f54e",
				"extraData": "0xd883010a06846765746888676f312e31352e36856c696e7578",
				"gasLimit": "0x7a1200",
				"gasUsed": "0x5e949",
				"hash": "0x9db735cdbe337477d38b70d96998decb9d8ea1d796cdc6c97546132978db668c",
				"logsBloom": "0x00200000000000000000000080000000000000004000001000010000000000000000000000000000000000000000000000000000000000000000000008000000040000000020400000004008000020200000010000000000004000008000000000000400020000800100000000000800080000000000400000000010000000000000000000000000004000000080000000000081010000080000004000200000000080000020000000000000000000000000200000080000000000000000000000000006000000000000000000000000000000200000001000002000000020000000000000000000000a00000000200000002000000000400000000000000000",
				"miner": "0xfbb61b8b98a59fbc4bd79c23212addbefaeb289f",
				"mixHash": "0xbb166a439393a562d5c71973a7e3f1b87bc6bb65b1b2524e846b021c6c170a16",
				"nonce": "0xee2e3a941040cee1",
				"number": "0xa367a4",
				"parentHash": "0xcaf94fe7cc38a012316dba0cc1296fa2ab3fb401aacef819c39aac934c29ef34",
				"receiptsRoot": "0x27f5405108f65bd36455ddddf2ce32fe2b87851be97fce3e5eff48636ee52f1e",
				"sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
				"size": "0x794",
				"stateRoot": "0xfcd5f2e0b1a728dbb2112c21c375cdfe425568493dde3bb71d036509c404a236",
				"timestamp": "0x60fe2f75",
				"totalDifficulty": "0x79b2e0d1c5829f",
				"transactions": [],
				"transactionsRoot": "0x2169e889c51cc5605d055a54a3fb095a90a33db18fbcf28e86073fd33288fbb4",
				"uncles": []
			},
			"parent_mmr_root": "0x1183acf36ada5ca93e31e618e7632c3ed23eddf3cebf077eb868873d6212179a"
		}"#,
		PARCEL,
	)
}

pub struct CustomOnRuntimeUpgrade;
impl OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		// --- paritytech ---
		use frame_support::{migration, Identity};
		// --- darwinia-network ---
		use darwinia_header_mmr::NodeIndex;

		darwinia_header_mmr::migration::migrate(b"HeaderMMR");

		assert!(migration::storage_key_iter::<NodeIndex, Hash, Identity>(
			b"HeaderMMR",
			b"MMRNodeList"
		)
		.next()
		.is_none());
		assert!(!migration::have_storage_value(
			b"HeaderMMR",
			b"MMRNodeList",
			&[]
		));
		assert!(!migration::have_storage_value(
			b"HeaderMMR",
			b"PruningConfiguration",
			&[]
		));

		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		darwinia_ethereum_relay::migration::migrate(PARCEL);
		darwinia_relayer_game::migration::migrate::<Runtime, EthereumRelayerGameInstance>();

		RuntimeBlockWeights::get().max_block
	}
}
