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

// crates.io
use codec::{Decode, Encode, MaxEncodedLen};
// darwinia
use crate::*;
// frontier
use fp_evm::{Precompile, PrecompileSet};
// substrate
use sp_core::{ConstU32, H160, H256, U256};

pub(crate) type Balance = u128;
pub(crate) type AccountId = H160;

#[derive(Clone, Encode, Decode, Debug, MaxEncodedLen, scale_info::TypeInfo)]
pub enum Account {
	Alice,
	Bob,
	Precompile,
}
#[allow(clippy::from_over_into)]
impl Into<H160> for Account {
	fn into(self) -> H160 {
		match self {
			Account::Alice => H160::repeat_byte(0xAA),
			Account::Bob => H160::repeat_byte(0xBB),
			Account::Precompile => H160::from_low_u64_be(1),
		}
	}
}

frame_support::parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for TestRuntime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockHashCount = ();
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type Header = sp_runtime::testing::Header;
	type Index = u64;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
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

impl pallet_balances::Config for TestRuntime {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = frame_support::traits::ConstU128<0>;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_timestamp::Config for TestRuntime {
	type MinimumPeriod = ();
	type Moment = u128;
	type OnTimestampSet = ();
	type WeightInfo = ();
}

pub enum KtonAsset {}
impl darwinia_deposit::SimpleAsset for KtonAsset {
	type AccountId = AccountId;

	fn mint(_: &Self::AccountId, _: Balance) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn burn(_: &Self::AccountId, _: Balance) -> sp_runtime::DispatchResult {
		Ok(())
	}
}

impl darwinia_deposit::Config for TestRuntime {
	type Kton = KtonAsset;
	type MaxDeposits = frame_support::traits::ConstU32<16>;
	type MinLockingAmount = frame_support::traits::ConstU128<100>;
	type Ring = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

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
		[addr(1)]
	}
}
impl<R> PrecompileSet for TestPrecompiles<R>
where
	crate::Staking<R>: Precompile,
	R: pallet_evm::Config,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		match handle.code_address() {
			a if a == addr(1) => Some(crate::Staking::<R>::execute(handle)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		Self::used_addresses().contains(&address)
	}
}
fn addr(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}

pub type PCall = StakingCall<TestRuntime>;

frame_support::parameter_types! {
	pub const BlockGasLimit: U256 = U256::MAX;
	pub const WeightPerGas: frame_support::weights::Weight = frame_support::weights::Weight::from_ref_time(20_000);
	pub PrecompilesValue: TestPrecompiles<TestRuntime> = TestPrecompiles::<_>::new();
}

impl pallet_evm::Config for TestRuntime {
	type AddressMapping = pallet_evm::IdentityAddressMapping;
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type ChainId = frame_support::traits::ConstU64<42>;
	type Currency = Balances;
	type FeeCalculator = ();
	type FindAuthor = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	type PrecompilesType = TestPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type RuntimeEvent = RuntimeEvent;
	type WeightPerGas = WeightPerGas;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
}

frame_support::parameter_types! {
	pub const PayoutFraction: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(40);
}

pub enum RingStaking {}
impl darwinia_staking::Stake for RingStaking {
	type AccountId = AccountId;
	type Item = Balance;

	fn stake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		<Balances as frame_support::traits::Currency<_>>::transfer(
			who,
			&darwinia_staking::account_id(),
			item,
			frame_support::traits::ExistenceRequirement::KeepAlive,
		)
	}

	fn unstake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		<Balances as frame_support::traits::Currency<_>>::transfer(
			&darwinia_staking::account_id(),
			who,
			item,
			frame_support::traits::ExistenceRequirement::AllowDeath,
		)
	}
}

pub enum KtonStaking {}
impl darwinia_staking::Stake for KtonStaking {
	type AccountId = AccountId;
	type Item = Balance;

	fn stake(_who: &Self::AccountId, _item: Self::Item) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn unstake(_who: &Self::AccountId, _item: Self::Item) -> sp_runtime::DispatchResult {
		Ok(())
	}
}

impl darwinia_staking::Config for TestRuntime {
	type Deposit = Deposit;
	type Kton = KtonStaking;
	type MaxDeposits = frame_support::traits::ConstU32<16>;
	type MaxUnstakings = frame_support::traits::ConstU32<16>;
	type MinStakingDuration = frame_support::traits::ConstU64<3>;
	type PayoutFraction = PayoutFraction;
	type RewardRemainder = ();
	type Ring = RingStaking;
	type RingCurrency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type UnixTime = Timestamp;
}

frame_support::construct_runtime! {
	pub enum TestRuntime where
		Block = frame_system::mocking::MockBlock<TestRuntime>,
		NodeBlock = frame_system::mocking::MockBlock<TestRuntime>,
		UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>,
	{
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		Deposit: darwinia_deposit,
		EVM: pallet_evm,
		Staking: darwinia_staking,
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
