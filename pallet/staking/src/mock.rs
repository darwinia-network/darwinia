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
use core::fmt::{Display, Formatter, Result as FmtResult};
// crates.io
use codec::MaxEncodedLen;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
// darwinia
use crate::*;
use dc_types::UNIT;
// polkadot-sdk
use frame_support::{
	assert_ok, derive_impl,
	traits::{OnFinalize, OnIdle, OnInitialize},
};
use sp_core::H160;
use sp_io::TestExternalities;
use sp_runtime::{BuildStorage, RuntimeAppPublic};

pub type BlockNumber = frame_system::pallet_prelude::BlockNumberFor<Runtime>;

#[derive(
	Clone,
	Copy,
	Debug,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Serialize,
	Deserialize,
	Encode,
	Decode,
	MaxEncodedLen,
	TypeInfo,
)]
pub struct AccountId(pub u64);
impl From<H160> for AccountId {
	fn from(value: H160) -> Self {
		Self(H160::to_low_u64_le(&value))
	}
}
impl From<AccountId> for u64 {
	fn from(value: AccountId) -> Self {
		value.0
	}
}
impl From<AccountId> for H160 {
	fn from(value: AccountId) -> Self {
		H160::from_low_u64_le(value.0)
	}
}
impl Display for AccountId {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}", self.0)
	}
}
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type Block = frame_system::mocking::MockBlock<Self>;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig as pallet_timestamp::DefaultConfig)]
impl pallet_timestamp::Config for Runtime {
	type MinimumPeriod = ();
	type Moment = Moment;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type ExistentialDeposit = ();
}

frame_support::parameter_types! {
	pub static SessionHandlerCollators: Vec<AccountId> = Vec::new();
	pub static SessionChangeBlock: BlockNumber = 0;
}
sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub uint: SessionHandler,
	}
}
pub type Period = frame_support::traits::ConstU64<5>;
pub struct SessionHandler;
impl pallet_session::SessionHandler<AccountId> for SessionHandler {
	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] =
		&[sp_runtime::testing::UintAuthorityId::ID];

	fn on_genesis_session<K>(keys: &[(AccountId, K)])
	where
		K: sp_runtime::traits::OpaqueKeys,
	{
		SessionHandlerCollators::set(keys.iter().map(|(a, _)| *a).collect::<Vec<_>>())
	}

	fn on_new_session<K>(_: bool, keys: &[(AccountId, K)], _: &[(AccountId, K)])
	where
		K: sp_runtime::traits::OpaqueKeys,
	{
		SessionChangeBlock::set(System::block_number());
		SessionHandlerCollators::set(keys.iter().map(|(a, _)| *a).collect::<Vec<_>>())
	}

	fn on_before_session_ending() {}

	fn on_disabled(_: u32) {}
}
impl sp_runtime::BoundToRuntimeAppPublic for SessionHandler {
	type Public = sp_runtime::testing::UintAuthorityId;
}
impl pallet_session::Config for Runtime {
	type Keys = SessionKeys;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, ()>;
	type RuntimeEvent = RuntimeEvent;
	type SessionHandler = SessionHandler;
	type SessionManager = Staking;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, ()>;
	type ValidatorId = Self::AccountId;
	type ValidatorIdOf = crate::IdentityCollator;
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub const TreasuryPalletId: frame_support::PalletId = frame_support::PalletId(*b"da/trsry");
	pub TreasuryAccount: AccountId = Treasury::account_id();
}
#[cfg(feature = "runtime-benchmarks")]
pub struct DummyBenchmarkHelper;
#[cfg(feature = "runtime-benchmarks")]
impl<AssetKind> pallet_treasury::ArgumentsFactory<AssetKind, AccountId> for DummyBenchmarkHelper
where
	AssetKind: Default,
{
	fn create_asset_kind(_: u32) -> AssetKind {
		Default::default()
	}

	fn create_beneficiary(_: [u8; 32]) -> AccountId {
		AccountId(0)
	}
}
impl pallet_treasury::Config for Runtime {
	type ApproveOrigin = frame_system::EnsureRoot<AccountId>;
	type AssetKind = ();
	type BalanceConverter = frame_support::traits::tokens::UnityAssetBalanceConversion;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = DummyBenchmarkHelper;
	type Beneficiary = AccountId;
	type BeneficiaryLookup = Self::Lookup;
	type Burn = ();
	type BurnDestination = ();
	type Currency = Balances;
	type MaxApprovals = ();
	type OnSlash = ();
	type PalletId = TreasuryPalletId;
	type Paymaster = frame_support::traits::tokens::PayFromAccount<Balances, TreasuryAccount>;
	type PayoutPeriod = ();
	type RejectOrigin = frame_system::EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SpendFunds = ();
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;
	type SpendPeriod = frame_support::traits::ConstU64<999>;
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub static InflationType: u8 = 0;
	pub static NextCollatorId: u64 = 1;
}
pub enum StatedOnSessionEnd {}
impl crate::IssuingManager<Runtime> for StatedOnSessionEnd {
	fn inflate() -> Balance {
		if INFLATION_TYPE.with(|v| *v.borrow()) == 0 {
			<crate::BalancesIssuing<Runtime>>::inflate()
		} else {
			<crate::TreasuryIssuing<Runtime, frame_support::traits::ConstU128<{ 20_000 * UNIT }>>>::inflate()
		}
	}

	fn calculate_reward(issued: Balance) -> Balance {
		if INFLATION_TYPE.with(|v| *v.borrow()) == 0 {
			<crate::BalancesIssuing<Runtime>>::calculate_reward(issued)
		} else {
			<crate::TreasuryIssuing<Runtime, frame_support::traits::ConstU128<{ 20_000 * UNIT }>>>::calculate_reward(issued)
		}
	}

	fn reward(amount: Balance) -> sp_runtime::DispatchResult {
		if INFLATION_TYPE.with(|v| *v.borrow()) == 0 {
			<crate::BalancesIssuing<Runtime>>::reward(amount)
		} else {
			<crate::TreasuryIssuing<Runtime,frame_support::traits::ConstU128<{20_000 * UNIT}>>>::reward( amount)
		}
	}
}
pub enum RingStaking {}
impl crate::Election<AccountId> for RingStaking {
	fn elect(x: u32) -> Option<Vec<AccountId>> {
		let start = NextCollatorId::get();
		let end = start + x as u64;

		assert!(end < 1_000);

		Some(
			(start..end)
				.map(|i| {
					let who = AccountId(i);

					assert_ok!(Session::set_keys(
						RuntimeOrigin::signed(who),
						SessionKeys { uint: i.into() },
						Vec::new(),
					));

					who
				})
				.collect(),
		)
	}
}
impl crate::Reward<AccountId> for RingStaking {
	fn allocate(who: Option<AccountId>, amount: Balance) {
		let Some(who) = who else { return };
		let _ = Balances::transfer_keep_alive(
			RuntimeOrigin::signed(Treasury::account_id()),
			who,
			amount,
		);
	}
}
pub enum KtonStaking {}
impl crate::Reward<AccountId> for KtonStaking {
	fn allocate(_: Option<AccountId>, amount: Balance) {
		let _ = Balances::transfer_keep_alive(
			RuntimeOrigin::signed(TreasuryAccount::get()),
			<KtonStakingContract<Runtime>>::get().unwrap(),
			amount,
		);
	}
}
impl crate::Config for Runtime {
	type Currency = Balances;
	type IssuingManager = StatedOnSessionEnd;
	type KtonStaking = KtonStaking;
	type RingStaking = RingStaking;
	type RuntimeEvent = RuntimeEvent;
	type Treasury = TreasuryAccount;
	type UnixTime = Timestamp;
	type WeightInfo = ();
}

frame_support::construct_runtime! {
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Session: pallet_session,
		Treasury: pallet_treasury,
		Staking: crate,
	}
}

pub enum Efflux {}
impl Efflux {
	pub fn time(millis: Moment) {
		Timestamp::set_timestamp(Timestamp::now() + millis);
	}

	pub fn block(number: BlockNumber) {
		let now = System::block_number();

		(now..now + number).for_each(|n| {
			initialize_block(n + 1);
			finalize_block(n + 1);
		});
	}
}

pub struct ExtBuilder;
impl ExtBuilder {
	pub fn inflation_type(self, r#type: u8) -> Self {
		INFLATION_TYPE.with(|v| *v.borrow_mut() = r#type);

		self
	}

	pub fn build(self) -> TestExternalities {
		let _ = pretty_env_logger::try_init();
		let mut storage =
			<frame_system::GenesisConfig<Runtime>>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: (1..=1_000)
				.map(|i| (AccountId(i), 100))
				.chain([(Treasury::account_id(), 1 << 126), (account_id(), 1 << 126)])
				.collect(),
		}
		.assimilate_storage(&mut storage)
		.unwrap();
		pallet_session::GenesisConfig::<Runtime> {
			keys: (1..=3)
				.map(|i| (AccountId(i), AccountId(i), SessionKeys { uint: i.into() }))
				.collect(),
		}
		.assimilate_storage(&mut storage)
		.unwrap();
		crate::GenesisConfig::<Runtime> { collator_count: 3, ..Default::default() }
			.assimilate_storage(&mut storage)
			.unwrap();

		let mut ext = TestExternalities::from(storage);

		ext.execute_with(|| {
			<RingStakingContract<Runtime>>::put(AccountId(1_001));
			<KtonStakingContract<Runtime>>::put(AccountId(1_002));

			preset_session_keys();
			new_session();
		});

		ext
	}
}

pub fn preset_session_keys() {
	(1..1_000).for_each(|i| {
		assert_ok!(Session::set_keys(
			RuntimeOrigin::signed(AccountId(i)),
			SessionKeys { uint: i.into() },
			Vec::new()
		));
	});
}

pub fn initialize_block(number: BlockNumber) {
	System::set_block_number(number);
	Efflux::time(1);
	AllPalletsWithSystem::on_initialize(number);
}

pub fn finalize_block(number: BlockNumber) {
	AllPalletsWithSystem::on_idle(number, Weight::MAX);
	AllPalletsWithSystem::on_finalize(number);
}

pub fn new_session() {
	let now = System::block_number();
	let target = now + <Period as sp_runtime::traits::Get<BlockNumber>>::get();
	let collators = Session::validators();

	(now..target).zip(collators.into_iter().cycle()).for_each(|(_, who)| {
		Staking::note_authors(&[who]);
		Efflux::block(1);
	});
}

pub fn events() -> Vec<Event<Runtime>> {
	System::read_events_for_pallet()
}
