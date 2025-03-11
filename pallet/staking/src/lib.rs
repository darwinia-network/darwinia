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
		let (mut r, mut w) = (0, 0);

		fn clear(item: &[u8], r: &mut u64, w: &mut u64) {
			let res = migration::clear_storage_prefix(PALLET, item, &[], None, None);

			*r += res.loops as u64;
			*w += res.backend as u64;
		}

		clear(b"UnissuedReward", &mut r, &mut w);
		clear(b"SessionStartTime", &mut r, &mut w);
		clear(b"ElapsedTime", &mut r, &mut w);

		StorageVersion::new(4).put::<Pallet<T>>();

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

		assert_is_none(b"UnissuedReward");
		assert_is_none(b"SessionStartTime");
		assert_is_none(b"ElapsedTime");
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
use dc_types::Balance;
// polkadot-sdk
use frame_support::{pallet_prelude::*, PalletId};
use frame_system::pallet_prelude::*;
use sp_core::H160;
use sp_runtime::{
	traits::{AccountIdConversion, Convert, One},
	Perbill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

#[frame_support::pallet]
pub mod pallet {
	// darwinia
	use crate::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Override the [`frame_system::Config::RuntimeEvent`].
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsic in this pallet.
		type WeightInfo: WeightInfo;

		/// RING staking interface.
		type RingStaking: Election<Self::AccountId> + Reward<Self::AccountId>;

		/// KTON staking interface.
		type KtonStaking: Reward<Self::AccountId>;

		/// Treasury address.
		#[pallet::constant]
		type Treasury: Get<Self::AccountId>;

		/// Reward amount per session.
		#[pallet::constant]
		type RewardPerSession: Get<Balance>;
	}

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Reward allocated to the account.
		RewardAllocated { who: T::AccountId, amount: Balance },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No reward to pay for this collator.
		NoReward,
	}

	/// The ideal number of active collators.
	#[pallet::storage]
	pub type CollatorCount<T> = StorageValue<_, u32, ValueQuery>;

	/// Number of blocks authored by the collator within current session.
	#[pallet::storage]
	#[pallet::unbounded]
	pub type AuthoredBlockCount<T: Config> =
		StorageValue<_, (BlockNumberFor<T>, BTreeMap<T::AccountId, BlockNumberFor<T>>), ValueQuery>;

	/// All outstanding rewards since the last payment.
	#[pallet::storage]
	pub type PendingRewards<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Balance>;

	/// RING staking contract address.
	#[pallet::storage]
	pub type RingStakingContract<T: Config> = StorageValue<_, T::AccountId>;
	/// KTON staking contract address.
	#[pallet::storage]
	pub type KtonStakingContract<T: Config> = StorageValue<_, T::AccountId>;

	/// Unallocated collator RING rewards.
	#[pallet::storage]
	pub type UnallocatedRingRewards<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Balance>;

	/// Unallocated collator KTON rewards.
	///
	/// The destination is the KTON staking contract.
	#[pallet::storage]
	pub type UnallocatedKtonRewards<T> = StorageValue<_, Balance, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T> {
		/// Genesis collator count.
		pub collator_count: u32,
		#[allow(missing_docs)]
		pub _marker: PhantomData<T>,
	}
	impl<T> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { collator_count: 1, _marker: Default::default() }
		}
	}
	#[pallet::genesis_build]
	impl<T> BuildGenesisConfig for GenesisConfig<T>
	where
		T: Config,
	{
		fn build(&self) {
			<CollatorCount<T>>::put(self.collator_count.max(1));
		}
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_idle(_: BlockNumberFor<T>, mut remaining_weight: Weight) -> Weight {
			Self::idle_allocate_ring_staking_reward(&mut remaining_weight);

			remaining_weight
		}
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
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

			<CollatorCount<T>>::put(count.max(1));

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

			log::info!("assembling new collators for new session {i} at #{bn:?}");

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
			Self::allocate_session_reward(T::RewardPerSession::get());
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
			&[Token::Address(who.clone().into())],
			rsc,
			amount.into(),
			1_000_000.into(),
		) {
			log::error!("failed to forward call due to {e:?}");

			<UnallocatedRingRewards<T>>::mutate(who, |u| u.map(|u| u + amount).or(Some(amount)));
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

			<UnallocatedKtonRewards<T>>::mutate(|u| *u += amount);
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
