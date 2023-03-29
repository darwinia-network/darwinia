// This file is part of Darwinia.
//
// Copyright (C) 2018-2023 Darwinia Network
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
// TODO: address the unused crates in test.
#![cfg_attr(not(test), deny(unused_crate_dependencies))]
#![recursion_limit = "256"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod pallets;
pub use pallets::*;

mod bridges_message;
pub use bridges_message::*;

mod migration;
mod weights;

pub use darwinia_common_runtime::*;
pub use dc_primitives::*;

// substrate
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
	BridgeRejectObsoleteHeadersAndMessages,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic =
	fp_self_contained::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra, sp_core::H160>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	migration::CustomOnRuntimeUpgrade,
>;

/// Darwinia proposal base fee.
pub const DARWINIA_PROPOSAL_REQUIREMENT: Balance = 5_000 * UNIT;

/// Runtime version.
#[sp_version::runtime_version]
pub const VERSION: sp_version::RuntimeVersion = sp_version::RuntimeVersion {
	spec_name: sp_runtime::create_runtime_str!("Crab2"),
	impl_name: sp_runtime::create_runtime_str!("DarwiniaOfficialRust"),
	authoring_version: 0,
	spec_version: 6_0_2_0,
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
frame_support::construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = dc_primitives::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System stuff.
		System: frame_system = 0,
		ParachainSystem: cumulus_pallet_parachain_system = 1,
		Timestamp: pallet_timestamp = 2,
		ParachainInfo: parachain_info = 3,

		// Monetary stuff.
		// Leave 4 here.
		// To keep balances consistent with the existing XCM configurations.
		Balances: pallet_balances = 5,
		TransactionPayment: pallet_transaction_payment = 6,
		Assets: pallet_assets = 7,
		Vesting: pallet_vesting = 8,
		Deposit: darwinia_deposit = 9,
		AccountMigration: darwinia_account_migration = 10,

		// Consensus stuff.
		Authorship: pallet_authorship = 11,
		DarwiniaStaking: darwinia_staking = 12,
		Session: pallet_session = 13,
		Aura: pallet_aura = 14,
		AuraExt: cumulus_pallet_aura_ext = 15,
		MessageGadget: darwinia_message_gadget = 16,
		EcdsaAuthority: darwinia_ecdsa_authority = 17,

		// Governance stuff.
		Democracy: pallet_democracy = 18,
		Council: pallet_collective::<Instance1> = 19,
		TechnicalCommittee: pallet_collective::<Instance2> = 20,
		PhragmenElection: pallet_elections_phragmen = 21,
		TechnicalMembership: pallet_membership::<Instance1> = 22,
		Treasury: pallet_treasury = 23,
		Tips: pallet_tips = 24,

		// Utility stuff.
		Sudo: pallet_sudo = 25,
		Utility: pallet_utility = 26,
		Identity: pallet_identity = 27,
		Scheduler: pallet_scheduler = 28,
		Preimage: pallet_preimage = 29,
		Proxy: pallet_proxy = 30,

		// XCM stuff.
		XcmpQueue: cumulus_pallet_xcmp_queue = 32,
		PolkadotXcm: pallet_xcm = 33,
		CumulusXcm: cumulus_pallet_xcm = 34,
		DmpQueue: cumulus_pallet_dmp_queue = 35,

		// EVM stuff.
		Ethereum: pallet_ethereum = 36,
		EVM: pallet_evm = 37,
		MessageTransact: darwinia_message_transact = 38,

		// Crab <> Darwinia
		BridgePolkadotGrandpa: pallet_bridge_grandpa::<Instance1> = 39,
		BridgePolkadotParachain: pallet_bridge_parachains::<Instance1> = 40,
		BridgeDarwiniaMessages: pallet_bridge_messages::<Instance1> = 41,
		BridgeDarwiniaDispatch: pallet_bridge_dispatch::<Instance1> = 42,
		DarwiniaFeeMarket: pallet_fee_market::<Instance1> = 43
	}
}

#[cfg(feature = "runtime-benchmarks")]
frame_benchmarking::define_benchmarks! {
	// darwinia
	[darwinia_account_migration, AccountMigration]
	[darwinia_deposit, Deposit]
	[darwinia_ecdsa_authority, EcdsaAuthority]
	[darwinia_staking, DarwiniaStaking]
	// darwinia-messages-substrate
	[pallet_bridge_grandpa, BridgePolkadotGrandpa]
	[pallet_fee_market, DarwiniaFeeMarket]
	// substrate
	[cumulus_pallet_xcmp_queue, XcmpQueue]
	[frame_system, SystemBench::<Runtime>]
	[pallet_assets, Assets]
	[pallet_balances, Balances]
	[pallet_collective, Council]
	[pallet_collective, TechnicalCommittee]
	[pallet_democracy, Democracy]
	[pallet_elections_phragmen, PhragmenElection]
	[pallet_identity, Identity]
	[pallet_membership, TechnicalMembership]
	[pallet_preimage, Preimage]
	[pallet_proxy, Proxy]
	[pallet_scheduler, Scheduler]
	[pallet_tips, Tips]
	[pallet_treasury, Treasury]
	[pallet_utility, Utility]
	[pallet_vesting, Vesting]
	[pallet_session, SessionBench::<Runtime>]
	[pallet_timestamp, Timestamp]
}

impl_self_contained_call!();

bridge_runtime_common::generate_bridge_reject_obsolete_headers_and_messages! {
	RuntimeCall, AccountId,
	// Grandpa
	BridgePolkadotGrandpa,
	// Messages
	BridgeDarwiniaMessages,
	// Parachain
	BridgePolkadotParachain
}

sp_api::impl_runtime_apis! {
	impl sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<sp_consensus_aura::sr25519::AuthorityId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> sp_version::RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as sp_runtime::traits::Block>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> sp_core::OpaqueMetadata {
			sp_core::OpaqueMetadata::new(Runtime::metadata().into())
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

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as sp_runtime::traits::Block>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			<<Runtime as pallet_evm::Config>::ChainId as sp_core::Get<u64>>::get()
		}

		fn account_basic(address: sp_core::H160) -> pallet_evm::Account {
			let (account, _) = EVM::account_basic(&address);

			account
		}

		fn gas_price() -> sp_core::U256 {
			// frontier
			use pallet_evm::FeeCalculator;

			let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();

			gas_price
		}

		fn account_code_at(address: sp_core::H160) -> Vec<u8> {
			EVM::account_codes(address)
		}

		fn author() -> sp_core::H160 {
			<pallet_evm::Pallet<Runtime>>::find_author()
		}

		fn storage_at(address: sp_core::H160, index: sp_core::U256) -> sp_core::H256 {
			let mut tmp = [0u8; 32];

			index.to_big_endian(&mut tmp);

			EVM::account_storages(address, sp_core::H256::from_slice(&tmp[..]))
		}

		fn call(
			from: sp_core::H160,
			to: sp_core::H160,
			data: Vec<u8>,
			value: sp_core::U256,
			gas_limit: sp_core::U256,
			max_fee_per_gas: Option<sp_core::U256>,
			max_priority_fee_per_gas: Option<sp_core::U256>,
			nonce: Option<sp_core::U256>,
			estimate: bool,
			access_list: Option<Vec<(sp_core::H160, Vec<sp_core::H256>)>>,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
			// frontier
			use pallet_evm::Runner;
			// substrate
			use sp_runtime::traits::UniqueSaturatedInto;

			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			let is_transactional = false;
			let validate = true;
			#[allow(clippy::or_fun_call)]
			let evm_config = config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config());
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
				is_transactional,
				validate,
				evm_config,
			).map_err(|err| err.error.into())
		}

		fn create(
			from: sp_core::H160,
			data: Vec<u8>,
			value: sp_core::U256,
			gas_limit: sp_core::U256,
			max_fee_per_gas: Option<sp_core::U256>,
			max_priority_fee_per_gas: Option<sp_core::U256>,
			nonce: Option<sp_core::U256>,
			estimate: bool,
			access_list: Option<Vec<(sp_core::H160, Vec<sp_core::H256>)>>,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			// frontier
			use pallet_evm::Runner;
			// substrate
			use sp_runtime::traits::UniqueSaturatedInto;

			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			let is_transactional = false;
			let validate = true;
			#[allow(clippy::or_fun_call)]
			let evm_config = config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config());
			<Runtime as pallet_evm::Config>::Runner::create(
				from,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				is_transactional,
				validate,
				evm_config,
			).map_err(|err| err.error.into())
		}

		fn current_transaction_statuses() -> Option<Vec<fp_rpc::TransactionStatus>> {
			Ethereum::current_transaction_statuses()
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			Ethereum::current_block()
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			Ethereum::current_receipts()
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<fp_rpc::TransactionStatus>>
		) {
			(
				Ethereum::current_block(),
				Ethereum::current_receipts(),
				Ethereum::current_transaction_statuses()
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

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			// substrate
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
			// substrate
			use frame_benchmarking::*;

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				array_bytes::hex_into_unchecked("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac"),
				// Total Issuance
				array_bytes::hex_into_unchecked("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80"),
				// Execution Phase
				array_bytes::hex_into_unchecked("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a"),
				// Event Count
				array_bytes::hex_into_unchecked("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850"),
				// System Events
				array_bytes::hex_into_unchecked("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7"),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }

			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (frame_support::weights::Weight, frame_support::weights::Weight) {
			// substrate
			use frame_support::log;

			log::info!("try-runtime::on_runtime_upgrade");

			let weight = Executive::try_runtime_upgrade(checks).unwrap();

			(weight, RuntimeBlockWeights::get().max_block)
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

struct CheckInherents;
impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}
cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
