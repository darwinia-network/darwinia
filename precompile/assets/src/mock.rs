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

// core
use core::marker::PhantomData;
// crates.io
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
// frontier
use precompile_utils::Precompile;
// darwinia
use crate::*;
// polkadot-sdk
use frame_support::derive_impl;
use sp_core::H160;
use sp_runtime::BuildStorage;
use sp_std::prelude::*;

pub type Balance = u128;
pub type AssetId = u64;
pub type InternalCall = ERC20AssetsCall<Runtime, AssetIdConverter>;
pub type AccountId = H160;

pub const TEST_ID: AssetId = 1026;

#[derive(Clone, Encode, Decode, Debug, MaxEncodedLen, TypeInfo)]
pub enum Account {
	Alice,
	Bob,
	Charlie,
	Precompile,
}
#[allow(clippy::from_over_into)]
impl Into<H160> for Account {
	fn into(self) -> H160 {
		match self {
			Account::Alice => H160::repeat_byte(0xAA),
			Account::Bob => H160::repeat_byte(0xBB),
			Account::Charlie => H160::repeat_byte(0xCC),
			Account::Precompile => addr(TEST_ID),
		}
	}
}
impl From<Account> for sp_core::H256 {
	fn from(x: Account) -> sp_core::H256 {
		let x: H160 = x.into();
		x.into()
	}
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type Block = frame_system::mocking::MockBlock<Self>;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type ExistentialDeposit = ();
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig as pallet_timestamp::DefaultConfig)]
impl pallet_timestamp::Config for Runtime {}

pub struct TestPrecompiles<R>(PhantomData<R>);
impl<R> TestPrecompiles<R>
where
	R: pallet_evm::Config,
{
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self(Default::default())
	}

	pub fn used_addresses() -> [H160; 1] {
		[addr(TEST_ID)]
	}
}

impl<R> fp_evm::PrecompileSet for TestPrecompiles<R>
where
	R: pallet_evm::Config,
	ERC20Assets<R, AssetIdConverter>: fp_evm::Precompile,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		match handle.code_address() {
			a if a == addr(TEST_ID) => Some(<ERC20Assets<R, AssetIdConverter>>::execute(handle)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160, _gas: u64) -> fp_evm::IsPrecompileResult {
		fp_evm::IsPrecompileResult::Answer {
			is_precompile: Self::used_addresses().contains(&address),
			extra_cost: 0,
		}
	}
}
fn addr(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}

pub struct AssetIdConverter;
impl AccountToAssetId<AccountId, AssetId> for AssetIdConverter {
	fn account_to_asset_id(account_id: AccountId) -> AssetId {
		account_id.to_low_u64_be()
	}
}

frame_support::parameter_types! {
	pub const BlockGasLimit: U256 = U256::MAX;
	pub const WeightPerGas: frame_support::weights::Weight = frame_support::weights::Weight::from_parts(20_000, 0);
	pub PrecompilesValue: TestPrecompiles<Runtime> = TestPrecompiles::<_>::new();
}

impl pallet_evm::Config for Runtime {
	type AddressMapping = pallet_evm::IdentityAddressMapping;
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type ChainId = frame_support::traits::ConstU64<42>;
	type Currency = Balances;
	type FeeCalculator = ();
	type FindAuthor = ();
	type GasLimitPovSizeRatio = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	type PrecompilesType = TestPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type RuntimeEvent = RuntimeEvent;
	type Timestamp = Timestamp;
	type WeightInfo = ();
	type SuicideQuickClearLimit = ();
	type WeightPerGas = WeightPerGas;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
}

#[cfg(feature = "runtime-benchmarks")]
pub enum BenchmarkHelper {}
#[cfg(feature = "runtime-benchmarks")]
impl pallet_assets::BenchmarkHelper<codec::Compact<AssetId>> for BenchmarkHelper {
	fn create_asset_id_parameter(id: u32) -> codec::Compact<AssetId> {
		(id as u64).into()
	}
}
impl pallet_assets::Config for Runtime {
	type ApprovalDeposit = ();
	type AssetAccountDeposit = ();
	type AssetDeposit = ();
	type AssetId = AssetId;
	type AssetIdParameter = codec::Compact<AssetId>;
	type Balance = Balance;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = BenchmarkHelper;
	type CallbackHandle = ();
	type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<
		frame_system::EnsureSignedBy<frame_support::traits::IsInVec<()>, AccountId>,
	>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Freezer = ();
	type MetadataDepositBase = ();
	type MetadataDepositPerByte = ();
	type RemoveItemsLimit = ();
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = frame_support::traits::ConstU32<50>;
	type WeightInfo = ();
}

frame_support::construct_runtime! {
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		EVM: pallet_evm,
		Assets: pallet_assets
	}
}

#[derive(Default)]
pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = <frame_system::GenesisConfig<Runtime>>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
			.assimilate_storage(&mut t)
			.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
