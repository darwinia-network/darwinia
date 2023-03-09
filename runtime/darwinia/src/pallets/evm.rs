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
// frontier
use pallet_evm::Precompile;

const BLOCK_GAS_LIMIT: u64 = 10_000_000;
frame_support::parameter_types! {
	pub BlockGasLimit: sp_core::U256 = sp_core::U256::from(BLOCK_GAS_LIMIT);
	pub PrecompilesValue: DarwiniaPrecompiles<Runtime> = DarwiniaPrecompiles::<_>::new();
	pub WeightPerGas: frame_support::weights::Weight = frame_support::weights::Weight::from_ref_time(fp_evm::weight_per_gas(BLOCK_GAS_LIMIT, NORMAL_DISPATCH_RATIO, WEIGHT_MILLISECS_PER_BLOCK));
}

// TODO: Integrate to the upstream repo
pub struct FromH160;
impl<T> pallet_evm::AddressMapping<T> for FromH160
where
	T: From<sp_core::H160>,
{
	fn into_account_id(address: sp_core::H160) -> T {
		address.into()
	}
}

pub struct DarwiniaPrecompiles<R>(sp_std::marker::PhantomData<R>);
impl<R> DarwiniaPrecompiles<R>
where
	R: pallet_evm::Config,
{
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self(Default::default())
	}

	pub fn used_addresses() -> [sp_core::H160; 16] {
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
			// For KTON asset.
			addr(1026),
			addr(1536),
			addr(1537),
			// Test only
			addr(2047),
			addr(2048),
		]
	}
}
impl<R> pallet_evm::PrecompileSet for DarwiniaPrecompiles<R>
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
		if self.is_precompile(code_addr) && code_addr > addr(9) && code_addr != context_addr {
			return Some(Err(precompile_utils::revert(
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
			a if a == addr(1025) =>
				Some(<pallet_evm_precompile_dispatch::Dispatch<Runtime>>::execute(handle)),
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
			a if a == addr(2047) =>
				Some(<darwinia_precompile_ark_bls12_381::ARKBLS12381<Runtime>>::execute(handle)),
			// [2048..) reserved for the experimental precompiles.
			a if a == addr(2048) =>
				Some(<darwinia_precompile_bls12_381::BLS12381<Runtime>>::execute(handle)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: sp_core::H160) -> bool {
		Self::used_addresses().contains(&address)
	}
}

impl pallet_evm::Config for Runtime {
	type AddressMapping = FromH160;
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type ChainId = ConstU64<46>;
	type Currency = Balances;
	type FeeCalculator = FixedGasPrice;
	type FindAuthor = DarwiniaFindAuthor<pallet_session::FindAccountFromAuthorIndex<Self, Aura>>;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	type PrecompilesType = DarwiniaPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type RuntimeEvent = RuntimeEvent;
	type WeightPerGas = WeightPerGas;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
}

fn addr(a: u64) -> sp_core::H160 {
	sp_core::H160::from_low_u64_be(a)
}
