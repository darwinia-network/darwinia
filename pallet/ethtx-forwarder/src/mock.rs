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

//! Test utilities

pub use crate as darwinia_ethtx_forwarder;

// crates.io
use sha3::Digest;
// polkadot-sdk
use frame_support::{derive_impl, traits::ConstU64};
use sp_runtime::BuildStorage;
use sp_std::prelude::*;

pub type AccountId = sp_core::H160;
pub type Balance = u64;

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
	type ExistentialDeposit = ConstU64<0>;
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig as pallet_timestamp::DefaultConfig)]
impl pallet_timestamp::Config for Runtime {}

frame_support::parameter_types! {
	pub const BlockGasLimit: sp_core::U256 = sp_core::U256::MAX;
	pub const WeightPerGas: frame_support::weights::Weight = frame_support::weights::Weight::from_parts(20_000, 0);
}
pub struct FixedGasPrice;
impl fp_evm::FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> (sp_core::U256, frame_support::weights::Weight) {
		(sp_core::U256::from(5), frame_support::weights::Weight::zero())
	}
}
impl pallet_evm::Config for Runtime {
	type AddressMapping = pallet_evm::IdentityAddressMapping;
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type ChainId = frame_support::traits::ConstU64<888>;
	type Currency = Balances;
	type FeeCalculator = FixedGasPrice;
	type FindAuthor = ();
	type GasLimitPovSizeRatio = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type RuntimeEvent = RuntimeEvent;
	type SuicideQuickClearLimit = ();
	type Timestamp = Timestamp;
	type WeightInfo = ();
	type WeightPerGas = WeightPerGas;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
}

frame_support::parameter_types! {
	pub const PostBlockAndTxnHashes: pallet_ethereum::PostLogContent = pallet_ethereum::PostLogContent::BlockAndTxnHashes;
}
impl pallet_ethereum::Config for Runtime {
	type ExtraDataLength = ();
	type PostLogContent = PostBlockAndTxnHashes;
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}

impl darwinia_ethtx_forwarder::Config for Runtime {
	type ForwardEthOrigin = crate::EnsureRuntimeEthOrigin;
	type ValidatedTransaction = pallet_ethereum::ValidatedTransaction<Self>;
}

frame_support::construct_runtime! {
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		EVM: pallet_evm,
		Ethereum: pallet_ethereum,
		EthTxForwarder: darwinia_ethtx_forwarder,
	}
}

impl fp_self_contained::SelfContainedCall for RuntimeCall {
	type SignedInfo = sp_core::H160;

	fn is_self_contained(&self) -> bool {
		match self {
			RuntimeCall::Ethereum(call) => call.is_self_contained(),
			_ => false,
		}
	}

	fn check_self_contained(
		&self,
	) -> Option<Result<Self::SignedInfo, sp_runtime::transaction_validity::TransactionValidityError>>
	{
		match self {
			RuntimeCall::Ethereum(call) => call.check_self_contained(),
			_ => None,
		}
	}

	fn validate_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<sp_runtime::transaction_validity::TransactionValidity> {
		match self {
			RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn pre_dispatch_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<Result<(), sp_runtime::transaction_validity::TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) =>
				call.pre_dispatch_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn apply_self_contained(
		self,
		info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<sp_runtime::traits::PostDispatchInfoOf<Self>>> {
		use sp_runtime::traits::Dispatchable as _;
		match self {
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) =>
				Some(call.dispatch(RuntimeOrigin::from(
					pallet_ethereum::RawOrigin::EthereumTransaction(info),
				))),
			_ => None,
		}
	}
}

pub struct AccountInfo {
	pub address: sp_core::H160,
}
pub fn address_build(seed: u8) -> AccountInfo {
	let raw_private_key = [seed + 1; 32];
	let secret_key = libsecp256k1::SecretKey::parse_slice(&raw_private_key).unwrap();
	let raw_public_key = &libsecp256k1::PublicKey::from_secret_key(&secret_key).serialize()[1..65];
	let raw_address = {
		let mut s = [0; 20];
		s.copy_from_slice(&sha3::Keccak256::digest(raw_public_key)[12..]);
		s
	};

	AccountInfo { address: raw_address.into() }
}

#[derive(Default)]
pub struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}
impl ExtBuilder {
	pub fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
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
