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

//! # Darwinia Staking Pallet
//!
//! ## Overview
//!
//! This is a completely specialized stake pallet designed only for Darwinia.
//! So, this pallet will eliminate the generic parameters as much as possible.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![allow(clippy::needless_borrows_for_generic_args)]

#[allow(missing_docs)]
pub mod migration {
	// darwinia
	use crate::*;
	// polkadot-sdk
	use frame_support::migration;

	const PALLET: &[u8] = b"DarwiniaStaking";

	pub fn migrate<T>() -> (u64, u64)
	where
		T: Config,
	{
		let ver = StorageVersion::get::<Pallet<T>>();
		let (mut r, mut w) = (1, 0);

		if ver != 2 {
			log::warn!(
				"\
				[pallet::staking] skipping v2 to v3 migration: executed on wrong storage version.\
				Expected version 2, found {ver:?}\
				",
			);

			return (r, w);
		}

		fn clear(item: &[u8], r: &mut u64, w: &mut u64) {
			let res = migration::clear_storage_prefix(PALLET, item, &[], None, None);

			*r += res.loops as u64;
			*w += res.backend as u64;
		}

		clear(b"Collators", &mut r, &mut w);
		clear(b"Nominators", &mut r, &mut w);
		clear(b"ExposureCacheStates", &mut r, &mut w);
		clear(b"ExposureCache0", &mut r, &mut w);
		clear(b"ExposureCache1", &mut r, &mut w);
		clear(b"ExposureCache2", &mut r, &mut w);
		clear(b"CacheStates", &mut r, &mut w);
		clear(b"CollatorsCache0", &mut r, &mut w);
		clear(b"CollatorsCache1", &mut r, &mut w);
		clear(b"CollatorsCache2", &mut r, &mut w);
		clear(b"MigrationStartPoint", &mut r, &mut w);
		clear(b"RateLimit", &mut r, &mut w);
		clear(b"RateLimitState", &mut r, &mut w);

		if let Some(abc) = migration::take_storage_value::<(
			BlockNumberFor<T>,
			BTreeMap<T::AccountId, BlockNumberFor<T>>,
		)>(PALLET, b"AuthoredBlocksCount", &[])
		{
			<AuthoredBlockCount<T>>::put(abc);

			r += 1;
			w += 1;
		}

		StorageVersion::new(3).put::<Pallet<T>>();

		w += 1;

		(r, w)
	}

	pub fn post_check<T>()
	where
		T: Config,
	{
		fn assert_is_none(item: &[u8]) {
			assert!(!migration::have_storage_value(PALLET, item, &[]));
		}

		assert_is_none(b"Collators");
		assert_is_none(b"Nominators");
		assert_is_none(b"ExposureCacheStates");
		assert_is_none(b"ExposureCache0");
		assert_is_none(b"ExposureCache1");
		assert_is_none(b"ExposureCache2");
		assert_is_none(b"CacheStates");
		assert_is_none(b"CollatorsCache0");
		assert_is_none(b"CollatorsCache1");
		assert_is_none(b"CollatorsCache2");
		assert_is_none(b"MigrationStartPoint");
		assert_is_none(b"RateLimit");
		assert_is_none(b"RateLimitState");
		assert_is_none(b"AuthoredBlocksCount");

		assert!(<AuthoredBlockCount<T>>::exists());
	}
}

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;
pub use weights::WeightInfo;

// crates.io
use codec::FullCodec;
use ethabi::{Function, Param, ParamType, StateMutability, Token};
// darwinia
use dc_types::{Balance, Moment};
// polkadot-sdk
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, UnixTime},
	DefaultNoBound, PalletId,
};
use frame_system::pallet_prelude::*;
use sp_core::H160;
use sp_runtime::{
	traits::{AccountIdConversion, Convert, One},
	Perbill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

const PAYOUT_FRAC: Perbill = Perbill::from_percent(40);

#[frame_support::pallet]
pub mod pallet {
	// darwinia
	use crate::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Override the [`frame_system::Config::RuntimeEvent`].
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsic in this pallet.
		type WeightInfo: WeightInfo;

		/// Unix time interface.
		type UnixTime: UnixTime;

		/// Currency interface to pay the reward.
		type Currency: Currency<Self::AccountId, Balance = Balance>;

		/// Inflation and reward manager.
		type IssuingManager: IssuingManager<Self>;

		/// RING staking interface.
		type RingStaking: Election<Self::AccountId> + Reward<Self::AccountId>;

		/// KTON staking interface.
		type KtonStaking: Reward<Self::AccountId>;

		/// Treasury address.
		type Treasury: Get<Self::AccountId>;
	}

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Reward allocated to the account.
		RewardAllocated { who: T::AccountId, amount: Balance },
		/// Fail to allocate the reward to the account.
		RewardAllocationFailed { who: T::AccountId, amount: Balance },
		/// Unstake all stakes for the account.
		UnstakeAllFor { who: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Collator count mustn't be zero.
		ZeroCollatorCount,
		/// No record for the account.
		NoRecord,
		/// No reward to pay for this collator.
		NoReward,
	}

	/// All staking ledgers.
	#[pallet::storage]
	#[pallet::getter(fn ledger_of)]
	pub type Ledgers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Ledger>;

	/// The ideal number of active collators.
	#[pallet::storage]
	#[pallet::getter(fn collator_count)]
	pub type CollatorCount<T> = StorageValue<_, u32, ValueQuery>;

	/// Number of blocks authored by the collator within current session.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn authored_block_count)]
	pub type AuthoredBlockCount<T: Config> =
		StorageValue<_, (BlockNumberFor<T>, BTreeMap<T::AccountId, BlockNumberFor<T>>), ValueQuery>;

	/// All outstanding rewards since the last payment.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn pending_reward_of)]
	pub type PendingRewards<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Balance>;

	/// Active session's start-time.
	#[pallet::storage]
	#[pallet::getter(fn session_start_time)]
	pub type SessionStartTime<T: Config> = StorageValue<_, Moment, ValueQuery>;

	/// Elapsed time.
	#[pallet::storage]
	#[pallet::getter(fn elapsed_time)]
	pub type ElapsedTime<T: Config> = StorageValue<_, Moment, ValueQuery>;

	/// RING staking contract address.
	#[pallet::storage]
	#[pallet::getter(fn ring_staking_contract)]
	pub type RingStakingContract<T: Config> = StorageValue<_, T::AccountId>;
	/// KTON staking contract address.
	#[pallet::storage]
	#[pallet::getter(fn kton_staking_contract)]
	pub type KtonStakingContract<T: Config> = StorageValue<_, T::AccountId>;

	#[derive(DefaultNoBound)]
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Current timestamp.
		pub now: Moment,
		/// The running time of Darwinia1.
		pub elapsed_time: Moment,
		/// Genesis collator count.
		pub collator_count: u32,
		#[allow(missing_docs)]
		pub _marker: PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			if self.collator_count == 0 {
				panic!("[pallet::staking] collator count mustn't be 0");
			}

			<SessionStartTime<T>>::put(self.now);
			<ElapsedTime<T>>::put(self.elapsed_time);
			<CollatorCount<T>>::put(self.collator_count);
		}
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_idle(_: BlockNumberFor<T>, mut remaining_weight: Weight) -> Weight {
			Self::idle_allocate_ring_staking_reward(&mut remaining_weight);
			Self::idle_unstake(&mut remaining_weight);

			remaining_weight
		}
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Withdraw all stakes.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::unstake_all_for())]
		pub fn unstake_all_for(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_signed(origin)?;

			let leger = <Ledgers<T>>::take(&who).ok_or(<Error<T>>::NoRecord)?;

			Self::unstake_all_for_inner(who, leger)?;

			Ok(())
		}

		/// Allocate the RING staking rewards to the designated RING staking contract of a
		/// particular collator.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::allocate_ring_staking_reward_of())]
		pub fn allocate_ring_staking_reward_of(
			origin: OriginFor<T>,
			who: T::AccountId,
		) -> DispatchResult {
			ensure_signed(origin)?;

			let amount = <PendingRewards<T>>::take(&who).ok_or(<Error<T>>::NoReward)?;

			Self::allocate_ring_staking_reward_of_inner(who, amount)?;

			Ok(())
		}

		/// Set the collator count.
		///
		/// This will apply to the incoming session.
		///
		/// Require root origin.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::set_collator_count())]
		pub fn set_collator_count(origin: OriginFor<T>, count: u32) -> DispatchResult {
			ensure_root(origin)?;

			if count == 0 {
				return Err(<Error<T>>::ZeroCollatorCount)?;
			}

			<CollatorCount<T>>::put(count);

			Ok(())
		}

		/// Set the RING reward distribution contract address.
		///
		/// Require root origin.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::set_ring_staking_contract())]
		pub fn set_ring_staking_contract(
			origin: OriginFor<T>,
			address: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			<RingStakingContract<T>>::put(address);

			Ok(())
		}

		/// Set the KTON reward distribution contract address.
		///
		/// Require root origin.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::set_kton_staking_contract())]
		pub fn set_kton_staking_contract(
			origin: OriginFor<T>,
			address: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			<KtonStakingContract<T>>::put(address);

			Ok(())
		}
	}
	impl<T> Pallet<T>
	where
		T: Config,
	{
		pub(crate) fn note_authors(authors: &[T::AccountId]) {
			<AuthoredBlockCount<T>>::mutate(|(total_block_count, author_map)| {
				authors.iter().cloned().for_each(|who| {
					author_map
						.entry(who)
						.and_modify(|authored_block_count| *authored_block_count += One::one())
						.or_insert(One::one());

					*total_block_count += One::one();
				});
			});
		}

		/// Allocate the session reward.
		pub fn allocate_session_reward(amount: Balance) {
			let treasury = <T as Config>::Treasury::get();

			if T::IssuingManager::reward(amount).is_ok() {
				Self::deposit_event(Event::RewardAllocated { who: treasury, amount });
			} else {
				Self::deposit_event(Event::RewardAllocationFailed { who: treasury, amount });
			}

			let reward_to_ring_staking = amount.saturating_div(2);
			let reward_to_kton_staking = amount.saturating_sub(reward_to_ring_staking);
			let (total_block_count, author_map) = <AuthoredBlockCount<T>>::take();

			author_map.into_iter().for_each(|(who, authored_block_count)| {
				let incoming_reward =
					Perbill::from_rational(authored_block_count, total_block_count)
						.mul_floor(reward_to_ring_staking);

				<PendingRewards<T>>::mutate(who, |maybe_pending_reward| {
					*maybe_pending_reward = maybe_pending_reward
						.map(|pending_reward| pending_reward + incoming_reward)
						.or(Some(incoming_reward))
				});
			});

			T::KtonStaking::allocate(None, reward_to_kton_staking);
		}

		pub(crate) fn allocate_ring_staking_reward_of_inner(
			who: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			T::RingStaking::allocate(Some(who.clone()), amount);

			Self::deposit_event(Event::RewardAllocated { who, amount });

			Ok(())
		}

		fn prepare_new_session(i: u32) -> Option<Vec<T::AccountId>> {
			let bn = <frame_system::Pallet<T>>::block_number();

			log::info!("assembling new collators for new session {i} at #{bn:?}",);

			let collators = T::RingStaking::elect(<CollatorCount<T>>::get()).unwrap_or_default();

			if collators.is_empty() {
				None
			} else {
				Some(collators)
			}
		}

		fn idle_allocate_ring_staking_reward(remaining_weight: &mut Weight) {
			const MAX_TASKS: usize = 10;

			#[cfg(test)]
			let wt = Weight::zero().add_ref_time(1);
			#[cfg(not(test))]
			let wt = T::WeightInfo::allocate_ring_staking_reward_of();
			let mut consumer = <PendingRewards<T>>::iter().drain();

			for _ in 0..MAX_TASKS {
				if let Some(rw) = remaining_weight.checked_sub(&wt) {
					*remaining_weight = rw;
				} else {
					break;
				}
				if let Some((k, v)) = consumer.next() {
					let _ = Self::allocate_ring_staking_reward_of_inner(k, v);
				} else {
					// There is nothing to do; add the weight back.
					*remaining_weight += wt;

					break;
				}
			}
		}

		fn idle_unstake(remaining_weight: &mut Weight) {
			const MAX_TASKS: usize = 10;

			#[cfg(test)]
			let wt = Weight::zero().add_ref_time(1);
			#[cfg(not(test))]
			let wt = T::WeightInfo::unstake_all_for();
			let mut consumer = <Ledgers<T>>::iter().drain();

			for _ in 0..MAX_TASKS {
				if let Some(rw) = remaining_weight.checked_sub(&wt) {
					*remaining_weight = rw;
				} else {
					break;
				}
				if let Some((k, v)) = consumer.next() {
					let _ = Self::unstake_all_for_inner(k, v);
				} else {
					// There is nothing to do; add the weight back.
					*remaining_weight += wt;

					break;
				}
			}
		}

		fn unstake_all_for_inner(who: T::AccountId, ledger: Ledger) -> DispatchResult {
			T::Currency::transfer(
				&account_id(),
				&who,
				ledger.ring,
				ExistenceRequirement::AllowDeath,
			)?;

			Self::deposit_event(Event::UnstakeAllFor { who });

			Ok(())
		}
	}
	impl<T> pallet_authorship::EventHandler<T::AccountId, BlockNumberFor<T>> for Pallet<T>
	where
		T: Config + pallet_authorship::Config + pallet_session::Config,
	{
		fn note_author(author: T::AccountId) {
			Self::note_authors(&[author])
		}
	}
	impl<T> pallet_session::SessionManager<T::AccountId> for Pallet<T>
	where
		T: Config,
	{
		fn end_session(_: u32) {
			T::IssuingManager::on_session_end();
		}

		fn start_session(_: u32) {}

		// No election in genesis.
		// Since RING contract isn't available at this time.
		fn new_session_genesis(i: u32) -> Option<Vec<T::AccountId>> {
			Self::prepare_new_session(i);

			None
		}

		fn new_session(i: u32) -> Option<Vec<T::AccountId>> {
			let maybe_collators = Self::prepare_new_session(i);

			if maybe_collators.is_none() {
				log::error!("fail to elect collators for session {i}");
			}

			maybe_collators
		}
	}
}
pub use pallet::*;

/// Election interface.
pub trait Election<AccountId> {
	/// Elect the new collators.
	fn elect(_: u32) -> Option<Vec<AccountId>> {
		None
	}
}
impl<AccountId> Election<AccountId> for () {}

/// Issuing and reward manager.
pub trait IssuingManager<T>
where
	T: Config,
{
	/// Generic session termination procedures.
	fn on_session_end() {
		let inflation = Self::inflate();
		let reward = Self::calculate_reward(inflation);

		<Pallet<T>>::allocate_session_reward(reward);
	}

	/// Inflation settings.
	fn inflate() -> Balance {
		0
	}

	/// Calculate the reward.
	fn calculate_reward(_: Balance) -> Balance {
		0
	}

	/// The reward function.
	fn reward(_: Balance) -> DispatchResult {
		Ok(())
	}
}
impl<T> IssuingManager<T> for () where T: Config {}
/// Issue new token from pallet-balances.
pub struct BalancesIssuing<T>(PhantomData<T>);
impl<T> IssuingManager<T> for BalancesIssuing<T>
where
	T: Config,
{
	fn inflate() -> Balance {
		let now = now::<T>() as Moment;
		let session_duration = now - <SessionStartTime<T>>::get();
		let elapsed_time = <ElapsedTime<T>>::mutate(|t| {
			*t = t.saturating_add(session_duration);

			*t
		});

		<SessionStartTime<T>>::put(now);

		dc_inflation::issuing_in_period(session_duration, elapsed_time).unwrap_or_default()
	}

	fn calculate_reward(issued: Balance) -> Balance {
		PAYOUT_FRAC * issued
	}

	fn reward(amount: Balance) -> DispatchResult {
		let _ = T::Currency::deposit_creating(&T::Treasury::get(), amount);

		Ok(())
	}
}
/// Transfer issued token from pallet-treasury.
pub struct TreasuryIssuing<T, R>(PhantomData<(T, R)>);
impl<T, R> IssuingManager<T> for TreasuryIssuing<T, R>
where
	T: Config,
	R: Get<Balance>,
{
	fn calculate_reward(_: Balance) -> Balance {
		R::get()
	}
}

/// Allocate the reward to a contract.
pub trait Reward<AccountId> {
	/// Allocate the reward.
	fn allocate(_: Option<AccountId>, _: Balance) {}
}
impl<AccountId> Reward<AccountId> for () {}

/// A convertor from collators id.
///
/// Since this pallet does not have stash/controller, this is just identity.
pub struct IdentityCollator;
impl<T> Convert<T, Option<T>> for IdentityCollator {
	fn convert(t: T) -> Option<T> {
		Some(t)
	}
}

/// Staking ledger.
#[derive(Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Ledger {
	/// Staked RING.
	pub ring: Balance,
	/// Staked deposits.
	pub deposits: BoundedVec<u16, ConstU32<512>>,
}

/// RING staking contract interface.
///
/// https://github.com/darwinia-network/DIP-7/blob/2249e3baa065b7e6c42427810b722fafa37628f1/src/collator/CollatorSet.sol#L27.
pub struct RingStaking<T>(PhantomData<T>);
impl<T> Election<T::AccountId> for RingStaking<T>
where
	T: Config + darwinia_ethtx_forwarder::Config,
	T::AccountId: From<H160> + Into<H160>,
{
	fn elect(x: u32) -> Option<Vec<T::AccountId>> {
		const ZERO: [u8; 20] = [0; 20];

		let Some(rsc) = <RingStakingContract<T>>::get() else {
			log::error!("RING staking contract must be some; qed");

			return None;
		};
		let rsc = rsc.into();
		#[allow(deprecated)]
		let function = Function {
			name: "getTopCollators".to_owned(),
			inputs: vec![Param {
				name: "k".to_owned(),
				kind: ParamType::Uint(256),
				internal_type: None,
			}],
			outputs: vec![Param {
				name: "collators".to_owned(),
				kind: ParamType::Array(Box::new(ParamType::Address)),
				internal_type: None,
			}],
			constant: None,
			state_mutability: StateMutability::View,
		};
		let input = function
			.encode_input(&[Token::Uint(x.into())])
			.map_err(|e| log::error!("failed to encode input due to {e:?}"))
			.ok()?;

		<darwinia_ethtx_forwarder::Pallet<T>>::forward_call(
			<T as Config>::Treasury::get().into(),
			rsc,
			input,
			Default::default(),
			1_000_000.into(),
		)
		.map_err(|e| log::error!("failed to forward call due to {e:?}"))
		.ok()
		.and_then(|i| {
			log::info!("getTopCollators({x})'s execution info {i:?}");

			function
				.decode_output(&i.value)
				.map_err(|e| log::error!("failed to decode output due to {e:?}"))
				.ok()
				.map(|o| {
					let Some(Token::Array(addrs)) = o.into_iter().next() else { return Vec::new() };

					addrs
						.into_iter()
						.filter_map(|addr| match addr {
							Token::Address(addr) if addr.0 != ZERO =>
								Some(T::AccountId::from(addr)),
							_ => None,
						})
						.collect()
				})
		})
	}
}
// Distribute the reward to RING staking contract.
//
// https://github.com/darwinia-network/DIP-7/blob/7fa307136586f06c6911ce98d16c88689d91ba8c/src/collator/CollatorStakingHub.sol#L142.
impl<T> Reward<T::AccountId> for RingStaking<T>
where
	T: Config + darwinia_ethtx_forwarder::Config,
	T::AccountId: Into<H160>,
{
	fn allocate(who: Option<T::AccountId>, amount: Balance) {
		let Some(who) = who else {
			log::error!("who must be some; qed");

			return;
		};
		let Some(rsc) = <RingStakingContract<T>>::get() else {
			log::error!("RING staking contract must be some; qed");

			return;
		};
		let rsc = rsc.into();

		#[allow(deprecated)]
		if let Err(e) = darwinia_ethtx_forwarder::quick_forward_transact::<T>(
			<T as Config>::Treasury::get().into(),
			Function {
				name: "distributeReward".into(),
				inputs: vec![Param {
					name: "address".to_owned(),
					kind: ParamType::Address,
					internal_type: None,
				}],
				outputs: Vec::new(),
				constant: None,
				state_mutability: StateMutability::Payable,
			},
			&[Token::Address(who.into())],
			rsc,
			amount.into(),
			1_000_000.into(),
		) {
			log::error!("failed to forward call due to {e:?}");
		}
	}
}

/// KTON staking contract interface.
pub struct KtonStaking<T>(PhantomData<T>);
// Distribute the reward to KTON staking contract.
//
// https://github.com/darwinia-network/KtonDAO/blob/2de20674f2ef90b749ade746d0768c7bda356402/src/staking/KtonDAOVault.sol#L40.
impl<T> Reward<T::AccountId> for KtonStaking<T>
where
	T: Config + darwinia_ethtx_forwarder::Config,
	T::AccountId: Into<H160>,
{
	fn allocate(_: Option<T::AccountId>, amount: Balance) {
		let Some(ksc) = <KtonStakingContract<T>>::get() else {
			log::error!("KTON staking contract must be some; qed");

			return;
		};
		let ksc = ksc.into();

		#[allow(deprecated)]
		if let Err(e) = darwinia_ethtx_forwarder::quick_forward_transact::<T>(
			<T as Config>::Treasury::get().into(),
			Function {
				name: "distributeRewards".into(),
				inputs: Vec::new(),
				outputs: vec![Param {
					name: "success or not".into(),
					kind: ParamType::Bool,
					internal_type: None,
				}],
				constant: None,
				state_mutability: StateMutability::Payable,
			},
			&[],
			ksc,
			amount.into(),
			1_000_000.into(),
		) {
			log::error!("failed to forward call due to {e:?}");
		}
	}
}

/// The account of the staking pot.
pub fn account_id<A>() -> A
where
	A: FullCodec,
{
	PalletId(*b"da/staki").into_account_truncating()
}

/// The current time in milliseconds.
pub fn now<T>() -> Moment
where
	T: Config,
{
	T::UnixTime::now().as_millis()
}
