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

pub use crate as darwinia_staking;

// darwinia
use dc_types::{Balance, Moment, UNIT};
// polkadot-sdk
use frame_support::{
	derive_impl,
	traits::{Currency, OnInitialize},
};
use sp_io::TestExternalities;
use sp_runtime::{BuildStorage, RuntimeAppPublic};

pub type AccountId = u32;
pub type BlockNumber = frame_system::pallet_prelude::BlockNumberFor<Runtime>;

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
	type Kton = KtonMinting;
	type MaxDeposits = frame_support::traits::ConstU32<16>;
	type MinLockingAmount = frame_support::traits::ConstU128<UNIT>;
	type Ring = Balances;
	type RuntimeEvent = RuntimeEvent;
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

	fn on_disabled(_: AccountId) {}
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
	type ValidatorIdOf = darwinia_staking::IdentityCollator;
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub const TreasuryPalletId: frame_support::PalletId = frame_support::PalletId(*b"da/trsry");
	pub TreasuryAccount: AccountId = Treasury::account_id();
}
impl pallet_treasury::Config for Runtime {
	type ApproveOrigin = frame_system::EnsureRoot<AccountId>;
	type AssetKind = ();
	type BalanceConverter = frame_support::traits::tokens::UnityAssetBalanceConversion;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
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
	pub PayoutFraction: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(40);
	pub static InflationType: u8 = 0;
}
pub enum StatedOnSessionEnd {}
impl darwinia_staking::IssuingManager<Runtime> for StatedOnSessionEnd {
	fn inflate() -> Balance {
		if INFLATION_TYPE.with(|v| *v.borrow()) == 0 {
			OnDarwiniaSessionEnd::inflate()
		} else {
			OnCrabSessionEnd::inflate()
		}
	}

	fn calculate_reward(issued: Balance) -> Balance {
		if INFLATION_TYPE.with(|v| *v.borrow()) == 0 {
			OnDarwiniaSessionEnd::calculate_reward(issued)
		} else {
			OnCrabSessionEnd::calculate_reward(issued)
		}
	}

	fn reward(who: &AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		if INFLATION_TYPE.with(|v| *v.borrow()) == 0 {
			OnDarwiniaSessionEnd::reward(who, amount)
		} else {
			OnCrabSessionEnd::reward(who, amount)
		}
	}
}
pub enum OnDarwiniaSessionEnd {}
impl darwinia_staking::IssuingManager<Runtime> for OnDarwiniaSessionEnd {
	fn inflate() -> Balance {
		let now = Timestamp::now();
		let session_duration = now - <darwinia_staking::SessionStartTime<Runtime>>::get();
		let elapsed_time = <darwinia_staking::ElapsedTime<Runtime>>::mutate(|t| {
			*t = t.saturating_add(session_duration);

			*t
		});

		<darwinia_staking::SessionStartTime<Runtime>>::put(now);

		dc_inflation::issuing_in_period(session_duration, elapsed_time).unwrap_or_default()
	}

	fn calculate_reward(issued: Balance) -> Balance {
		PayoutFraction::get() * issued
	}

	fn reward(who: &AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		let _ = Balances::deposit_creating(who, amount);

		Ok(())
	}
}
pub enum OnCrabSessionEnd {}
impl darwinia_staking::IssuingManager<Runtime> for OnCrabSessionEnd {
	fn calculate_reward(_inflation: Balance) -> Balance {
		10_000 * UNIT
	}

	fn reward(who: &AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		<Balances as Currency<AccountId>>::transfer(
			&Treasury::account_id(),
			who,
			amount,
			frame_support::traits::ExistenceRequirement::KeepAlive,
		)
	}
}
pub enum ShouldEndSession {}
impl frame_support::traits::Get<bool> for ShouldEndSession {
	fn get() -> bool {
		// polkadot-sdk
		use pallet_session::ShouldEndSession;

		<Runtime as pallet_session::Config>::ShouldEndSession::should_end_session(
			System::block_number(),
		)
	}
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
			frame_support::traits::ExistenceRequirement::AllowDeath,
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

	fn stake(_: &Self::AccountId, _: Self::Item) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn unstake(_: &Self::AccountId, _: Self::Item) -> sp_runtime::DispatchResult {
		Ok(())
	}
}
impl darwinia_staking::Config for Runtime {
	type Currency = Balances;
	type Deposit = Deposit;
	type IssuingManager = StatedOnSessionEnd;
	type Kton = KtonStaking;
	type KtonRewardDistributionContract = ();
	type RewardToKton = ();
	type MaxDeposits = <Self as darwinia_deposit::Config>::MaxDeposits;
	type Ring = RingStaking;
	type RuntimeEvent = RuntimeEvent;
	type ShouldEndSession = ShouldEndSession;
	type WeightInfo = ();
}
#[cfg(not(feature = "runtime-benchmarks"))]
impl darwinia_staking::DepositConfig for Runtime {}

frame_support::construct_runtime! {
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Deposit: darwinia_deposit,
		Session: pallet_session,
		Treasury: pallet_treasury,
		Staking: darwinia_staking,
	}
}

pub enum Efflux {}
impl Efflux {
	pub fn time(milli_secs: Moment) {
		Timestamp::set_timestamp(Timestamp::now() + milli_secs);
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
		let _ = pretty_env_logger::try_init();
		let mut storage =
			<frame_system::GenesisConfig<Runtime>>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: (1..=10)
				.map(|i| (i, 1_000 * UNIT))
				.chain([(Treasury::account_id(), 1_000_000 * UNIT)])
				.collect(),
		}
		.assimilate_storage(&mut storage)
		.unwrap();
		darwinia_staking::GenesisConfig::<Runtime> {
			rate_limit: 100 * UNIT,
			collator_count: self.collator_count,
			collators: if self.genesis_collator {
				(1..=self.collator_count).map(|i| (i, UNIT)).collect()
			} else {
				Default::default()
			},
			..Default::default()
		}
		.assimilate_storage(&mut storage)
		.unwrap();
		if self.genesis_collator {
			pallet_session::GenesisConfig::<Runtime> {
				keys: (1..=self.collator_count)
					.map(|i| (i, i, SessionKeys { uint: (i as u64).into() }))
					.collect(),
			}
			.assimilate_storage(&mut storage)
			.unwrap();
		}

		let mut ext = TestExternalities::from(storage);

		ext.execute_with(|| new_session());

		ext
	}
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self { collator_count: 1, genesis_collator: false }
	}
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
	darwinia_staking::call_on_exposure!(<Previous<Runtime>>::iter_keys().for_each(|c| {
		let _ = Staking::payout_inner(c);
	}))
	.unwrap();
}
