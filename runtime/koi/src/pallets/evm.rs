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

// darwinia
use crate::*;
use pallet_config::precompiles::{self, *};
// frontier
use pallet_evm::{ExitError, IsPrecompileResult, Precompile};
use pallet_evm_precompile_dispatch::DispatchValidateT;
// polkadot-sdk
use frame_support::dispatch::{DispatchClass, GetDispatchInfo, Pays};

frame_support::parameter_types! {
	pub PrecompilesValue: Precompiles = Precompiles;
}

pub struct Precompiles;
impl Precompiles {
	pub const fn set() -> [[u8; 20]; 25] {
		[
			ADDR_EC_RECOVER,
			ADDR_SHA256,
			ADDR_RIPEMD160,
			ADDR_IDENTITY,
			ADDR_MODEXP,
			ADDR_BN128_ADD,
			ADDR_BN128_MUL,
			ADDR_BN128_PAIRING,
			ADDR_BLAKE2F,
			ADDR_BLS12381_G1_ADD,
			ADDR_BLS12381_G1_MUL,
			ADDR_BLS12381_G1_MULTI_EXP,
			ADDR_BLS12381_G2_ADD,
			ADDR_BLS12381_G2_MUL,
			ADDR_BLS12381_G2_MULTI_EXP,
			ADDR_BLS12381_PAIRING,
			ADDR_BLS12381_MAP_G1,
			ADDR_BLS12381_MAP_G2,
			ADDR_STATE_STORAGE,
			ADDR_DISPATCH,
			ADDR_KTON,
			ADDR_USDT,
			ADDR_DOT,
			ADDR_CONVICTION_VOTING,
			ADDR_EXPERIMENTAL,
		]
	}
}
impl pallet_evm::PrecompileSet for Precompiles {
	fn execute(
		&self,
		handle: &mut impl pallet_evm::PrecompileHandle,
	) -> Option<pallet_evm::PrecompileResult> {
		// darwinia
		use darwinia_precompile_assets::AccountToAssetId;

		let (code_addr, context_addr) = (handle.code_address().0, handle.context().address.0);

		// Filter known precompile addresses except Ethereum officials.
		if Self::set().contains(&code_addr)
			&& code_addr > precompiles::address_of(9)
			&& code_addr != context_addr
		{
			return Some(Err(precompile_utils::prelude::revert(
				"Cannot be called using `DELEGATECALL` or `CALLCODE`.",
			)));
		};

		let output = match code_addr {
			ADDR_EC_RECOVER => pallet_evm_precompile_simple::ECRecover::execute(handle),
			ADDR_SHA256 => pallet_evm_precompile_simple::Sha256::execute(handle),
			ADDR_RIPEMD160 => pallet_evm_precompile_simple::Ripemd160::execute(handle),
			ADDR_IDENTITY => pallet_evm_precompile_simple::Identity::execute(handle),
			ADDR_MODEXP => pallet_evm_precompile_modexp::Modexp::execute(handle),
			ADDR_BN128_ADD => pallet_evm_precompile_bn128::Bn128Add::execute(handle),
			ADDR_BN128_MUL => pallet_evm_precompile_bn128::Bn128Mul::execute(handle),
			ADDR_BN128_PAIRING => pallet_evm_precompile_bn128::Bn128Pairing::execute(handle),
			ADDR_BLAKE2F => pallet_evm_precompile_blake2::Blake2F::execute(handle),
			ADDR_BLS12381_G1_ADD => pallet_evm_precompile_bls12381::Bls12381G1Add::execute(handle),
			ADDR_BLS12381_G1_MUL => pallet_evm_precompile_bls12381::Bls12381G1Mul::execute(handle),
			ADDR_BLS12381_G1_MULTI_EXP =>
				pallet_evm_precompile_bls12381::Bls12381G1MultiExp::execute(handle),
			ADDR_BLS12381_G2_ADD => pallet_evm_precompile_bls12381::Bls12381G2Add::execute(handle),
			ADDR_BLS12381_G2_MUL => pallet_evm_precompile_bls12381::Bls12381G2Mul::execute(handle),
			ADDR_BLS12381_G2_MULTI_EXP =>
				pallet_evm_precompile_bls12381::Bls12381G2MultiExp::execute(handle),
			ADDR_BLS12381_PAIRING => pallet_evm_precompile_bls12381::Bls12381Pairing::execute(handle),
			ADDR_BLS12381_MAP_G1 => pallet_evm_precompile_bls12381::Bls12381MapG1::execute(handle),
			ADDR_BLS12381_MAP_G2 => pallet_evm_precompile_bls12381::Bls12381MapG2::execute(handle),
			ADDR_STATE_STORAGE => <darwinia_precompile_state_storage::StateStorage<
				Runtime,
				darwinia_precompile_state_storage::StateStorageFilter,
			>>::execute(handle),
			ADDR_DISPATCH => <pallet_evm_precompile_dispatch::Dispatch<
				Runtime,
				DarwiniaDispatchValidator,
			>>::execute(handle),
			a if (0x402..0x600).contains(&AssetIdConverter::account_to_asset_id(a.into())) =>
				<darwinia_precompile_assets::ERC20Assets<Runtime, AssetIdConverter>>::execute(
					handle,
				),
			ADDR_CONVICTION_VOTING =>
				<pallet_evm_precompile_conviction_voting::ConvictionVotingPrecompile<Runtime>>::execute(handle),
			ADDR_EXPERIMENTAL | ADDR_DEPOSIT_DEPRECATED  | ADDR_STAKING_DEPRECATED =>
				Err(precompile_utils::prelude::revert("This precompile is not supported.")),
			_ => return None,
		};

		Some(output)
	}

	fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
		IsPrecompileResult::Answer {
			is_precompile: Self::set().contains(&address.0),
			extra_cost: 0,
		}
	}
}

pub struct TransactionPaymentGasPrice;
impl pallet_evm::FeeCalculator for TransactionPaymentGasPrice {
	fn min_gas_price() -> (U256, frame_support::weights::Weight) {
		// polkadot-sdk
		use frame_support::weights::WeightToFee;
		use sp_runtime::FixedPointNumber;

		
		(
			TransactionPayment::next_fee_multiplier()
				.saturating_mul_int::<Balance>(
					<Runtime as pallet_transaction_payment::Config>::WeightToFee::weight_to_fee(
						&pallet_config::WeightPerGas::get(),
					),
				)
				.into(),
			<Runtime as frame_system::Config>::DbWeight::get().reads(1),
		)
	}
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
				| RuntimeCall::EthTxForwarder(..)
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

impl pallet_evm::Config for Runtime {
	type AddressMapping = pallet_evm::IdentityAddressMapping;
	type BlockGasLimit = pallet_config::BlockGasLimit;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<Self::AccountId>;
	type ChainId = ConstU64<701>;
	type Currency = Balances;
	type FeeCalculator = TransactionPaymentGasPrice;
	type FindAuthor = FindAuthor<pallet_session::FindAccountFromAuthorIndex<Self, Aura>>;
	type GasLimitPovSizeRatio = pallet_config::GasLimitPovSizeRatio;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type OnChargeTransaction = pallet_evm::EVMFungibleAdapter<Balances, ()>;
	type OnCreate = ();
	type PrecompilesType = Precompiles;
	type PrecompilesValue = PrecompilesValue;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type RuntimeEvent = RuntimeEvent;
	type SuicideQuickClearLimit = ();
	type Timestamp = Timestamp;
	type WeightInfo = ();
	type WeightPerGas = pallet_config::WeightPerGas;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<Self::AccountId>;
}
