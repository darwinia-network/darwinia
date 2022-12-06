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

//! Test utilities

// crates.io
use codec::{Decode, Encode};
use sha3::{Digest, Keccak256};
// frontier
use pallet_ethereum::IntermediateStateRoot;
use pallet_evm::IdentityAddressMapping;
// substrate
use frame_support::{
	dispatch::RawOrigin,
	ensure,
	pallet_prelude::Weight,
	traits::{ConstU32, Everything},
};
use sp_core::{H256, U256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, DispatchInfoOf, IdentifyAccount, IdentityLookup, Verify},
	transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError},
};
use sp_std::prelude::*;
// darwinia
use crate::*;
use bp_message_dispatch::{CallValidate, IntoDispatchOrigin as IntoDispatchOriginT};

pub type Block = frame_system::mocking::MockBlock<TestRuntime>;
pub type Balance = u64;
pub type AccountId = H160;
pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;

frame_support::parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for TestRuntime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type BlockHashCount = ();
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

frame_support::parameter_types! {
	pub const MaxLocks: u32 = 10;
	pub const ExistentialDeposit: u64 = 0;
}
impl pallet_balances::Config for TestRuntime {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub const MinimumPeriod: u64 = 6000 / 2;
}
impl pallet_timestamp::Config for TestRuntime {
	type MinimumPeriod = MinimumPeriod;
	type Moment = u64;
	type OnTimestampSet = ();
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub const TransactionByteFee: u64 = 1;
	pub const ChainId: u64 = 42;
	pub const BlockGasLimit: U256 = U256::MAX;
	pub const WeightPerGas: Weight = Weight::from_ref_time(20_000);
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> (U256, Weight) {
		(U256::from(5), Weight::zero())
	}
}

impl pallet_evm::Config for TestRuntime {
	type AddressMapping = IdentityAddressMapping;
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type ChainId = ChainId;
	type Currency = Balances;
	type FeeCalculator = FixedGasPrice;
	type FindAuthor = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type OnChargeTransaction = ();
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type RuntimeEvent = RuntimeEvent;
	type WeightPerGas = WeightPerGas;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
}

impl pallet_ethereum::Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = IntermediateStateRoot<Self>;
}

pub struct MockAccountIdConverter;
impl sp_runtime::traits::Convert<H256, AccountId> for MockAccountIdConverter {
	fn convert(hash: H256) -> AccountId {
		hash.into()
	}
}

#[derive(Decode, Encode, Clone)]
pub struct MockEncodedCall(pub Vec<u8>);
impl From<MockEncodedCall> for Result<RuntimeCall, ()> {
	fn from(call: MockEncodedCall) -> Result<RuntimeCall, ()> {
		RuntimeCall::decode(&mut &call.0[..]).map_err(drop)
	}
}

pub struct MockCallValidator;
impl CallValidate<AccountId, RuntimeOrigin, RuntimeCall> for MockCallValidator {
	fn check_receiving_before_dispatch(
		relayer_account: &AccountId,
		call: &RuntimeCall,
	) -> Result<(), &'static str> {
		match call {
			RuntimeCall::MessageTransact(crate::Call::message_transact { transaction: tx }) => {
				let total_payment = crate::total_payment::<TestRuntime>((&**tx).into());
				let relayer = pallet_evm::Pallet::<TestRuntime>::account_basic(&relayer_account).0;

				ensure!(relayer.balance >= total_payment, "Insufficient balance");
				Ok(())
			},
			_ => Ok(()),
		}
	}

	fn call_validate(
		relayer_account: &AccountId,
		origin: &RuntimeOrigin,
		call: &RuntimeCall,
	) -> Result<(), TransactionValidityError> {
		match call {
			RuntimeCall::MessageTransact(crate::Call::message_transact { transaction: tx }) =>
				match origin.caller {
					OriginCaller::MessageTransact(LcmpEthOrigin::MessageTransact(id)) => {
						let total_payment = crate::total_payment::<TestRuntime>((&**tx).into());
						pallet_balances::Pallet::<TestRuntime>::transfer(
							RawOrigin::Signed(*relayer_account).into(),
							id,
							total_payment.as_u64(),
						)
						.map_err(|_| {
							TransactionValidityError::Invalid(InvalidTransaction::Payment)
						})?;

						Ok(())
					},
					_ => Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner)),
				},
			_ => Ok(()),
		}
	}
}
pub struct MockIntoDispatchOrigin;
impl IntoDispatchOriginT<AccountId, RuntimeCall, RuntimeOrigin> for MockIntoDispatchOrigin {
	fn into_dispatch_origin(id: &AccountId, call: &RuntimeCall) -> RuntimeOrigin {
		match call {
			RuntimeCall::MessageTransact(crate::Call::message_transact { .. }) =>
				crate::LcmpEthOrigin::MessageTransact(*id).into(),
			_ => frame_system::RawOrigin::Signed(*id).into(),
		}
	}
}
#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct MockAccountPublic(AccountId);
impl IdentifyAccount for MockAccountPublic {
	type AccountId = AccountId;

	fn into_account(self) -> AccountId {
		self.0
	}
}
#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct MockSignature(AccountId);
impl Verify for MockSignature {
	type Signer = MockAccountPublic;

	fn verify<L: sp_runtime::traits::Lazy<[u8]>>(&self, _msg: L, signer: &AccountId) -> bool {
		self.0 == *signer
	}
}

pub(crate) type MockBridgeMessageId = [u8; 4];

impl pallet_bridge_dispatch::Config for TestRuntime {
	type AccountIdConverter = MockAccountIdConverter;
	type BridgeMessageId = MockBridgeMessageId;
	type CallValidator = MockCallValidator;
	type EncodedCall = MockEncodedCall;
	type IntoDispatchOrigin = MockIntoDispatchOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type SourceChainAccountId = AccountId;
	type TargetChainAccountPublic = MockAccountPublic;
	type TargetChainSignature = MockSignature;
}

impl crate::Config for TestRuntime {
	type LcmpEthOrigin = crate::EnsureLcmpEthOrigin;
	type ValidatedTransaction = pallet_ethereum::ValidatedTransaction<Self>;
}

frame_support::construct_runtime! {
	pub enum TestRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		EVM: pallet_evm::{Pallet, Call, Storage, Config, Event<T>},
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Origin},
		MessageTransact: crate::{Pallet, Call, Origin},
		Dispatch: pallet_bridge_dispatch::{Pallet, Call, Event<T>},
	}
}

impl fp_self_contained::SelfContainedCall for RuntimeCall {
	type SignedInfo = H160;

	fn is_self_contained(&self) -> bool {
		match self {
			RuntimeCall::Ethereum(call) => call.is_self_contained(),
			_ => false,
		}
	}

	fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => call.check_self_contained(),
			_ => None,
		}
	}

	fn validate_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<TransactionValidity> {
		match self {
			RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn pre_dispatch_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<Result<(), TransactionValidityError>> {
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

pub(crate) struct AccountInfo {
	pub address: H160,
	pub private_key: H256,
}

pub(crate) fn address_build(seed: u8) -> AccountInfo {
	let raw_private_key = [seed + 1; 32];
	let secret_key = libsecp256k1::SecretKey::parse_slice(&raw_private_key).unwrap();
	let raw_public_key = &libsecp256k1::PublicKey::from_secret_key(&secret_key).serialize()[1..65];
	let raw_address = {
		let mut s = [0; 20];
		s.copy_from_slice(&Keccak256::digest(raw_public_key)[12..]);
		s
	};

	AccountInfo { private_key: raw_private_key.into(), address: raw_address.into() }
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
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<TestRuntime> { balances: self.balances }
			.assimilate_storage(&mut t)
			.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
