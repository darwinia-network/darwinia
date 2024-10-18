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
// frontier
use pallet_evm::{ExitError, IsPrecompileResult, Precompile};
use pallet_evm_precompile_dispatch::DispatchValidateT;
// polkadot-sdk
use frame_support::dispatch::{DispatchClass, GetDispatchInfo, Pays};

frame_support::parameter_types! {
	pub PrecompilesValue: CrabPrecompiles<Runtime> = CrabPrecompiles::<_>::new();
}

pub struct CrabPrecompiles<R>(core::marker::PhantomData<R>);
impl<R> CrabPrecompiles<R>
where
	R: pallet_evm::Config,
{
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self(Default::default())
	}

	pub fn used_addresses() -> [H160; 16] {
		[
			addr(0x01),
			addr(0x02),
			addr(0x03),
			addr(0x04),
			addr(0x05),
			addr(0x06),
			addr(0x07),
			addr(0x08),
			addr(0x09),
			addr(0x400),
			addr(0x401),
			addr(0x402), // For KTON asset.
			addr(0x600),
			addr(0x601),
			addr(0x602),
			addr(0x800),
		]
	}
}
impl<R> pallet_evm::PrecompileSet for CrabPrecompiles<R>
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
			a if a == addr(0x01) => Some(pallet_evm_precompile_simple::ECRecover::execute(handle)),
			a if a == addr(0x02) => Some(pallet_evm_precompile_simple::Sha256::execute(handle)),
			a if a == addr(0x03) => Some(pallet_evm_precompile_simple::Ripemd160::execute(handle)),
			a if a == addr(0x04) => Some(pallet_evm_precompile_simple::Identity::execute(handle)),
			a if a == addr(0x05) => Some(pallet_evm_precompile_modexp::Modexp::execute(handle)),
			a if a == addr(0x06) => Some(pallet_evm_precompile_bn128::Bn128Add::execute(handle)),
			a if a == addr(0x07) => Some(pallet_evm_precompile_bn128::Bn128Mul::execute(handle)),
			a if a == addr(0x08) => Some(pallet_evm_precompile_bn128::Bn128Pairing::execute(handle)),
			a if a == addr(0x09) => Some(pallet_evm_precompile_blake2::Blake2F::execute(handle)),
			// Darwinia precompiles: [0x400, 0x800) for stable precompiles.
			a if a == addr(0x400) => Some(<darwinia_precompile_state_storage::StateStorage<
				Runtime,
				darwinia_precompile_state_storage::StateStorageFilter,
			>>::execute(handle)),
			a if a == addr(0x401) => Some(<pallet_evm_precompile_dispatch::Dispatch<
				Runtime,
				DarwiniaDispatchValidator,
			>>::execute(handle)),
			// [0x402, 0x600) reserved for assets precompiles.
			a if (0x402..0x600).contains(&AssetIdConverter::account_to_asset_id(a.into())) =>
				Some(<darwinia_precompile_assets::ERC20Assets<Runtime, AssetIdConverter>>::execute(
					handle,
				)),
			// [0x600, 0x800) reserved for other stable precompiles.
			a if a == addr(0x602) =>
				Some(<pallet_evm_precompile_conviction_voting::ConvictionVotingPrecompile<Runtime>>::execute(handle)),
			// [0x800..) reserved for the experimental precompiles.
			a if a == addr(0x800) => Some(Err(precompile_utils::prelude::revert("This precompile is no longer supported."))),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
		IsPrecompileResult::Answer {
			is_precompile: Self::used_addresses().contains(&address),
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

fn addr(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}

impl pallet_evm::Config for Runtime {
	type AddressMapping = pallet_evm::IdentityAddressMapping;
	type BlockGasLimit = pallet_config::BlockGasLimit;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<Self::AccountId>;
	type ChainId = ConstU64<44>;
	type Currency = Balances;
	type FeeCalculator = TransactionPaymentGasPrice;
	type FindAuthor = FindAuthor<pallet_session::FindAccountFromAuthorIndex<Self, Aura>>;
	type GasLimitPovSizeRatio = pallet_config::GasLimitPovSizeRatio;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type OnChargeTransaction = pallet_evm::EVMFungibleAdapter<Balances, ()>;
	type OnCreate = ();
	type PrecompilesType = CrabPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type RuntimeEvent = RuntimeEvent;
	type SuicideQuickClearLimit = ();
	type Timestamp = Timestamp;
	type WeightInfo = ();
	type WeightPerGas = pallet_config::WeightPerGas;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<Self::AccountId>;
}
