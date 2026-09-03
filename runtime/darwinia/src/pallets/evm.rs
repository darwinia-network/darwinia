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
	pub fn set() -> [[u8; 20]; 17] {
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
			ADDR_STATE_STORAGE,
			ADDR_DISPATCH,
			ADDR_KTON,
			ADDR_USDT,
			ADDR_PINK,
			ADDR_DOT,
			ADDR_CONVICTION_VOTING,
			ADDR_EXPERIMENTAL,
		]
	}

	fn is_asset_precompile(address: [u8; 20]) -> bool {
		// Compare the full address so a non-zero prefix cannot alias an asset precompile.
		(ADDR_KTON..ADDR_DEPOSIT_DEPRECATED).contains(&address)
	}

	fn is_precompile_address(address: [u8; 20]) -> bool {
		Self::set().contains(&address) || Self::is_asset_precompile(address)
	}

	fn precompile_context_error(
		code_address: [u8; 20],
		context_address: [u8; 20],
	) -> Option<&'static str> {
		(Self::is_precompile_address(code_address)
			&& code_address > precompiles::address_of(9)
			&& code_address != context_address)
			.then_some("Cannot be called using `DELEGATECALL` or `CALLCODE`.")
	}
}
impl pallet_evm::PrecompileSet for Precompiles {
	fn execute(
		&self,
		handle: &mut impl pallet_evm::PrecompileHandle,
	) -> Option<pallet_evm::PrecompileResult> {
		let (code_addr, context_addr) = (handle.code_address().0, handle.context().address.0);

		if let Some(message) = Self::precompile_context_error(code_addr, context_addr) {
			return Some(Err(precompile_utils::prelude::revert(message)));
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
			ADDR_STATE_STORAGE => <darwinia_precompile_state_storage::StateStorage<
				Runtime,
				darwinia_precompile_state_storage::StateStorageFilter,
			>>::execute(handle),
			ADDR_DISPATCH => <pallet_evm_precompile_dispatch::Dispatch<
				Runtime,
				DarwiniaDispatchValidator,
			>>::execute(handle),
			a if Self::is_asset_precompile(a) =>
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
			is_precompile: Self::is_precompile_address(address.0),
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
	type AccountProvider = pallet_evm::FrameSystemAccountProvider<Self>;
	type AddressMapping = pallet_evm::IdentityAddressMapping;
	type BlockGasLimit = pallet_config::BlockGasLimit;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<Self::AccountId>;
	type ChainId = ConstU64<46>;
	type Currency = Balances;
	type FeeCalculator = TransactionPaymentGasPrice;
	type FindAuthor = FindAuthor<pallet_session::FindAccountFromAuthorIndex<Self, Aura>>;
	type GasLimitPovSizeRatio = pallet_config::GasLimitPovSizeRatio;
	type GasLimitStorageGrowthRatio = pallet_config::GasLimitStorageGrowthRatio;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type OnChargeTransaction = pallet_evm::EVMFungibleAdapter<Balances, ()>;
	type OnCreate = ();
	type PrecompilesType = Precompiles;
	type PrecompilesValue = PrecompilesValue;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type RuntimeEvent = RuntimeEvent;
	type Timestamp = Timestamp;
	type WeightInfo = ();
	type WeightPerGas = pallet_config::WeightPerGas;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<Self::AccountId>;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn asset_precompile_alias_is_not_a_precompile() {
		let canonical = precompiles::address_of(0x402);
		let mut alias = canonical;
		alias[11] = 1;

		assert_eq!(H160::from(alias).to_low_u64_be(), 0x402);
		assert!(Precompiles::is_asset_precompile(canonical));
		assert!(!Precompiles::is_asset_precompile(alias));
		assert!(!Precompiles::is_precompile_address(alias));
		assert_eq!(Precompiles::precompile_context_error(alias, alias), None);
		assert_eq!(Precompiles::precompile_context_error(alias, [0xAA; 20]), None);
	}

	#[test]
	fn asset_precompile_rejects_foreign_execution_context() {
		let canonical = precompiles::address_of(0x402);

		assert_eq!(Precompiles::precompile_context_error(canonical, canonical), None);
		assert_eq!(
			Precompiles::precompile_context_error(canonical, [0xAA; 20]),
			Some("Cannot be called using `DELEGATECALL` or `CALLCODE`.")
		);
	}

	#[test]
	fn every_non_zero_asset_address_prefix_is_outside_the_precompile_range() {
		let canonical = precompiles::address_of(0x402);

		for byte_index in 0..12 {
			let mut alias = canonical;
			alias[byte_index] = 1;

			assert!(!Precompiles::is_asset_precompile(alias));
			assert!(!Precompiles::is_precompile_address(alias));
		}
	}

	#[test]
	fn dynamic_asset_precompiles_are_reported_consistently() {
		assert!(Precompiles::is_precompile_address(precompiles::address_of(0x402)));
		assert!(Precompiles::is_precompile_address(precompiles::address_of(0x500)));
		assert!(Precompiles::is_precompile_address(precompiles::address_of(0x5ff)));
		assert!(!Precompiles::is_asset_precompile(precompiles::address_of(0x401)));
		assert!(!Precompiles::is_asset_precompile(precompiles::address_of(0x600)));
	}
}
