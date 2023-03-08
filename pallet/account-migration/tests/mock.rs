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

pub use dc_primitives::*;

// substrate
use sp_io::TestExternalities;

pub struct Dummy;
impl darwinia_deposit::SimpleAsset for Dummy {
	type AccountId = AccountId;

	fn mint(_: &Self::AccountId, _: Balance) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn burn(_: &Self::AccountId, _: Balance) -> sp_runtime::DispatchResult {
		Ok(())
	}
}
impl darwinia_staking::Stake for Dummy {
	type AccountId = AccountId;
	type Item = Balance;

	fn stake(_: &Self::AccountId, _: Self::Item) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn unstake(_: &Self::AccountId, _: Self::Item) -> sp_runtime::DispatchResult {
		Ok(())
	}
}
impl frame_support::traits::UnixTime for Dummy {
	fn now() -> core::time::Duration {
		core::time::Duration::new(0, 0)
	}
}

#[sp_version::runtime_version]
pub const VERSION: sp_version::RuntimeVersion = sp_version::RuntimeVersion {
	spec_name: sp_runtime::create_runtime_str!("Darwinia2"),
	impl_name: sp_runtime::create_runtime_str!("DarwiniaOfficialRust"),
	authoring_version: 0,
	spec_version: 6_0_0_0,
	impl_version: 0,
	apis: sp_version::create_apis_vec!([]),
	transaction_version: 0,
	state_version: 0,
};
frame_support::parameter_types! {
	pub const Version: sp_version::RuntimeVersion = VERSION;
}
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockHashCount = ();
	type BlockLength = ();
	type BlockNumber = BlockNumber;
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type Header = sp_runtime::generic::Header<Self::BlockNumber, Self::Hashing>;
	type Index = Index;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = Version;
}

impl pallet_timestamp::Config for Runtime {
	type MinimumPeriod = ();
	type Moment = Moment;
	type OnTimestampSet = ();
	type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_assets::Config for Runtime {
	type ApprovalDeposit = ();
	type AssetAccountDeposit = ();
	type AssetDeposit = ();
	type AssetId = AssetId;
	type AssetIdParameter = codec::Compact<AssetId>;
	type Balance = Balance;
	type CallbackHandle = ();
	type CreateOrigin =
		frame_support::traits::AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = frame_system::EnsureSigned<AccountId>;
	type Freezer = ();
	type MetadataDepositBase = ();
	type MetadataDepositPerByte = ();
	type RemoveItemsLimit = ();
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = ();
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub UnvestedFundsAllowedWithdrawReasons: frame_support::traits::WithdrawReasons =
		frame_support::traits::WithdrawReasons::except(
			frame_support::traits::WithdrawReasons::TRANSFER | frame_support::traits::WithdrawReasons::RESERVE
		);
}
impl pallet_vesting::Config for Runtime {
	type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
	type Currency = Balances;
	type MinVestedTransfer = ();
	type RuntimeEvent = RuntimeEvent;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	type WeightInfo = ();

	const MAX_VESTING_SCHEDULES: u32 = 1;
}

impl darwinia_deposit::Config for Runtime {
	type Kton = Dummy;
	type MaxDeposits = ();
	type MinLockingAmount = ();
	type Ring = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl darwinia_staking::Config for Runtime {
	type Deposit = Deposit;
	type Kton = Dummy;
	type MaxDeposits = ();
	type MaxUnstakings = ();
	type MinStakingDuration = ();
	type PayoutFraction = ();
	type RewardRemainder = ();
	type Ring = Dummy;
	type RingCurrency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type UnixTime = Dummy;
}

impl pallet_identity::Config for Runtime {
	type BasicDeposit = ();
	type Currency = Balances;
	type FieldDeposit = ();
	type ForceOrigin = frame_system::EnsureSigned<AccountId>;
	type MaxAdditionalFields = ();
	type MaxRegistrars = ();
	type MaxSubAccounts = ();
	type RegistrarOrigin = frame_system::EnsureSigned<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type Slashed = ();
	type SubAccountDeposit = ();
	type WeightInfo = ();
}

impl darwinia_account_migration::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

frame_support::construct_runtime! {
	pub enum Runtime where
		Block = frame_system::mocking::MockBlock<Runtime>,
		NodeBlock = frame_system::mocking::MockBlock<Runtime>,
		UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>,
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Vesting: pallet_vesting,
		Deposit: darwinia_deposit,
		Staking: darwinia_staking,
		Identity: pallet_identity,
		AccountMigration: darwinia_account_migration,
	}
}

pub fn new_test_ext() -> TestExternalities {
	frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into()
}
