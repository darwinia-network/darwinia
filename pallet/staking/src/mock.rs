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
use frame_support::{assert_ok, derive_impl, traits::OnInitialize};
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
#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
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

pub enum KtonMinting {}
impl darwinia_deposit::SimpleAsset for KtonMinting {
	type AccountId = AccountId;

	fn mint(_: &Self::AccountId, _: Balance) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn burn(_: &Self::AccountId, _: Balance) -> sp_runtime::DispatchResult {
		Ok(())
	}
}
impl darwinia_deposit::Config for Runtime {
	type DepositMigrator = ();
	type Kton = KtonMinting;
	type MaxDeposits = frame_support::traits::ConstU32<16>;
	type Ring = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Treasury = TreasuryAcct;
	type WeightInfo = ();
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
pub type Period = frame_support::traits::ConstU64<3>;
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
	pub TreasuryAcct: AccountId = Treasury::account_id();
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
	type Paymaster = frame_support::traits::tokens::PayFromAccount<Balances, TreasuryAcct>;
	type PayoutPeriod = ();
	type ProposalBond = ();
	type ProposalBondMaximum = ();
	type ProposalBondMinimum = ();
	type RejectOrigin = frame_system::EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SpendFunds = ();
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;
	type SpendPeriod = frame_support::traits::ConstU64<999>;
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub static InflationType: u8 = 0;
}
pub enum StatedOnSessionEnd {}
impl crate::IssuingManager<Runtime> for StatedOnSessionEnd {
	fn inflate() -> Balance {
		if INFLATION_TYPE.with(|v| *v.borrow()) == 0 {
			<crate::BalanceIssuing<Runtime>>::inflate()
		} else {
			<crate::TreasuryIssuing<Runtime, frame_support::traits::ConstU128<{ 20_000 * UNIT }>>>::inflate()
		}
	}

	fn calculate_reward(issued: Balance) -> Balance {
		if INFLATION_TYPE.with(|v| *v.borrow()) == 0 {
			<crate::BalanceIssuing<Runtime>>::calculate_reward(issued)
		} else {
			<crate::TreasuryIssuing<Runtime, frame_support::traits::ConstU128<{ 20_000 * UNIT }>>>::calculate_reward(issued)
		}
	}

	fn reward(who: &AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		if INFLATION_TYPE.with(|v| *v.borrow()) == 0 {
			<crate::BalanceIssuing<Runtime>>::reward(who, amount)
		} else {
			<crate::TreasuryIssuing<Runtime,frame_support::traits::ConstU128<{20_000 * UNIT}>>>::reward(who, amount)
		}
	}
}
pub enum RingStaking {}
impl crate::Stake for RingStaking {
	type AccountId = AccountId;
	type Item = Balance;

	fn stake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		<Balances as frame_support::traits::Currency<_>>::transfer(
			who,
			&crate::account_id(),
			item,
			frame_support::traits::ExistenceRequirement::AllowDeath,
		)
	}

	fn unstake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		<Balances as frame_support::traits::Currency<_>>::transfer(
			&crate::account_id(),
			who,
			item,
			frame_support::traits::ExistenceRequirement::AllowDeath,
		)
	}
}
impl crate::Election<AccountId> for RingStaking {
	fn elect(n: u32) -> Option<Vec<AccountId>> {
		Some(
			(100..(100 + n) as u64)
				.map(|i| {
					let who = AccountId(i);

					assert_ok!(Session::set_keys(
						RuntimeOrigin::signed(who),
						SessionKeys { uint: i.into() },
						Vec::new()
					));

					who
				})
				.collect(),
		)
	}
}
impl crate::Reward<AccountId> for RingStaking {
	fn distribute(who: Option<AccountId>, amount: Balance) {
		let Some(who) = who else { return };
		let _ =
			Balances::transfer_keep_alive(RuntimeOrigin::signed(TreasuryAcct::get()), who, amount);
	}
}
pub enum KtonStaking {}
impl crate::Reward<AccountId> for KtonStaking {
	fn distribute(_: Option<AccountId>, amount: Balance) {
		let _ = Balances::transfer_keep_alive(
			RuntimeOrigin::signed(TreasuryAcct::get()),
			<KtonStakingContract<Runtime>>::get().unwrap(),
			amount,
		);
	}
}
impl crate::Config for Runtime {
	type Currency = Balances;
	type Deposit = Deposit;
	type IssuingManager = StatedOnSessionEnd;
	type KtonStaking = KtonStaking;
	type MaxDeposits = <Self as darwinia_deposit::Config>::MaxDeposits;
	type Ring = RingStaking;
	type RingStaking = RingStaking;
	type RuntimeEvent = RuntimeEvent;
	type ShouldEndSession = crate::ShouldEndSession<Self>;
	type Treasury = TreasuryAcct;
	type UnixTime = Timestamp;
	type WeightInfo = ();
}
#[cfg(not(feature = "runtime-benchmarks"))]
impl crate::DepositConfig for Runtime {}

frame_support::construct_runtime! {
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Deposit: darwinia_deposit,
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

pub struct ExtBuilder {
	collator_count: u32,
	genesis_collator: bool,
}
impl ExtBuilder {
	pub fn inflation_type(self, r#type: u8) -> Self {
		INFLATION_TYPE.with(|v| *v.borrow_mut() = r#type);

		self
	}

	pub fn collator_count(mut self, collator_count: u32) -> Self {
		self.collator_count = collator_count;

		self
	}

	pub fn genesis_collator(mut self) -> Self {
		self.genesis_collator = true;

		self
	}

	pub fn build(self) -> TestExternalities {
		// let _ = pretty_env_logger::try_init();
		let mut storage =
			<frame_system::GenesisConfig<Runtime>>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: (1..=10)
				.map(|i| (AccountId(i), 1_000 * UNIT))
				.chain([(TreasuryAcct::get(), 1_000_000 * UNIT)])
				.collect(),
		}
		.assimilate_storage(&mut storage)
		.unwrap();
		crate::GenesisConfig::<Runtime> {
			rate_limit: 100 * UNIT,
			collator_count: self.collator_count,
			collators: if self.genesis_collator {
				(1..=self.collator_count as u64).map(|i| (AccountId(i), i as _)).collect()
			} else {
				Default::default()
			},
			..Default::default()
		}
		.assimilate_storage(&mut storage)
		.unwrap();
		if self.genesis_collator {
			pallet_session::GenesisConfig::<Runtime> {
				keys: (1..=self.collator_count as u64)
					.map(|i| (AccountId(i), AccountId(i), SessionKeys { uint: i.into() }))
					.collect(),
			}
			.assimilate_storage(&mut storage)
			.unwrap();
		}

		let mut ext = TestExternalities::from(storage);

		ext.execute_with(|| {
			<RingStakingContract<Runtime>>::put(AccountId(718));
			<KtonStakingContract<Runtime>>::put(AccountId(719));

			new_session();
		});

		ext
	}
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self { collator_count: 1, genesis_collator: false }
	}
}

pub fn preset_collator_wait_list(n: u64) {
	(10..10 + n).for_each(|i| {
		let who = AccountId(i);
		let _ = <pallet_balances::Pallet<Runtime>>::deposit_creating(&who, i as _);

		assert_ok!(Session::set_keys(
			RuntimeOrigin::signed(who),
			SessionKeys { uint: i.into() },
			Vec::new()
		));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(who), i as _, Vec::new()));
		assert_ok!(Staking::collect(RuntimeOrigin::signed(who), Perbill::zero()));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who), who));
	});
	(100..100 + n).for_each(|i| {
		let who = AccountId(i);
		let _ = <pallet_balances::Pallet<Runtime>>::deposit_creating(&who, i as _);
	});
}

pub fn initialize_block(number: BlockNumber) {
	System::set_block_number(number);
	Efflux::time(1);
	<AllPalletsWithSystem as OnInitialize<BlockNumber>>::on_initialize(number);
}

pub fn finalize_block(number: BlockNumber) {
	<AllPalletsWithSystem as frame_support::traits::OnFinalize<BlockNumber>>::on_finalize(number);
}

pub fn new_session() {
	let now = System::block_number();
	let target = now + <Period as sp_runtime::traits::Get<BlockNumber>>::get();

	(now..target).for_each(|_| Efflux::block(1));
}

pub fn payout() {
	crate::call_on_cache_v2!(<Previous<Runtime>>::get().into_iter().for_each(|c| {
		let _ = Staking::payout_inner(c);
	}))
	.unwrap();
	crate::call_on_cache_v1!(<Previous<Runtime>>::iter_keys().for_each(|c| {
		let _ = Staking::payout_inner(c);
	}))
	.unwrap();
}
