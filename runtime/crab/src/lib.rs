// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
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

//! Crab runtime.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod weights;

mod pallets;
pub use pallets::*;

mod migration;

pub use darwinia_common_runtime::*;
pub use dc_primitives::*;

// crates.io
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
// polkadot-sdk
use sp_core::{H160, H256, U256};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

/// Block type as expected by this runtime.
pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = sp_runtime::generic::SignedBlock<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	cumulus_primitives_storage_weight_reclaim::StorageWeightReclaim<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	(
		migration::CustomOnRuntimeUpgrade,
		cumulus_pallet_xcmp_queue::migration::v5::MigrateV4ToV5<Runtime>,
	),
>;

/// Runtime version.
#[sp_version::runtime_version]
pub const VERSION: sp_version::RuntimeVersion = sp_version::RuntimeVersion {
	spec_name: sp_runtime::create_runtime_str!("Crab2"),
	impl_name: sp_runtime::create_runtime_str!("DarwiniaOfficialRust"),
	authoring_version: 0,
	spec_version: 6_9_1_0,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 0,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> sp_version::NativeVersion {
	sp_version::NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

// Create the runtime by composing the FRAME pallets that were previously configured.
#[cfg(not(feature = "dev"))]
#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask
	)]
	pub struct Runtime;

	// System stuff.
	#[runtime::pallet_index(0)]
	pub type System = frame_system;
	#[runtime::pallet_index(1)]
	pub type ParachainSystem = cumulus_pallet_parachain_system;
	#[runtime::pallet_index(2)]
	pub type Timestamp = pallet_timestamp;
	#[runtime::pallet_index(3)]
	pub type ParachainInfo = parachain_info;

	// Monetary stuff.
	// Leave 4 here.
	// To keep balances consistent with the existing XCM configurations.
	#[runtime::pallet_index(5)]
	pub type Balances = pallet_balances;
	#[runtime::pallet_index(6)]
	pub type TransactionPayment = pallet_transaction_payment;
	#[runtime::pallet_index(7)]
	pub type Assets = pallet_assets;
	// #[runtime::pallet_index(8)]
	// pub type Vesting = pallet_vesting;
	#[runtime::pallet_index(9)]
	pub type Deposit = darwinia_deposit;
	#[runtime::pallet_index(10)]
	pub type AccountMigration = darwinia_account_migration;

	// Consensus stuff.
	#[runtime::pallet_index(11)]
	pub type Authorship = pallet_authorship;
	#[runtime::pallet_index(12)]
	pub type DarwiniaStaking = darwinia_staking;
	#[runtime::pallet_index(13)]
	pub type Session = pallet_session;
	#[runtime::pallet_index(14)]
	pub type Aura = pallet_aura;
	#[runtime::pallet_index(15)]
	pub type AuraExt = cumulus_pallet_aura_ext;
	// #[runtime::pallet_index(16)]
	// pub type MessageGadget = darwinia_message_gadget;
	// #[runtime::pallet_index(17)]
	// pub type EcdsaAuthority = darwinia_ecdsa_authority;

	// Governance stuff.
	// #[runtime::pallet_index(21)]
	// pub type PhragmenElection = pallet_elections_phragmen;
	// #[runtime::pallet_index(22)]
	// pub type TechnicalMembership = pallet_membership::<Instance1>;
	// #[runtime::pallet_index(19)]
	// pub type Council = pallet_collective::<Instance1>;
	#[runtime::pallet_index(20)]
	pub type TechnicalCommittee = pallet_collective<Instance2>;
	#[runtime::pallet_index(23)]
	pub type Treasury = pallet_treasury;
	// #[runtime::pallet_index(24)]
	// pub type Tips = pallet_tips;
	// #[runtime::pallet_index(18)]
	// pub type Democracy = pallet_democracy;
	#[runtime::pallet_index(44)]
	pub type ConvictionVoting = pallet_conviction_voting;
	#[runtime::pallet_index(45)]
	pub type Referenda = pallet_referenda;
	#[runtime::pallet_index(46)]
	pub type Origins = custom_origins;
	#[runtime::pallet_index(47)]
	pub type Whitelist = pallet_whitelist;

	// Utility stuff.
	// #[runtime::pallet_index(25)]
	// pub type Sudo = pallet_sudo;
	#[runtime::pallet_index(26)]
	pub type Utility = pallet_utility;
	// #[runtime::pallet_index(27)]
	// pub type Identity = pallet_identity;
	#[runtime::pallet_index(28)]
	pub type Scheduler = pallet_scheduler;
	#[runtime::pallet_index(29)]
	pub type Preimage = pallet_preimage;
	#[runtime::pallet_index(30)]
	pub type Proxy = pallet_proxy;
	#[runtime::pallet_index(48)]
	pub type TxPause = pallet_tx_pause;

	// XCM stuff.
	#[runtime::pallet_index(32)]
	pub type XcmpQueue = cumulus_pallet_xcmp_queue;
	#[runtime::pallet_index(33)]
	pub type PolkadotXcm = pallet_xcm;
	#[runtime::pallet_index(34)]
	pub type CumulusXcm = cumulus_pallet_xcm;
	// #[runtime::pallet_index(35)]
	// pub type DmpQueue = cumulus_pallet_dmp_queue;
	#[runtime::pallet_index(39)]
	pub type MessageQueue = pallet_message_queue;

	// EVM stuff.
	#[runtime::pallet_index(36)]
	pub type Ethereum = pallet_ethereum;
	#[runtime::pallet_index(37)]
	pub type EVM = pallet_evm;
	#[runtime::pallet_index(38)]
	pub type EthTxForwarder = darwinia_ethtx_forwarder;
}
// Create the runtime by composing the FRAME pallets that were previously configured.
#[cfg(feature = "dev")]
#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask
	)]
	pub struct Runtime;

	// System stuff.
	#[runtime::pallet_index(0)]
	pub type System = frame_system;
	#[runtime::pallet_index(1)]
	pub type ParachainSystem = cumulus_pallet_parachain_system;
	#[runtime::pallet_index(2)]
	pub type Timestamp = pallet_timestamp;
	#[runtime::pallet_index(3)]
	pub type ParachainInfo = parachain_info;

	// Monetary stuff.
	// Leave 4 here.
	// To keep balances consistent with the existing XCM configurations.
	#[runtime::pallet_index(5)]
	pub type Balances = pallet_balances;
	#[runtime::pallet_index(6)]
	pub type TransactionPayment = pallet_transaction_payment;
	#[runtime::pallet_index(7)]
	pub type Assets = pallet_assets;
	// #[runtime::pallet_index(8)]
	// pub type Vesting = pallet_vesting;
	#[runtime::pallet_index(9)]
	pub type Deposit = darwinia_deposit;
	#[runtime::pallet_index(10)]
	pub type AccountMigration = darwinia_account_migration;

	// Consensus stuff.
	#[runtime::pallet_index(11)]
	pub type Authorship = pallet_authorship;
	#[runtime::pallet_index(12)]
	pub type DarwiniaStaking = darwinia_staking;
	#[runtime::pallet_index(13)]
	pub type Session = pallet_session;
	#[runtime::pallet_index(14)]
	pub type Aura = pallet_aura;
	#[runtime::pallet_index(15)]
	pub type AuraExt = cumulus_pallet_aura_ext;
	// #[runtime::pallet_index(16)]
	// pub type MessageGadget = darwinia_message_gadget;
	// #[runtime::pallet_index(17)]
	// pub type EcdsaAuthority = darwinia_ecdsa_authority;

	// Governance stuff.
	// #[runtime::pallet_index(21)]
	// pub type PhragmenElection = pallet_elections_phragmen;
	// #[runtime::pallet_index(22)]
	// pub type TechnicalMembership = pallet_membership::<Instance1>;
	// #[runtime::pallet_index(19)]
	// pub type Council = pallet_collective::<Instance1>;
	#[runtime::pallet_index(20)]
	pub type TechnicalCommittee = pallet_collective<Instance2>;
	#[runtime::pallet_index(23)]
	pub type Treasury = pallet_treasury;
	// #[runtime::pallet_index(24)]
	// pub type Tips = pallet_tips;
	// #[runtime::pallet_index(18)]
	// pub type Democracy = pallet_democracy;
	#[runtime::pallet_index(44)]
	pub type ConvictionVoting = pallet_conviction_voting;
	#[runtime::pallet_index(45)]
	pub type Referenda = pallet_referenda;
	#[runtime::pallet_index(46)]
	pub type Origins = custom_origins;
	#[runtime::pallet_index(47)]
	pub type Whitelist = pallet_whitelist;

	// Utility stuff.
	// #[runtime::pallet_index(25)]
	// pub type Sudo = pallet_sudo;
	#[runtime::pallet_index(26)]
	pub type Utility = pallet_utility;
	// #[runtime::pallet_index(27)]
	// pub type Identity = pallet_identity;
	#[runtime::pallet_index(28)]
	pub type Scheduler = pallet_scheduler;
	#[runtime::pallet_index(29)]
	pub type Preimage = pallet_preimage;
	#[runtime::pallet_index(30)]
	pub type Proxy = pallet_proxy;
	#[runtime::pallet_index(48)]
	pub type TxPause = pallet_tx_pause;

	// XCM stuff.
	#[runtime::pallet_index(32)]
	pub type XcmpQueue = cumulus_pallet_xcmp_queue;
	#[runtime::pallet_index(33)]
	pub type PolkadotXcm = pallet_xcm;
	#[runtime::pallet_index(34)]
	pub type CumulusXcm = cumulus_pallet_xcm;
	// #[runtime::pallet_index(35)]
	// pub type DmpQueue = cumulus_pallet_dmp_queue;
	#[runtime::pallet_index(39)]
	pub type MessageQueue = pallet_message_queue;

	// EVM stuff.
	#[runtime::pallet_index(36)]
	pub type Ethereum = pallet_ethereum;
	#[runtime::pallet_index(37)]
	pub type EVM = pallet_evm;
	#[runtime::pallet_index(38)]
	pub type EthTxForwarder = darwinia_ethtx_forwarder;

	// Dev stuff.
	#[runtime::pallet_index(255)]
	pub type Sudo = pallet_sudo;
}

#[cfg(feature = "runtime-benchmarks")]
frame_benchmarking::define_benchmarks! {
	// darwinia
	[darwinia_account_migration, AccountMigration]
	[darwinia_deposit, Deposit]
	[darwinia_staking, DarwiniaStaking]
	// polkadot-sdk
	[cumulus_pallet_parachain_system, ParachainSystem]
	[cumulus_pallet_xcmp_queue, XcmpQueue]
	[frame_system, SystemBench::<Runtime>]
	[pallet_assets, Assets]
	[pallet_balances, Balances]
	[pallet_collective, TechnicalCommittee]
	[pallet_conviction_voting, ConvictionVoting]
	[pallet_message_queue, MessageQueue]
	[pallet_preimage, Preimage]
	[pallet_proxy, Proxy]
	[pallet_referenda, Referenda]
	[pallet_scheduler, Scheduler]
	[pallet_session, SessionBench::<Runtime>]
	[pallet_timestamp, Timestamp]
	[pallet_treasury, Treasury]
	[pallet_tx_pause, TxPause]
	[pallet_utility, Utility]
	[pallet_whitelist, Whitelist]
}

impl_self_contained_call!();

sp_api::impl_runtime_apis! {
	impl sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<sp_consensus_aura::sr25519::AuthorityId> {
			<pallet_aura::Authorities<Runtime>>::get().into_inner()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> sp_version::RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as sp_runtime::traits::Block>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> sp_core::OpaqueMetadata {
			sp_core::OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<sp_core::OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as sp_runtime::traits::Block>::Extrinsic) -> sp_runtime::ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as sp_runtime::traits::Block>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as sp_runtime::traits::Block>::Extrinsic> {
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
			source: sp_runtime::transaction_validity::TransactionSource,
			tx: <Block as sp_runtime::traits::Block>::Extrinsic,
			block_hash: <Block as sp_runtime::traits::Block>::Hash,
		) -> sp_runtime::transaction_validity::TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as sp_runtime::traits::Block>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_runtime::KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as sp_runtime::traits::Block>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as sp_runtime::traits::Block>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: frame_support::weights::Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: frame_support::weights::Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			frame_support::genesis_builder_helper::build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			frame_support::genesis_builder_helper::get_preset::<RuntimeGenesisConfig>(id, |_| None)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			vec![]
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as sp_runtime::traits::Block>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	impl cumulus_primitives_aura::AuraUnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
			included_hash: <Block as sp_runtime::traits::Block>::Hash,
			slot: cumulus_primitives_aura::Slot,
		) -> bool {
			ConsensusHook::can_build_upon(included_hash, slot)
		}
	}

	impl xcm_runtime_apis::fees::XcmPaymentApi<Block> for Runtime {
		fn query_acceptable_payment_assets(xcm_version: xcm::Version) -> Result<Vec<xcm::VersionedAssetId>, xcm_runtime_apis::fees::Error> {
			let acceptable_assets = vec![xcm::latest::AssetId(RelayLocation::get())];

			PolkadotXcm::query_acceptable_payment_assets(xcm_version, acceptable_assets)
		}

		fn query_weight_to_asset_fee(weight: frame_support::weights::Weight, asset: xcm::VersionedAssetId) -> Result<u128, xcm_runtime_apis::fees::Error> {
			// polkadot-sdk
			use frame_support::weights::WeightToFee as _;

			match asset.try_as::<xcm::latest::AssetId>() {
				Ok(asset_id) if asset_id.0 == RelayLocation::get() => {
					Ok(WeightToFee::weight_to_fee(&weight))
				},
				Ok(asset_id) => {
					log::trace!(target: "xcm::xcm_runtime_apis", "query_weight_to_asset_fee - unhandled asset_id: {asset_id:?}!");

					Err(xcm_runtime_apis::fees::Error::AssetNotFound)
				},
				Err(_) => {
					log::trace!(target: "xcm::xcm_runtime_apis", "query_weight_to_asset_fee - failed to convert asset: {asset:?}!");

					Err(xcm_runtime_apis::fees::Error::VersionedConversionFailed)
				}
			}
		}

		fn query_xcm_weight(message: xcm::VersionedXcm<()>) -> Result<frame_support::weights::Weight, xcm_runtime_apis::fees::Error> {
			PolkadotXcm::query_xcm_weight(message)
		}

		fn query_delivery_fees(destination: xcm::VersionedLocation, message: xcm::VersionedXcm<()>) -> Result<xcm::VersionedAssets, xcm_runtime_apis::fees::Error> {
			PolkadotXcm::query_delivery_fees(destination, message)
		}
	}

	impl xcm_runtime_apis::dry_run::DryRunApi<Block, RuntimeCall, RuntimeEvent, OriginCaller> for Runtime {
		fn dry_run_call(origin: OriginCaller, call: RuntimeCall) -> Result<xcm_runtime_apis::dry_run::CallDryRunEffects<RuntimeEvent>, xcm_runtime_apis::dry_run::Error> {
			PolkadotXcm::dry_run_call::<Runtime, XcmRouter, OriginCaller, RuntimeCall>(origin, call)
		}

		fn dry_run_xcm(origin_location: xcm::VersionedLocation, xcm: xcm::VersionedXcm<RuntimeCall>) -> Result<xcm_runtime_apis::dry_run::XcmDryRunEffects<RuntimeEvent>, xcm_runtime_apis::dry_run::Error> {
			PolkadotXcm::dry_run_xcm::<Runtime, XcmRouter, RuntimeCall, XcmExecutorConfig>(origin_location, xcm)
		}
	}

	impl xcm_runtime_apis::conversions::LocationToAccountApi<Block, AccountId> for Runtime {
		fn convert_location(location: xcm::VersionedLocation) -> Result<
			AccountId,
			xcm_runtime_apis::conversions::Error
		> {
			xcm_runtime_apis::conversions::LocationToAccountHelper::<
				AccountId,
				LocationToAccountId,
			>::convert_location(location)
		}
	}

	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			<<Runtime as pallet_evm::Config>::ChainId as sp_core::Get<u64>>::get()
		}

		fn account_basic(address: H160) -> pallet_evm::Account {
			let (account, _) = EVM::account_basic(&address);

			account
		}

		fn gas_price() -> U256 {
			// frontier
			use pallet_evm::FeeCalculator;

			let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();

			gas_price
		}

		fn account_code_at(address: H160) -> Vec<u8> {
			pallet_evm::AccountCodes::<Runtime>::get(address)
		}

		fn author() -> H160 {
			<pallet_evm::Pallet<Runtime>>::find_author()
		}

		fn storage_at(address: H160, index: U256) -> H256 {
			let mut tmp = [0u8; 32];

			index.to_big_endian(&mut tmp);

			pallet_evm::AccountStorages::<Runtime>::get(address, H256::from_slice(&tmp[..]))
		}

		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
			// frontier
			use pallet_evm::{Runner, GasWeightMapping as _};
			// polkadot-sdk
			use sp_runtime::traits::UniqueSaturatedInto;

			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			// Estimated encoded transaction size must be based on the heaviest transaction
			// type (EIP1559Transaction) to be compatible with all transaction types.
			let mut estimated_transaction_len = data.len() +
				// pallet ethereum index: 1
				// transact call index: 1
				// Transaction enum variant: 1
				// chain_id 8 bytes
				// nonce: 32
				// max_priority_fee_per_gas: 32
				// max_fee_per_gas: 32
				// gas_limit: 32
				// action: 21 (enum varianrt + call address)
				// value: 32
				// access_list: 1 (empty vec size)
				// 65 bytes signature
				258;

			if access_list.is_some() {
				estimated_transaction_len += access_list.encoded_size();
			}
			let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();
			let without_base_extrinsic_weight = true;

			let (weight_limit, proof_size_base_cost) =
				match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					gas_limit,
					without_base_extrinsic_weight
				) {
					weight_limit if weight_limit.proof_size() > 0 => {
						(Some(weight_limit), Some(estimated_transaction_len as u64))
					}
					_ => (None, None),
				};

			<Runtime as pallet_evm::Config>::Runner::call(
				from,
				to,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				false,
				true,
				weight_limit,
				proof_size_base_cost,
				config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.error.into())
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			// frontier
			use pallet_evm::{Runner, GasWeightMapping as _};
			// polkadot-sdk
			use sp_runtime::traits::UniqueSaturatedInto;

			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			let mut estimated_transaction_len = data.len() +
				// from: 20
				// value: 32
				// gas_limit: 32
				// nonce: 32
				// 1 byte transaction action variant
				// chain id 8 bytes
				// 65 bytes signature
				190;

			if max_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if max_priority_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if access_list.is_some() {
				estimated_transaction_len += access_list.encoded_size();
			}

			let gas_limit = if gas_limit > U256::from(u64::MAX) {
				u64::MAX
			} else {
				gas_limit.low_u64()
			};
			let without_base_extrinsic_weight = true;

			let (weight_limit, proof_size_base_cost) =
				match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					gas_limit,
					without_base_extrinsic_weight
				) {
					weight_limit if weight_limit.proof_size() > 0 => {
						(Some(weight_limit), Some(estimated_transaction_len as u64))
					}
					_ => (None, None),
				};

			<Runtime as pallet_evm::Config>::Runner::create(
				from,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				false,
				true,
				weight_limit,
				proof_size_base_cost,
				config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.error.into())
		}

		fn current_transaction_statuses() -> Option<Vec<fp_rpc::TransactionStatus>> {
			pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			pallet_ethereum::CurrentBlock::<Runtime>::get()
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			pallet_ethereum::CurrentReceipts::<Runtime>::get()
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<fp_rpc::TransactionStatus>>
		) {
			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentReceipts::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}

		fn extrinsic_filter(
			xts: Vec<<Block as sp_runtime::traits::Block>::Extrinsic>,
		) -> Vec<pallet_ethereum::Transaction> {
			xts.into_iter().filter_map(|xt| match xt.0.function {
				RuntimeCall::Ethereum(
					pallet_ethereum::Call::<Runtime>::transact { transaction }
				) => Some(transaction),
				_ => None
			}).collect::<Vec<pallet_ethereum::Transaction>>()
		}

		fn elasticity() -> Option<sp_runtime::Permill> {
			None
		}

		fn gas_limit_multiplier_support() {}

		fn pending_block(
			xts: Vec<<Block as sp_runtime::traits::Block>::Extrinsic>,
		) -> (Option<pallet_ethereum::Block>, Option<Vec<fp_rpc::TransactionStatus>>) {
			// polkadot-sdk
			use frame_support::traits::OnFinalize;

			for ext in xts.into_iter() {
				let _ = Executive::apply_extrinsic(ext);
			}

			Ethereum::on_finalize(System::block_number() + 1);

			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}

		fn initialize_pending_block(header: &<Block as sp_runtime::traits::Block>::Header) {
			Executive::initialize_block(header);
		}
	}

	impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
		fn convert_transaction(
			transaction: pallet_ethereum::Transaction
		) -> <Block as sp_runtime::traits::Block>::Extrinsic {
			UncheckedExtrinsic::new_unsigned(
				pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
			)
		}
	}

	impl moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block> for Runtime {
		fn trace_transaction(
			_extrinsics: Vec<<Block as sp_runtime::traits::Block>::Extrinsic>,
			_traced_transaction: &pallet_ethereum::Transaction,
			_header: &<Block as sp_runtime::traits::Block>::Header,
		) -> Result<
			(),
			sp_runtime::DispatchError,
		> {
			#[cfg(feature = "evm-tracing")]
			{
				use moonbeam_evm_tracer::tracer::EvmTracer;
				use xcm_primitives::{
					ETHEREUM_XCM_TRACING_STORAGE_KEY,
					EthereumXcmTracingStatus
				};
				use frame_support::storage::unhashed;

				// Tell the CallDispatcher we are tracing a specific Transaction.
				unhashed::put::<EthereumXcmTracingStatus>(
					ETHEREUM_XCM_TRACING_STORAGE_KEY,
					&EthereumXcmTracingStatus::Transaction(_traced_transaction.hash()),
				);

				// Initialize block: calls the "on_initialize" hook on every pallet
				// in AllPalletsWithSystem.
				// After pallet message queue was introduced, this must be done only after
				// enabling XCM tracing by setting ETHEREUM_XCM_TRACING_STORAGE_KEY
				// in the storage
				Executive::initialize_block(_header);

				// Apply the a subset of extrinsics: all the substrate-specific or ethereum
				// transactions that preceded the requested transaction.
				for ext in _extrinsics.into_iter() {
					let _ = match &ext.0.function {
						RuntimeCall::Ethereum(pallet_ethereum::Call::transact { transaction }) => {
							if transaction == _traced_transaction {
								EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
								return Ok(());
							} else {
								Executive::apply_extrinsic(ext)
							}
						}
						_ => Executive::apply_extrinsic(ext),
					};
					if let Some(EthereumXcmTracingStatus::TransactionExited) = unhashed::get(
						ETHEREUM_XCM_TRACING_STORAGE_KEY
					) {
						return Ok(());
					}
				}

				if let Some(EthereumXcmTracingStatus::Transaction(_)) = unhashed::get(
					ETHEREUM_XCM_TRACING_STORAGE_KEY
				) {
					// If the transaction was not found, it might be
					// an eth-xcm transaction that was executed at on_idle
					replay_on_idle();
				}

				if let Some(EthereumXcmTracingStatus::TransactionExited) = unhashed::get(
					ETHEREUM_XCM_TRACING_STORAGE_KEY
				) {
					// The transaction was found
					Ok(())
				} else {
					// The transaction was not-found
					Err(sp_runtime::DispatchError::Other(
						"Failed to find Ethereum transaction among the extrinsics.",
					))
				}
			}
			#[cfg(not(feature = "evm-tracing"))]
			Err(sp_runtime::DispatchError::Other(
				"Missing `evm-tracing` compile time feature flag.",
			))
		}

		fn trace_block(
			_extrinsics: Vec<<Block as sp_runtime::traits::Block>::Extrinsic>,
			_known_transactions: Vec<H256>,
			_header: &<Block as sp_runtime::traits::Block>::Header,
		) -> Result<
			(),
			sp_runtime::DispatchError,
		> {
			#[cfg(feature = "evm-tracing")]
			{
				use moonbeam_evm_tracer::tracer::EvmTracer;
				use xcm_primitives::EthereumXcmTracingStatus;

				// Tell the CallDispatcher we are tracing a full Block.
				frame_support::storage::unhashed::put::<EthereumXcmTracingStatus>(
					xcm_primitives::ETHEREUM_XCM_TRACING_STORAGE_KEY,
					&EthereumXcmTracingStatus::Block,
				);

				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;

				// Initialize block: calls the "on_initialize" hook on every pallet
				// in AllPalletsWithSystem.
				// After pallet message queue was introduced, this must be done only after
				// enabling XCM tracing by setting ETHEREUM_XCM_TRACING_STORAGE_KEY
				// in the storage
				Executive::initialize_block(_header);

				// Apply all extrinsics. Ethereum extrinsics are traced.
				for ext in _extrinsics.into_iter() {
					match &ext.0.function {
						RuntimeCall::Ethereum(pallet_ethereum::Call::transact { transaction }) => {
							if _known_transactions.contains(&transaction.hash()) {
								// Each known extrinsic is a new call stack.
								EvmTracer::emit_new();
								EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
							} else {
								let _ = Executive::apply_extrinsic(ext);
							}
						}
						_ => {
							let _ = Executive::apply_extrinsic(ext);
						}
					};
				}

				// Replay on_idle
				// Some XCM messages with eth-xcm transaction might be executed at on_idle
				replay_on_idle();

				Ok(())
			}
			#[cfg(not(feature = "evm-tracing"))]
			Err(sp_runtime::DispatchError::Other(
				"Missing `evm-tracing` compile time feature flag.",
			))
		}

		fn trace_call(
			_header: &<Block as sp_runtime::traits::Block>::Header,
			_from: H160,
			_to: H160,
			_data: Vec<u8>,
			_value: U256,
			_gas_limit: U256,
			_max_fee_per_gas: Option<U256>,
			_max_priority_fee_per_gas: Option<U256>,
			_nonce: Option<U256>,
			_access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<(), sp_runtime::DispatchError> {
			#[cfg(feature = "evm-tracing")]
			{
				use pallet_evm::{GasWeightMapping, Runner};
				use moonbeam_evm_tracer::tracer::EvmTracer;

				// Initialize block: calls the "on_initialize" hook on every pallet
				// in AllPalletsWithSystem.
				Executive::initialize_block(_header);

				EvmTracer::new().trace(|| {
					let is_transactional = false;
					let validate = true;
					let without_base_extrinsic_weight = true;


					// Estimated encoded transaction size must be based on the heaviest transaction
					// type (EIP1559Transaction) to be compatible with all transaction types.
					let mut estimated_transaction_len = _data.len() +
					// pallet ethereum index: 1
					// transact call index: 1
					// Transaction enum variant: 1
					// chain_id 8 bytes
					// nonce: 32
					// max_priority_fee_per_gas: 32
					// max_fee_per_gas: 32
					// gas_limit: 32
					// action: 21 (enum varianrt + call address)
					// value: 32
					// access_list: 1 (empty vec size)
					// 65 bytes signature
					258;

					if _access_list.is_some() {
						estimated_transaction_len += _access_list.encoded_size();
					}

					let gas_limit = _gas_limit.min(u64::MAX.into()).low_u64();

					let (weight_limit, proof_size_base_cost) =
						match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
							gas_limit,
							without_base_extrinsic_weight
						) {
							weight_limit if weight_limit.proof_size() > 0 => {
								(Some(weight_limit), Some(estimated_transaction_len as u64))
							}
							_ => (None, None),
						};

					let _ = <Runtime as pallet_evm::Config>::Runner::call(
						_from,
						_to,
						_data,
						_value,
						gas_limit,
						_max_fee_per_gas,
						_max_priority_fee_per_gas,
						_nonce,
						_access_list.unwrap_or_default(),
						is_transactional,
						validate,
						weight_limit,
						proof_size_base_cost,
						<Runtime as pallet_evm::Config>::config(),
					);
				});
				Ok(())
			}
			#[cfg(not(feature = "evm-tracing"))]
			Err(sp_runtime::DispatchError::Other(
				"Missing `evm-tracing` compile time feature flag.",
			))
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			// polkadot-sdk
			use frame_benchmarking::*;
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			// polkadot-sdk
			use frame_benchmarking::*;
			use frame_support::traits::TrackedStorageKey;

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {
				fn setup_set_code_requirements(code: &sp_std::vec::Vec<u8>) -> Result<(), BenchmarkError> {
					ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
					Ok(())
				}

				fn verify_set_code() {
					System::assert_last_event(cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into());
				}
			}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				array_bytes::dehexify_vec_then_into("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").unwrap(),
				// Total Issuance
				array_bytes::dehexify_vec_then_into("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").unwrap(),
				// Execution Phase
				array_bytes::dehexify_vec_then_into("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").unwrap(),
				// Event Count
				array_bytes::dehexify_vec_then_into("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").unwrap(),
				// System Events
				array_bytes::dehexify_vec_then_into("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").unwrap(),
			];

			let mut batches = <Vec<BenchmarkBatch>>::new();
			let params = (&config, &whitelist);

			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }

			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (frame_support::weights::Weight, frame_support::weights::Weight) {
			log::info!("try-runtime::on_runtime_upgrade");

			let weight = Executive::try_runtime_upgrade(checks).unwrap();

			(weight, pallet_config::RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect,
		) -> frame_support::weights::Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
		}
	}
}

// Helper function to replay the "on_idle" hook for all pallets, we need this for
// evm-tracing because some ethereum-xcm transactions might be executed at on_idle.
//
// We need to make sure that we replay on_idle exactly the same way as the
// original block execution, but unfortunately frame executive doesn't provide a function
// to replay only on_idle, so we need to copy here some code inside frame executive.
#[cfg(feature = "evm-tracing")]
fn replay_on_idle() {
	use frame_support::traits::OnIdle;
	use frame_system::pallet_prelude::BlockNumberFor;

	let weight = <frame_system::Pallet<Runtime>>::block_weight();
	let max_weight = pallet_config::RuntimeBlockWeights::get().max_block;
	let remaining_weight = max_weight.saturating_sub(weight.total());
	if remaining_weight.all_gt(frame_support::weights::Weight::zero()) {
		let _ = <AllPalletsWithSystem as OnIdle<BlockNumberFor<Runtime>>>::on_idle(
			<frame_system::Pallet<Runtime>>::block_number(),
			remaining_weight,
		);
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
