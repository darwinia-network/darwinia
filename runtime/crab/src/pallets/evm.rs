// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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
use darwinia_precompile_assets::{AccountToAssetId, ERC20Assets};
use darwinia_precompile_bls12_381::BLS12381;
use darwinia_precompile_state_storage::{EthereumStorageFilter, StateStorage};
// frontier
use pallet_ethereum::EthereumBlockHashMapping;
use pallet_evm::{
	AddressMapping, EnsureAddressNever, EnsureAddressRoot, FeeCalculator, FixedGasWeightMapping,
	Precompile, PrecompileHandle, PrecompileResult, PrecompileSet,
};
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
// substrate
use frame_support::{traits::FindAuthor, ConsensusEngineId};
use sp_core::crypto::ByteArray;
use sp_std::marker::PhantomData;

const WEIGHT_PER_GAS: u64 = 40_000;

frame_support::parameter_types! {
	pub BlockGasLimit: U256 = U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT.ref_time() / WEIGHT_PER_GAS);
	pub PrecompilesValue: CrabPrecompiles<Runtime> = CrabPrecompiles::<_>::new();
	pub WeightPerGas: Weight = Weight::from_ref_time(WEIGHT_PER_GAS);
}

pub struct FindAuthorTruncated<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		F::find_author(digests).and_then(|i| {
			Aura::authorities().get(i as usize).and_then(|id| {
				let raw = id.to_raw_vec();

				if raw.len() >= 24 {
					Some(H160::from_slice(&raw[4..24]))
				} else {
					None
				}
			})
		})
	}
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> (U256, Weight) {
		(U256::from(GWEI), Weight::zero())
	}
}

// TODO: Integrate to the upstream repo
pub struct FromH160;
impl<T> AddressMapping<T> for FromH160
where
	T: From<H160>,
{
	fn into_account_id(address: H160) -> T {
		address.into()
	}
}

pub struct AssetIdConverter;
impl AccountToAssetId<AccountId, AssetId> for AssetIdConverter {
	fn account_to_asset_id(account_id: AccountId) -> AssetId {
		let addr: H160 = account_id.into();
		addr.to_low_u64_be()
	}
}

pub struct CrabPrecompiles<R>(PhantomData<R>);
impl<R> CrabPrecompiles<R>
where
	R: pallet_evm::Config,
{
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self(Default::default())
	}

	pub fn used_addresses() -> [H160; 13] {
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
			addr(1026), // For KTON asset
			addr(2048),
		]
	}
}
impl<R> PrecompileSet for CrabPrecompiles<R>
where
	R: pallet_evm::Config,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		match handle.code_address() {
			// Ethereum precompiles:
			a if a == addr(1) => Some(ECRecover::execute(handle)),
			a if a == addr(2) => Some(Sha256::execute(handle)),
			a if a == addr(3) => Some(Ripemd160::execute(handle)),
			a if a == addr(4) => Some(Identity::execute(handle)),
			a if a == addr(5) => Some(Modexp::execute(handle)),
			a if a == addr(6) => Some(Bn128Add::execute(handle)),
			a if a == addr(7) => Some(Bn128Mul::execute(handle)),
			a if a == addr(8) => Some(Bn128Pairing::execute(handle)),
			a if a == addr(9) => Some(Blake2F::execute(handle)),
			// Darwinia precompiles: [1024, 2048) for stable precompiles.
			a if a == addr(1024) =>
				Some(<StateStorage<Runtime, EthereumStorageFilter>>::execute(handle)),
			a if a == addr(1025) => Some(<Dispatch<Runtime>>::execute(handle)),
			// [1026, 1536) reserved for assets precompiles.
			a if (1026..1536).contains(&AssetIdConverter::account_to_asset_id(a.into())) =>
				Some(<ERC20Assets<Runtime, AssetIdConverter>>::execute(handle)),
			// [1536, 2048) reserved for other stable precompiles.
			// [2048..) reserved for the experimental precompiles.
			a if a == addr(2048) => Some(<BLS12381<Runtime>>::execute(handle)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		Self::used_addresses().contains(&address)
	}
}

impl pallet_evm::Config for Runtime {
	type AddressMapping = FromH160;
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type ChainId = ConstU64<43>;
	type Currency = Balances;
	type FeeCalculator = FixedGasPrice;
	type FindAuthor = FindAuthorTruncated<Aura>;
	type GasWeightMapping = FixedGasWeightMapping<Self>;
	type OnChargeTransaction = ();
	type PrecompilesType = CrabPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type RuntimeEvent = RuntimeEvent;
	type WeightPerGas = WeightPerGas;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
}

fn addr(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
