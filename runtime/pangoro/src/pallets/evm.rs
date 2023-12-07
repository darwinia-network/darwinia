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

// darwinia
use crate::*;
// substrate
use cumulus_primitives_core::relay_chain::MAX_POV_SIZE;
use frame_support::dispatch::{DispatchClass, GetDispatchInfo, Pays};
// frontier
use pallet_evm::{ExitError, IsPrecompileResult, Precompile};
use pallet_evm_precompile_dispatch::DispatchValidateT;

const BLOCK_GAS_LIMIT: u64 = 20_000_000;
frame_support::parameter_types! {
	pub BlockGasLimit: sp_core::U256 = sp_core::U256::from(BLOCK_GAS_LIMIT);
	// Restrict the POV size of the Ethereum transactions in the same way as weight limit.
	pub BlockPovSizeLimit: u64 = NORMAL_DISPATCH_RATIO * MAX_POV_SIZE as u64;
	pub PrecompilesValue: PangoroPrecompiles<Runtime> = PangoroPrecompiles::<_>::new();
	pub WeightPerGas: frame_support::weights::Weight = frame_support::weights::Weight::from_parts(
		fp_evm::weight_per_gas(BLOCK_GAS_LIMIT, NORMAL_DISPATCH_RATIO, WEIGHT_MILLISECS_PER_BLOCK),
		0
	);
	// TODO: FIX ME. https://github.com/rust-lang/rust/issues/88581
	pub GasLimitPovSizeRatio: u64 = BLOCK_GAS_LIMIT.saturating_div(BlockPovSizeLimit::get()) + 1;
}

pub struct PangoroPrecompiles<R>(core::marker::PhantomData<R>);
impl<R> PangoroPrecompiles<R>
where
	R: pallet_evm::Config,
{
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self(Default::default())
	}

	pub fn used_addresses() -> [sp_core::H160; 15] {
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
			addr(1026), // For KTON asset.
			addr(1536),
			addr(1537),
			addr(2048),
		]
	}
}
impl<R> pallet_evm::PrecompileSet for PangoroPrecompiles<R>
where
	R: pallet_evm::Config,
{
	fn execute(
		&self,
		handle: &mut impl pallet_evm::PrecompileHandle,
	) -> Option<pallet_evm::PrecompileResult> {
		// darwinia
		use darwinia_precompile_assets::AccountToAssetId;

		let (code_addr, context_addr) = (handle.code_address(), handle.context().address);
		// Filter known precompile addresses except Ethereum officials
		if Self::used_addresses().contains(&code_addr)
			&& code_addr > addr(9)
			&& code_addr != context_addr
		{
			return Some(Err(precompile_utils::prelude::revert(
				"cannot be called with DELEGATECALL or CALLCODE",
			)));
		};

		match code_addr {
			// Ethereum precompiles:
			a if a == addr(1) => Some(pallet_evm_precompile_simple::ECRecover::execute(handle)),
			a if a == addr(2) => Some(pallet_evm_precompile_simple::Sha256::execute(handle)),
			a if a == addr(3) => Some(pallet_evm_precompile_simple::Ripemd160::execute(handle)),
			a if a == addr(4) => Some(pallet_evm_precompile_simple::Identity::execute(handle)),
			a if a == addr(5) => Some(pallet_evm_precompile_modexp::Modexp::execute(handle)),
			a if a == addr(6) => Some(pallet_evm_precompile_bn128::Bn128Add::execute(handle)),
			a if a == addr(7) => Some(pallet_evm_precompile_bn128::Bn128Mul::execute(handle)),
			a if a == addr(8) => Some(pallet_evm_precompile_bn128::Bn128Pairing::execute(handle)),
			a if a == addr(9) => Some(pallet_evm_precompile_blake2::Blake2F::execute(handle)),
			// Darwinia precompiles: [1024, 2048) for stable precompiles.
			a if a == addr(1024) => Some(<darwinia_precompile_state_storage::StateStorage<
				Runtime,
				darwinia_precompile_state_storage::StateStorageFilter,
			>>::execute(handle)),
			a if a == addr(1025) => Some(<pallet_evm_precompile_dispatch::Dispatch<
				Runtime,
				DarwiniaDispatchValidator,
			>>::execute(handle)),
			// [1026, 1536) reserved for assets precompiles.
			a if (1026..1536).contains(&AssetIdConverter::account_to_asset_id(a.into())) =>
				Some(<darwinia_precompile_assets::ERC20Assets<Runtime, AssetIdConverter>>::execute(
					handle,
				)),
			// [1536, 2048) reserved for other stable precompiles.
			a if a == addr(1536) =>
				Some(<darwinia_precompile_deposit::Deposit<Runtime>>::execute(handle)),
			a if a == addr(1537) =>
				Some(<darwinia_precompile_staking::Staking<Runtime>>::execute(handle)),
			// [2048..) reserved for the experimental precompiles.
			a if a == addr(2048) =>
				Some(<darwinia_precompile_bls12_381::BLS12381<Runtime>>::execute(handle)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: sp_core::H160, _gas: u64) -> IsPrecompileResult {
		IsPrecompileResult::Answer {
			is_precompile: Self::used_addresses().contains(&address),
			extra_cost: 0,
		}
	}
}

pub struct TransactionPaymentGasPrice;
impl pallet_evm::FeeCalculator for TransactionPaymentGasPrice {
	fn min_gas_price() -> (sp_core::U256, frame_support::weights::Weight) {
		// substrate
		use frame_support::weights::WeightToFee;
		use sp_runtime::FixedPointNumber;
		(
			TransactionPayment::next_fee_multiplier()
				.saturating_mul_int::<Balance>(
					<Runtime as pallet_transaction_payment::Config>::WeightToFee::weight_to_fee(
						&WeightPerGas::get(),
					),
				)
				.into(),
			<Runtime as frame_system::Config>::DbWeight::get().reads(1),
		)
	}
}

impl pallet_evm::Config for Runtime {
	type AddressMapping = pallet_evm::IdentityAddressMapping;
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type ChainId = ConstU64<45>;
	type Currency = Balances;
	type FeeCalculator = TransactionPaymentGasPrice;
	type FindAuthor = FindAuthor<pallet_session::FindAccountFromAuthorIndex<Self, Aura>>;
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	type PrecompilesType = PangoroPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type RuntimeEvent = RuntimeEvent;
	type Timestamp = Timestamp;
	type WeightInfo = ();
	type WeightPerGas = WeightPerGas;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
}

fn addr(a: u64) -> sp_core::H160 {
	sp_core::H160::from_low_u64_be(a)
}

/// Validation rule for dispatch precompile
pub struct DarwiniaDispatchValidator;
impl DispatchValidateT<AccountId, RuntimeCall> for DarwiniaDispatchValidator {
	fn validate_before_dispatch(
		_origin: &AccountId,
		call: &RuntimeCall,
	) -> Option<fp_evm::PrecompileFailure> {
		let info = call.get_dispatch_info();

		if matches!(
			call,
			RuntimeCall::Assets(..)
				| RuntimeCall::Ethereum(..)
				| RuntimeCall::EVM(..)
				| RuntimeCall::MessageTransact(..)
		) {
			Some(fp_evm::PrecompileFailure::Error {
				exit_status: ExitError::Other(
					"These pallet's calls are not allowed to be called from precompile.".into(),
				),
			})
		} else if info.pays_fee == Pays::No || info.class == DispatchClass::Mandatory {
			Some(fp_evm::PrecompileFailure::Error {
				exit_status: ExitError::Other("Permission denied calls".into()),
			})
		} else {
			None
		}
	}
}
