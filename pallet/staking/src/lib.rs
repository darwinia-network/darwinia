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

//! # Darwinia parachain staking pallet
//!
//! ## Overview
//!
//! This is a completely specialized stake pallet designed only for Darwinia parachain.
//! So, this pallet will eliminate the generic parameters as much as possible.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![allow(clippy::needless_borrows_for_generic_args)]

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
use pallet_session::ShouldEndSession as _;
use sp_core::H160;
use sp_runtime::{
	traits::{AccountIdConversion, Convert, One, Zero},
	Perbill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

/// Make it easier to call a function on a specific collators storage.
#[macro_export]
macro_rules! call_on_cache {
	($s_e:expr, <$s:ident<$t:ident>>$($f:tt)*) => {{
		match $s_e {
			($crate::CacheState::$s, _, _) => Ok(<$crate::CollatorsCache0<$t>>$($f)*),
			(_, $crate::CacheState::$s, _) => Ok(<$crate::CollatorsCache1<$t>>$($f)*),
			(_, _, $crate::CacheState::$s) => Ok(<$crate::CollatorsCache2<$t>>$($f)*),
			_ => {
				log::error!("collators cache states must be correct; qed");

				Err("[pallet::staking] collators cache states must be correct; qed")
			},
		}
	}};
	(<$s:ident<$t:ident>>$($f:tt)*) => {{
		let s = <$crate::CacheStates<$t>>::get();

		$crate::call_on_cache!(s, <$s<$t>>$($f)*)
	}};
}

const PAYOUT_FRAC: Perbill = Perbill::from_percent(40);

#[frame_support::pallet]
pub mod pallet {
	// darwinia
	use crate::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Override the [`frame_system::Config::RuntimeEvent`].
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsic in this pallet.
		type WeightInfo: WeightInfo;

		/// Unix time interface.
		type UnixTime: UnixTime;

		/// Pass [`pallet_session::Config::ShouldEndSession`]'s result to here.
		type ShouldEndSession: Get<bool>;

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
		/// A payout has been made for the staker.
		Payout { who: T::AccountId, amount: Balance },
		/// Unable to pay the staker's reward.
		Unpaid { who: T::AccountId, amount: Balance },
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

	/// Cache states.
	///
	/// To avoid extra DB RWs during new session, such as:
	/// ```nocompile
	/// previous = current;
	/// current = next;
	/// next = elect();
	/// ```
	///
	/// Now, with data:
	/// ```nocompile
	/// cache1 == previous;
	/// cache2 == current;
	/// cache3 == next;
	/// ```
	/// Just need to shift the marker and write the storage map once:
	/// ```nocompile
	/// mark(cache3, current);
	/// mark(cache2, previous);
	/// mark(cache1, next);
	/// cache1 = elect();
	/// ```
	#[pallet::storage]
	#[pallet::getter(fn exposure_cache_states)]
	pub type CacheStates<T: Config> =
		StorageValue<_, (CacheState, CacheState, CacheState), ValueQuery, CacheStatesDefault<T>>;
	/// Default value for [`CacheStates`].
	#[pallet::type_value]
	pub fn CacheStatesDefault<T: Config>() -> (CacheState, CacheState, CacheState) {
		(CacheState::Previous, CacheState::Current, CacheState::Next)
	}

	/// The ideal number of active collators.
	#[pallet::storage]
	#[pallet::getter(fn collator_count)]
	pub type CollatorCount<T> = StorageValue<_, u32, ValueQuery>;

	/// Number of blocks authored by the collator within current session.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn authored_block_count)]
	pub type AuthoredBlocksCount<T: Config> =
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

	// TODO: use `BoundedVec`.
	/// Collators cache 0.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn collators_cache_0)]
	pub type CollatorsCache0<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Collators cache 1.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn collators_cache_1)]
	pub type CollatorsCache1<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Collators cache 2.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn collators_cache_2)]
	pub type CollatorsCache2<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[derive(DefaultNoBound)]
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Current timestamp.
		pub now: Moment,
		/// The running time of Darwinia1.
		pub elapsed_time: Moment,
		/// Genesis collator count.
		pub collator_count: u32,
		_marker: PhantomData<T>,
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
	pub struct Pallet<T>(_);
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_: BlockNumberFor<T>) -> Weight {
			// There are already plenty of tasks to handle during the new session,
			// so refrain from assigning any additional ones here.
			if !T::ShouldEndSession::get() {
				call_on_cache!(<Previous<T>>::get())
					.map(|cs| {
						let mut cs = cs.into_iter();
						let w = cs
							.by_ref()
							// ? make this value adjustable.
							.take(1)
							.fold(Weight::zero(), |acc, c| {
								acc + Self::payout_for_inner(c).unwrap_or(Zero::zero())
							});
						let _ = call_on_cache!(<Previous<T>>::put(cs.collect::<Vec<_>>()));

						w
					})
					.unwrap_or_default()
			} else {
				Zero::zero()
			}
		}

		fn on_idle(_: BlockNumberFor<T>, mut remaining_weight: Weight) -> Weight {
			// At least 1 read weight is required.
			if let Some(rw) = remaining_weight.checked_sub(&T::DbWeight::get().reads(1)) {
				remaining_weight = rw;
			} else {
				return remaining_weight;
			}

			#[cfg(feature = "test")]
			let wt = Weight::zero().add_ref_time(1);
			#[cfg(not(feature = "test"))]
			let wt = T::WeightInfo::unstake_all();
			let mut ledger_to_migrate = Vec::new();

			for (w, l) in <Ledgers<T>>::iter() {
				if let Some(rw) = remaining_weight.checked_sub(&wt) {
					remaining_weight = rw;

					ledger_to_migrate.push((w, l));
				} else {
					break;
				}
			}

			for (w, l) in ledger_to_migrate {
				let _ = Self::unstake_all_for_inner(w, l);
			}

			remaining_weight
		}
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Withdraw all stakes from the staking pool.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::unstake_all())]
		pub fn unstake_all_for(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_signed(origin)?;

			let l = <Ledgers<T>>::take(&who).ok_or(<Error<T>>::NoRecord)?;

			Self::unstake_all_for_inner(who, l)?;

			Ok(())
		}

		/// Making the payout for the specified.
		#[pallet::call_index(8)]
		#[pallet::weight(<T as Config>::WeightInfo::payout_for())]
		pub fn payout_for(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_signed(origin)?;

			Self::payout_for_inner(who)?;

			Ok(())
		}

		/// Set the collator count.
		///
		/// This will apply to the incoming session.
		///
		/// Require root origin.
		#[pallet::call_index(7)]
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
		#[pallet::call_index(11)]
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
		#[pallet::call_index(10)]
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
		/// Distribute the session reward to staking pot and update the stakers' reward record.
		pub fn distribute_session_reward(amount: Balance) {
			let who = <T as Config>::Treasury::get();

			if T::IssuingManager::reward(&who, amount).is_ok() {
				Self::deposit_event(Event::Payout { who, amount });
			} else {
				Self::deposit_event(Event::Unpaid { who, amount });
			}

			let reward_r = amount.saturating_div(2);
			let reward_k = amount.saturating_sub(reward_r);
			let (b_total, map) = <AuthoredBlocksCount<T>>::take();

			map.into_iter().for_each(|(c, b)| {
				let r = Perbill::from_rational(b, b_total).mul_floor(reward_r);

				<PendingRewards<T>>::mutate(c, |u| *u = u.map(|u| u + r).or(Some(r)));
			});

			T::KtonStaking::distribute(None, reward_k);
		}

		fn unstake_all_for_inner(who: T::AccountId, ledger: Ledger) -> DispatchResult {
			T::Currency::transfer(
				&account_id(),
				&who,
				ledger.ring,
				ExistenceRequirement::AllowDeath,
			)?;

			Ok(())
		}

		/// Pay the reward to the RING staking contract.
		fn payout_for_inner(collator: T::AccountId) -> Result<Weight, DispatchError> {
			if call_on_cache!(<Previous<T>>::get()).unwrap_or_default().contains(&collator) {
				T::RingStaking::distribute(
					Some(collator.clone()),
					<PendingRewards<T>>::take(&collator).ok_or(<Error<T>>::NoReward)?,
				);
			} else {
				// Impossible case.

				Err(<Error<T>>::NoReward)?;
			}

			Ok(<T as Config>::WeightInfo::payout_for())
		}

		/// Update the record of block production.
		fn note_authors(authors: &[T::AccountId]) {
			<AuthoredBlocksCount<T>>::mutate(|(total, map)| {
				authors.iter().cloned().for_each(|c| {
					*total += One::one();

					map.entry(c).and_modify(|p_| *p_ += One::one()).or_insert(One::one());
				});
			});
		}

		/// Prepare the session state.
		fn prepare_new_session(index: u32) -> Option<Vec<T::AccountId>> {
			<Pallet<T>>::shift_cache_states();

			call_on_cache!(<Next<T>>::kill()).ok()?;

			let bn = <frame_system::Pallet<T>>::block_number();

			log::info!("assembling new collators for new session {index} at #{bn:?}",);

			let cs = Self::elect().unwrap_or_default();

			if cs.is_empty() {
				// TODO: update this once the migration is completed.
				// Possible case under the hybrid election mode.

				// Impossible case.
				//
				// But if there is an issue, retain the old collators; do not alter the session
				// collators if any error occurs to prevent the chain from stalling.
				None
			} else {
				Some(cs)
			}
		}

		/// Shift the cache states.
		///
		/// Previous Current  Next
		/// Next     Previous Current
		/// Current  Next     Previous
		///
		/// ```nocompile
		/// loop { mutate(2, 0, 1) }
		/// ```
		fn shift_cache_states() {
			let (s0, s1, s2) = <CacheStates<T>>::get();

			<CacheStates<T>>::put((s2, s0, s1));
		}

		fn elect() -> Option<Vec<T::AccountId>> {
			let winners = T::RingStaking::elect(<CollatorCount<T>>::get())?;

			call_on_cache!(<Next<T>>::put(winners.clone())).ok()?;

			Some(winners)
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

		fn new_session(index: u32) -> Option<Vec<T::AccountId>> {
			let maybe_collators = Self::prepare_new_session(index);

			if maybe_collators.is_none() {
				log::error!("fail to elect collators for session {index}");
			}

			maybe_collators
		}
	}
}
pub use pallet::*;

/// Issuing and reward manager.
pub trait IssuingManager<T>
where
	T: Config,
{
	/// Generic session termination procedures.
	fn on_session_end() {
		let inflation = Self::inflate();
		let reward = Self::calculate_reward(inflation);

		<Pallet<T>>::distribute_session_reward(reward);
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
	fn reward(_: &T::AccountId, _: Balance) -> DispatchResult {
		Ok(())
	}
}
impl<T> IssuingManager<T> for () where T: Config {}

/// Election interface.
pub trait Election<AccountId> {
	/// Elect the new collators.
	fn elect(_: u32) -> Option<Vec<AccountId>> {
		None
	}
}
impl<AccountId> Election<AccountId> for () {}

/// Distribute the reward to a contract.
pub trait Reward<AccountId> {
	/// Distribute the reward.
	fn distribute(_: Option<AccountId>, _: Balance) {}
}
impl<AccountId> Reward<AccountId> for () {}

/// UnstakeAll trait that stakes must be implemented.
pub trait UnstakeAll {
	/// Account type.
	type AccountId;

	/// Withdraw all stakes from the staking pool.
	fn unstake_all(who: &Self::AccountId) -> DispatchResult;
}

/// Cache state.
#[allow(missing_docs)]
#[cfg_attr(any(test, feature = "runtime-benchmarks", feature = "try-runtime"), derive(PartialEq))]
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub enum CacheState {
	Previous,
	Current,
	Next,
}

/// Session ending checker.
pub struct ShouldEndSession<T>(PhantomData<T>);
impl<T> Get<bool> for ShouldEndSession<T>
where
	T: frame_system::Config + pallet_session::Config,
{
	fn get() -> bool {
		<T as pallet_session::Config>::ShouldEndSession::should_end_session(
			<frame_system::Pallet<T>>::block_number(),
		)
	}
}

/// Issue new token from pallet-balances.
pub struct BalanceIssuing<T>(PhantomData<T>);
impl<T> IssuingManager<T> for BalanceIssuing<T>
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

	fn reward(who: &T::AccountId, amount: Balance) -> DispatchResult {
		let _ = T::Currency::deposit_creating(who, amount);

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

	fn reward(who: &T::AccountId, amount: Balance) -> DispatchResult {
		let treasury = <T as Config>::Treasury::get();

		if who == &treasury {
			Ok(())
		} else {
			T::Currency::transfer(&treasury, who, amount, ExistenceRequirement::KeepAlive)
		}
	}
}

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
	fn elect(n: u32) -> Option<Vec<T::AccountId>> {
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
			.encode_input(&[Token::Uint(n.into())])
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
			log::info!("getTopCollators({n})'s execution info {i:?}");

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
	fn distribute(who: Option<T::AccountId>, amount: Balance) {
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
	fn distribute(_: Option<T::AccountId>, amount: Balance) {
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
