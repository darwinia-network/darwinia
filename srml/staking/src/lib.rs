// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(feature = "bench", test), feature(test))]
#![feature(drain_filter)]

#[cfg(all(feature = "bench", test))]
extern crate test;

use parity_codec::{CompactAs, Decode, Encode, HasCompact};
use primitives::traits::{
	Bounded, CheckedSub, Convert, One, SaturatedConversion, Saturating, StaticLookup, Zero,
};
use primitives::Perbill;
#[cfg(feature = "std")]
use primitives::{Deserialize, Serialize};
use rstd::{collections::btree_map::BTreeMap, prelude::*, result};
#[cfg(feature = "std")]
use runtime_io::with_storage;
use session::{OnSessionEnding, SessionIndex};
use srml_support::{
	decl_event, decl_module, decl_storage, ensure,
	traits::{
		Currency, Get, Imbalance, LockIdentifier, LockableCurrency, OnFreeBalanceZero,
		OnUnbalanced, WithdrawReason, WithdrawReasons,
	},
	EnumerableStorageMap, StorageMap, StorageValue,
};
use system::ensure_signed;

use phragmen::{elect, equalize, ExtendedBalance, ACCURACY};

mod utils;

#[cfg(any(feature = "bench", test))]
mod mock;
//
#[cfg(test)]
mod tests;

mod phragmen;

//#[cfg(all(feature = "bench", test))]
//mod benches;

const RECENT_OFFLINE_COUNT: usize = 32;
const DEFAULT_MINIMUM_VALIDATOR_COUNT: u32 = 4;
const MAX_NOMINATIONS: usize = 16;
const MAX_UNSTAKE_THRESHOLD: u32 = 10;
const MAX_UNLOCKING_CHUNKS: usize = 32;
const MONTH_IN_SECONDS: u32 = 2_592_000;
const STAKING_ID: LockIdentifier = *b"staking ";

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum StakerStatus<AccountId> {
	/// Chilling.
	Idle,
	/// Declared desire in validating or already participating in it.
	Validator,
	/// Nominating for a group of other stakers.
	Nominator(Vec<AccountId>),
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ValidatorPrefs {
	/// Validator should ensure this many more slashes than is necessary before being unstaked.
	#[codec(compact)]
	pub unstake_threshold: u32,
	/// percent of Reward that validator takes up-front; only the rest is split between themselves and
	/// nominators.
	pub validator_payment_ratio: Perbill,
}

impl Default for ValidatorPrefs {
	fn default() -> Self {
		ValidatorPrefs {
			unstake_threshold: 3,
			validator_payment_ratio: Default::default(),
		}
	}
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum StakingBalance<RingBalance, KtonBalance> {
	Ring(RingBalance),
	Kton(KtonBalance),
}

impl<RingBalance: Default, KtonBalance: Default> Default
	for StakingBalance<RingBalance, KtonBalance>
{
	fn default() -> Self {
		StakingBalance::Ring(Default::default())
	}
}

/// A destination account for payment.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum RewardDestination {
	/// Pay into the stash account, increasing the amount at stake accordingly.
	/// for now, we dont use this.
	//    DeprecatedStaked,
	/// Pay into the stash account, not increasing the amount at stake.
	Stash,
	/// Pay into the controller account.
	Controller,
}

impl Default for RewardDestination {
	fn default() -> Self {
		RewardDestination::Stash
	}
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct UnlockChunk<StakingBalance> {
	/// Amount of funds to be unlocked.
	value: StakingBalance,
	/// Era number at which point it'll be unlocked.
	#[codec(compact)]
	era: EraIndex,
	is_time_deposit: bool,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct TimeDepositItem<RingBalance: HasCompact, Moment> {
	#[codec(compact)]
	value: RingBalance,
	#[codec(compact)]
	start_time: Moment,
	#[codec(compact)]
	expire_time: Moment,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct StakingLedgers<
	AccountId,
	RingBalance: HasCompact,
	KtonBalance: HasCompact,
	StakingBalance,
	Moment,
> {
	pub stash: AccountId,
	// normal pattern: for ring
	/// total_ring = nomarl_ring + time_deposit_ring
	#[codec(compact)]
	pub total_ring: RingBalance,
	#[codec(compact)]
	pub total_deposit_ring: RingBalance,
	#[codec(compact)]
	pub active_ring: RingBalance,
	// active time-deposit ring
	#[codec(compact)]
	pub active_deposit_ring: RingBalance,
	#[codec(compact)]
	pub total_kton: KtonBalance,
	#[codec(compact)]
	pub active_kton: KtonBalance,
	// time-deposit items:
	// if you deposit ring for a minimum period,
	// you can get KTON as bonus
	// which can also be used for staking
	pub deposit_items: Vec<TimeDepositItem<RingBalance, Moment>>,
	pub unlocking: Vec<UnlockChunk<StakingBalance>>,
}

/// The amount of exposure (to slashing) than an individual nominator has.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct IndividualExpo<AccountId, Power> {
	/// The stash account of the nominator in question.
	who: AccountId,
	/// Amount of funds exposed.
	value: Power,
}

/// A snapshot of the stake backing a single validator in the system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Exposures<AccountId, Power> {
	/// The total balance backing this validator.
	pub total: Power,
	/// The validator's own stash that is exposed.
	pub own: Power,
	/// The portions of nominators stashes that are exposed.
	pub others: Vec<IndividualExpo<AccountId, Power>>,
}

type RingBalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;
type KtonBalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::Balance;

// for ring
type RingPositiveImbalanceOf<T> =
	<<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type RingNegativeImbalanceOf<T> =
	<<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

// for kton
type KtonPositiveImbalanceOf<T> =
	<<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type KtonNegativeImbalanceOf<T> =
	<<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

type RawAssignment<T> = (<T as system::Trait>::AccountId, ExtendedBalance);
type Assignment<T> = (
	<T as system::Trait>::AccountId,
	ExtendedBalance,
	ExtendedBalance,
);
type ExpoMap<T> = BTreeMap<
	<T as system::Trait>::AccountId,
	Exposures<<T as system::Trait>::AccountId, ExtendedBalance>,
>;

pub trait Trait: timestamp::Trait + session::Trait {
	type Ring: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	type Kton: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

	type CurrencyToVote: Convert<KtonBalanceOf<Self>, u64> + Convert<u128, KtonBalanceOf<Self>>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// Handler for the unbalanced reduction when slashing a staker.
	type RingSlash: OnUnbalanced<RingNegativeImbalanceOf<Self>>;

	/// Handler for the unbalanced increment when rewarding a staker.
	type RingReward: OnUnbalanced<RingPositiveImbalanceOf<Self>>;

	type KtonSlash: OnUnbalanced<KtonNegativeImbalanceOf<Self>>;
	type KtonReward: OnUnbalanced<KtonPositiveImbalanceOf<Self>>;

	/// Number of sessions per era.
	type SessionsPerEra: Get<SessionIndex>;

	/// Number of eras that staked funds must remain bonded for.
	type BondingDuration: Get<EraIndex>;

	// custom
	type Cap: Get<<Self::Ring as Currency<Self::AccountId>>::Balance>;
	type ErasPerEpoch: Get<EraIndex>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Staking {

		pub ValidatorCount get(validator_count) config(): u32;

		pub MinimumValidatorCount get(minimum_validator_count) config():
			u32 = DEFAULT_MINIMUM_VALIDATOR_COUNT;

		pub SessionReward get(session_reward) config(): Perbill = Perbill::from_percent(60);

		pub OfflineSlash get(offline_slash) config(): Perbill = Perbill::from_parts(1000);

		pub OfflineSlashGrace get(offline_slash_grace) config(): u32;

		pub Invulnerables get(invulnerables) config(): Vec<T::AccountId>;

		pub Bonded get(bonded): map T::AccountId => Option<T::AccountId>;

		pub Ledger get(ledger): map T::AccountId => Option<StakingLedgers<
			T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>, StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
			T::Moment>>;

		pub Payee get(payee): map T::AccountId => RewardDestination;

		pub Validators get(validators): linked_map T::AccountId => ValidatorPrefs;

		pub Nominators get(nominators): linked_map T::AccountId => Vec<T::AccountId>;

		pub Stakers get(stakers): map T::AccountId => Exposures<T::AccountId, ExtendedBalance>;

		pub CurrentElected get(current_elected): Vec<T::AccountId>;

		pub CurrentEra get(current_era) config(): EraIndex;

		pub SlotStake get(slot_stake): ExtendedBalance;

		pub SlashCount get(slash_count): map T::AccountId => u32;

		pub RecentlyOffline get(recently_offline): Vec<(T::AccountId, T::BlockNumber, u32)>;

		pub ForceNewEra get(forcing_new_era): bool;

		pub EpochIndex get(epoch_index): u32 = 0;

		/// The accumulated reward for the current era. Reset to zero at the beginning of the era
		/// and increased for every successfully finished session.
		pub CurrentEraTotalReward get(current_era_total_reward) config(): RingBalanceOf<T>;

		pub NodeName get(node_name): map T::AccountId => Vec<u8>;

		pub RingPool get(ring_pool): RingBalanceOf<T>;

		pub KtonPool get(kton_pool): KtonBalanceOf<T>;
	}
	add_extra_genesis {
		config(stakers):
			Vec<(T::AccountId, T::AccountId, RingBalanceOf<T>, StakerStatus<T::AccountId>)>;
		build(|
			storage: &mut primitives::StorageOverlay,
			_: &mut primitives::ChildrenStorageOverlay,
			config: &GenesisConfig<T>
		| {
			with_storage(storage, || {
				for &(ref stash, ref controller, balance, ref status) in &config.stakers {
					assert!(T::Ring::free_balance(&stash) >= balance);
					let _ = <Module<T>>::bond(
						T::Origin::from(Some(stash.clone()).into()),
						T::Lookup::unlookup(controller.clone()),
						StakingBalance::Ring(balance),
						RewardDestination::Stash,
						12
					);
					let _ = match status {
						StakerStatus::Validator => {
							<Module<T>>::validate(
								T::Origin::from(Some(controller.clone()).into()),
								[0;8].to_vec(),
								0,
								3
							)
						},
						StakerStatus::Nominator(votes) => {
							<Module<T>>::nominate(
								T::Origin::from(Some(controller.clone()).into()),
								votes.iter().map(|l| {T::Lookup::unlookup(l.clone())}).collect()
							)
						}, _ => Ok(())
					};
				}

				if let (_, Some(validators)) = <Module<T>>::select_validators() {
					<session::Validators<T>>::put(&validators);
				}
			});
		});
	}
}

decl_event!(
    pub enum Event<T> where Balance = RingBalanceOf<T>, <T as system::Trait>::AccountId {
        /// All validators have been rewarded by the given balance.
		Reward(Balance),
		/// One validator (and its nominators) has been given an offline-warning (it is still
		/// within its grace). The accrued number of slashes is recorded, too.
		OfflineWarning(AccountId, u32),
		/// One validator (and its nominators) has been slashed by the given ratio.
		OfflineSlash(AccountId, u32),
		/// NodeName changed
	    NodeNameUpdated,
    }
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Number of sessions per era.
		const SessionsPerEra: SessionIndex = T::SessionsPerEra::get();

		/// Number of eras that staked funds must remain bonded for.
		const BondingDuration: EraIndex = T::BondingDuration::get();

		fn deposit_event<T>() = default;

		fn bond(origin,
			controller: <T::Lookup as StaticLookup>::Source,
			value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
			payee: RewardDestination,
			promise_month: u32
		) {
			let stash = ensure_signed(origin)?;
			ensure!( promise_month <= 36, "months at most is 36.");

			if <Bonded<T>>::exists(&stash) {
				return Err("stash already bonded")
			}

			let controller = T::Lookup::lookup(controller)?;

			if <Ledger<T>>::exists(&controller) {
				return Err("controller already paired")
			}

			<Bonded<T>>::insert(&stash, &controller);
			<Payee<T>>::insert(&stash, payee);

			let ledger = StakingLedgers {stash: stash.clone(), ..Default::default()};
			match value {
				StakingBalance::Ring(r) => {
					let stash_balance = T::Ring::free_balance(&stash);
					let value = r.min(stash_balance);
					// increase ring pool
					<RingPool<T>>::mutate(|r| *r += value);
					Self::bond_helper_in_ring(&stash, &controller, value, promise_month, ledger);
				},
				StakingBalance::Kton(k) => {
					let stash_balance = T::Kton::free_balance(&stash);
					let value: KtonBalanceOf<T> = k.min(stash_balance);
					// increase kton pool
					<KtonPool<T>>::mutate(|k| *k += value);
					Self::bond_helper_in_kton(&controller, value, ledger);
				},
			}
		}

		fn bond_extra(origin,
			value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
			promise_month: u32
		) {
			let stash = ensure_signed(origin)?;
			ensure!( promise_month <= 36, "months at most is 36.");
			let controller = Self::bonded(&stash).ok_or("not a stash")?;
			let ledger = Self::ledger(&controller).ok_or("not a controller")?;
			match value {
				 StakingBalance::Ring(r) => {
					let stash_balance = T::Ring::free_balance(&stash);
					if let Some(extra) = stash_balance.checked_sub(&ledger.total_ring) {
						let extra = extra.min(r);
						<RingPool<T>>::mutate(|r| *r += extra);
						Self::bond_helper_in_ring(&stash, &controller, extra, promise_month, ledger);
					}
				},
				StakingBalance::Kton(k) => {
					let stash_balance = T::Kton::free_balance(&stash);
					if let Some(extra) = stash_balance.checked_sub(&ledger.total_kton) {
						let extra = extra.min(k);
						<KtonPool<T>>::mutate(|k| *k += extra);
						Self::bond_helper_in_kton(&controller, extra, ledger);
					}
				},
			}
		}

		/// for normal_ring or normal_kton, follow the original substrate pattern
		/// for time_deposit_ring, transform it into normal_ring first
		/// modify time_deposit_items and time_deposit_ring amount
		fn unbond(origin, value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>) {
			let controller = ensure_signed(origin)?;
			let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;
			let StakingLedgers {
				active_ring,
				active_deposit_ring,
				active_kton,
				deposit_items,
				unlocking,
				..
			} = &mut ledger;

			ensure!(
				unlocking.len() < MAX_UNLOCKING_CHUNKS,
				"can not schedule more unlock chunks"
			);

			let era = Self::current_era() + T::BondingDuration::get();

			match value {
				StakingBalance::Ring(r) => {
					// total_unbond_value = normal_unbond + time_deposit_unbond
					let total_value = r.min(*active_ring);
					let active_normal_ring = *active_ring - *active_deposit_ring;
				    // unbond normal ring first
					let active_normal_value = total_value.min(active_normal_ring);

					<RingPool<T>>::mutate(|r| *r -= active_normal_value);
					let mut unlock_value_left = total_value - active_normal_value;

					if !active_normal_value.is_zero() {
						*active_ring -= active_normal_value;
						unlocking.push(UnlockChunk {
							value: StakingBalance::Ring(total_value),
							era,
							is_time_deposit: false
						});
					}

					// no active_normal_ring
					let is_time_deposit = active_normal_value.is_zero() || !unlock_value_left.is_zero();
					let mut total_deposit_changed = 0.into();

					if is_time_deposit {
						let now = <timestamp::Module<T>>::now();

						/// for time_deposit_ring, transform into normal one
						deposit_items.drain_filter(|item| {
							if item.expire_time > now {
								return false;
							}

							// NOTE: value that a user wants to unbond must
							// be big enough to unlock all time_deposit_ring
							// double check

							if unlock_value_left.is_zero() {
								return true;
							}

							let value = unlock_value_left.min(item.value);

							unlock_value_left = unlock_value_left.saturating_sub(value);

							*active_deposit_ring = active_deposit_ring.saturating_sub(value);
							*active_ring = active_ring.saturating_sub(value);

							total_deposit_changed += value;
							item.value -= value;

							item.value.is_zero()
						});

						// update unlocking list
						unlocking.push(UnlockChunk {
							value: StakingBalance::Ring(total_deposit_changed),
							era,
							is_time_deposit: true,
						});
						 <RingPool<T>>::mutate(|r| *r -= total_deposit_changed);
					}
				},
				StakingBalance::Kton(k) => {
					let value = k.min(*active_kton);
					<KtonPool<T>>::mutate(|k| *k -= value);

					*active_kton -= value;
					unlocking.push(UnlockChunk {
						value: StakingBalance::Kton(value),
						era,
						is_time_deposit: false,
					});
				},
			}

			<Ledger<T>>::insert(&controller, ledger);
		}

		// NOTE: considered that expire_time won't
		fn unbond_with_punish(origin, value: RingBalanceOf<T>, expire_time: T::Moment) {
			let controller = ensure_signed(origin)?;
			let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;
			let StakingLedgers {
				stash,
				active_ring,
				active_deposit_ring,
				deposit_items,
				unlocking,
				..
			} = &mut ledger;
			let now = <timestamp::Module<T>>::now();

			ensure!(expire_time > now, "use unbond instead.");
			deposit_items.drain_filter(|item| {
				if item.expire_time != expire_time {
					return false;
				}

				let value = item.value.min(value);
				// at least 1 month
				let month_left = (
					(expire_time.clone() - now.clone()).saturated_into::<u32>()
					/ MONTH_IN_SECONDS
				).max(1);
				let kton_slash = utils::compute_kton_return::<T>(value, month_left) * 3.into();

				// check total free balance and locked one
				// strict on punishing in kton
				if T::Kton::free_balance(stash)
					.checked_sub(&kton_slash)
					.and_then(|new_balance| {
						T::Kton::ensure_can_withdraw(
							stash,
							kton_slash,
							WithdrawReason::Transfer,
							new_balance
						).ok()
					})
					.is_none() {
						return false;
					}

				// update ring
				item.value -= value;
				*active_ring = active_ring.saturating_sub(value);
				*active_deposit_ring = active_deposit_ring.saturating_sub(value);

				let (imbalance, _) = T::Kton::slash(stash, kton_slash);

				T::KtonSlash::on_unbalanced(imbalance);

				// update unlocks
				unlocking.push(UnlockChunk {
					value: StakingBalance::Ring(value),
					era: Self::current_era() + T::BondingDuration::get(),
					is_time_deposit: true
				});

				item.value.is_zero()
			});

			<Ledger<T>>::insert(&controller, ledger);

		}

		/// called by controller
		fn promise_extra(origin, value: RingBalanceOf<T>, promise_month: u32) {
			let controller = ensure_signed(origin)?;

			ensure!( promise_month <= 36, "months at most is 36.");
			let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;
			let StakingLedgers {
				active_ring,
				total_deposit_ring,
				active_deposit_ring,
				deposit_items,
				stash,
				..
			} = &mut ledger;
			// remove expired deposit_items
			let now = <timestamp::Module<T>>::now();
			deposit_items.retain(|item| {
				if item.expire_time < now {
					// reduce deposit_ring,
					// total / active ring
					*active_deposit_ring = active_deposit_ring.saturating_sub(item.value);
					*total_deposit_ring = total_deposit_ring.saturating_sub(item.value);

					false
				} else {
					true
				}
			});

			let value = value.min(*active_ring - *active_deposit_ring); // active_normal_ring

			if promise_month >= 3 {
				// update time_deposit_ring
				// while total_ring stays the same
				*total_deposit_ring += value;
				*active_deposit_ring += value;

				// for now, kton_return is free
				// mint kton
				let kton_return = utils::compute_kton_return::<T>(value, promise_month);
				let kton_positive_imbalance = T::Kton::deposit_creating(stash, kton_return);
				T::KtonReward::on_unbalanced(kton_positive_imbalance);

				let expire_time = now.clone() + (MONTH_IN_SECONDS * promise_month).into();
				deposit_items.push(TimeDepositItem {
					value,
					start_time: now,
					expire_time,
				});
			}

			<Ledger<T>>::insert(&controller, ledger);
		}

		/// may both withdraw ring and kton at the same time
		fn withdraw_unbonded(origin) {
			let controller = ensure_signed(origin)?;
			let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;
			let StakingLedgers {
				total_ring,
				total_deposit_ring,
				total_kton,
				unlocking,
				..
			} = &mut ledger;
			let mut balance_kind = 0u8;
			let current_era = Self::current_era();

			unlocking.retain(|UnlockChunk {
				value,
				era,
				is_time_deposit,
			}| {
				if *era > current_era {
					return true;
				}

				match value {
					StakingBalance::Ring(ring) => {
						balance_kind |= 0b01;
						*total_ring = total_ring.saturating_sub(*ring);

						if *is_time_deposit {
							*total_deposit_ring = total_deposit_ring.saturating_sub(*ring);
						}

					}
					StakingBalance::Kton(kton) => {
						balance_kind |= 0b10;
						*total_kton = total_kton.saturating_sub(*kton);
					}
				}

				false
			});

			match balance_kind {
				0 => (),
				1 => Self::update_ledger(&controller, &ledger, StakingBalance::Ring(0.into())),
				2 => Self::update_ledger(&controller, &ledger, StakingBalance::Kton(0.into())),
				3 => {
					Self::update_ledger(&controller, &ledger, StakingBalance::Ring(0.into()));
					Self::update_ledger(&controller, &ledger, StakingBalance::Kton(0.into()));
				}
				_ => unreachable!(),
			}
		}

		fn validate(origin, name: Vec<u8>, ratio: u32, unstake_threshold: u32) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or("not a controller")?;
			let stash = &ledger.stash;
			ensure!(
				unstake_threshold <= MAX_UNSTAKE_THRESHOLD,
				"unstake threshold too large"
			);
			// at most 100%
			let ratio = Perbill::from_percent(ratio.min(100));
			let prefs = ValidatorPrefs {unstake_threshold: unstake_threshold, validator_payment_ratio: ratio };

			<Nominators<T>>::remove(stash);
			<Validators<T>>::insert(stash, prefs);
			if !<NodeName<T>>::exists(&controller) {
				<NodeName<T>>::insert(controller, name);
				Self::deposit_event(RawEvent::NodeNameUpdated);
			}
		}

		fn nominate(origin, targets: Vec<<T::Lookup as StaticLookup>::Source>) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or("not a controller")?;
			let stash = &ledger.stash;
			ensure!(!targets.is_empty(), "targets cannot be empty");
			let targets = targets.into_iter()
				.take(MAX_NOMINATIONS)
				.map(T::Lookup::lookup)
				.collect::<result::Result<Vec<T::AccountId>, &'static str>>()?;

			<Validators<T>>::remove(stash);
			<Nominators<T>>::insert(stash, targets);
		}

		fn chill(origin) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or("not a controller")?;
			let stash = &ledger.stash;
			<Validators<T>>::remove(stash);
			<Nominators<T>>::remove(stash);
		}

		fn set_payee(origin, payee: RewardDestination) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or("not a controller")?;
			let stash = &ledger.stash;
			<Payee<T>>::insert(stash, payee);
		}

		fn set_controller(origin, controller: <T::Lookup as StaticLookup>::Source) {
			let stash = ensure_signed(origin)?;
			let old_controller = Self::bonded(&stash).ok_or("not a stash")?;
			let controller = T::Lookup::lookup(controller)?;
			if <Ledger<T>>::exists(&controller) {
				return Err("controller already paired")
			}
			if controller != old_controller {
				<Bonded<T>>::insert(&stash, &controller);
				if let Some(l) = <Ledger<T>>::take(&old_controller) {
					<Ledger<T>>::insert(&controller, l);
				}
			}
		}

		/// The ideal number of validators.
		fn set_validator_count(#[compact] new: u32) {
			ValidatorCount::put(new);
		}

		// ----- Root calls.

		fn force_new_era() {
			Self::apply_force_new_era()
		}

		/// Set the offline slash grace period.
		fn set_offline_slash_grace(#[compact] new: u32) {
			OfflineSlashGrace::put(new);
		}

		/// Set the validators who cannot be slashed (if any).
		fn set_invulnerables(validators: Vec<T::AccountId>) {
			<Invulnerables<T>>::put(validators);
		}
	}
}

impl<T: Trait> Module<T> {
	/// The total that can be slashed from a validator controller account as of
	/// right now.
	pub fn slashable_balance(who: &T::AccountId) -> ExtendedBalance {
		Self::stakers(who).total
	}

	fn bond_helper_in_ring(
		stash: &T::AccountId,
		controller: &T::AccountId,
		value: RingBalanceOf<T>,
		promise_month: u32,
		mut ledger: StakingLedgers<
			T::AccountId,
			RingBalanceOf<T>,
			KtonBalanceOf<T>,
			StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
			T::Moment,
		>,
	) {
		// if stash promise to a extra-lock
		// there will be extra reward, kton, which
		// can also be use to stake.
		if promise_month >= 3 {
			ledger.active_deposit_ring += value;
			ledger.total_deposit_ring += value;
			// for now, kton_return is free
			// mint kton
			let kton_return = utils::compute_kton_return::<T>(value, promise_month);
			let kton_positive_imbalance = T::Kton::deposit_creating(&stash, kton_return);
			T::KtonReward::on_unbalanced(kton_positive_imbalance);
			let now = <timestamp::Module<T>>::now();
			let expire_time = now.clone() + (MONTH_IN_SECONDS * promise_month).into();
			ledger.deposit_items.push(TimeDepositItem {
				value,
				start_time: now,
				expire_time,
			});
		}
		ledger.active_ring = ledger.active_ring.saturating_add(value);
		ledger.total_ring = ledger.total_ring.saturating_add(value);

		Self::update_ledger(&controller, &ledger, StakingBalance::Ring(value));
	}

	fn bond_helper_in_kton(
		controller: &T::AccountId,
		value: KtonBalanceOf<T>,
		mut ledger: StakingLedgers<
			T::AccountId,
			RingBalanceOf<T>,
			KtonBalanceOf<T>,
			StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
			T::Moment,
		>,
	) {
		ledger.total_kton += value;
		ledger.active_kton += value;

		Self::update_ledger(&controller, &ledger, StakingBalance::Kton(value));
	}

	fn update_ledger(
		controller: &T::AccountId,
		ledger: &StakingLedgers<
			T::AccountId,
			RingBalanceOf<T>,
			KtonBalanceOf<T>,
			StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
			T::Moment,
		>,
		staking_balance: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
	) {
		match staking_balance {
			StakingBalance::Ring(_r) => T::Ring::set_lock(
				STAKING_ID,
				&ledger.stash,
				ledger.total_ring,
				T::BlockNumber::max_value(),
				WithdrawReasons::all(),
			),

			StakingBalance::Kton(_k) => T::Kton::set_lock(
				STAKING_ID,
				&ledger.stash,
				ledger.total_kton,
				T::BlockNumber::max_value(),
				WithdrawReasons::all(),
			),
		}

		<Ledger<T>>::insert(controller, ledger);
	}

	fn slash_validator(stash: &T::AccountId, slash_ratio_in_u32: u32) {
		// construct Perbill here to make sure slash_ratio lt 0.
		let slash_ratio = Perbill::from_parts(slash_ratio_in_u32);
		// The exposures (backing stake) information of the validator to be slashed.
		let exposures = Self::stakers(stash);

		let (mut ring_imbalance, mut kton_imbalance) = Self::slash_individual(stash, slash_ratio);

		for i in exposures.others.iter() {
			let (rn, kn) = Self::slash_individual(&i.who, slash_ratio);
			ring_imbalance.subsume(rn);
			kton_imbalance.subsume(kn);
		}

		T::RingSlash::on_unbalanced(ring_imbalance);
		T::KtonSlash::on_unbalanced(kton_imbalance);
	}

	fn slash_individual(
		stash: &T::AccountId,
		slash_ratio: Perbill,
	) -> (RingNegativeImbalanceOf<T>, KtonNegativeImbalanceOf<T>) {
		let controller = Self::bonded(stash).unwrap();
		let mut ledger = Self::ledger(&controller).unwrap();

		// slash ring
		let ring_imbalance = if ledger.total_ring.is_zero() {
			<RingNegativeImbalanceOf<T>>::zero()
		} else {
			let slashable_ring = slash_ratio * ledger.total_ring;
			let value_slashed = Self::slash_helper(
				&controller,
				&mut ledger,
				StakingBalance::Ring(slashable_ring),
			);

			T::Ring::slash(stash, value_slashed.0).0
		};
		let kton_imbalance = if ledger.total_kton.is_zero() {
			<KtonNegativeImbalanceOf<T>>::zero()
		} else {
			let slashable_kton = slash_ratio * ledger.total_kton;
			let value_slashed = Self::slash_helper(
				&controller,
				&mut ledger,
				StakingBalance::Kton(slashable_kton),
			);

			T::Kton::slash(stash, value_slashed.1).0
		};

		(ring_imbalance, kton_imbalance)
	}

	fn slash_helper(
		controller: &T::AccountId,
		ledger: &mut StakingLedgers<
			T::AccountId,
			RingBalanceOf<T>,
			KtonBalanceOf<T>,
			StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
			T::Moment,
		>,
		value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
	) -> (RingBalanceOf<T>, KtonBalanceOf<T>) {
		match value {
			StakingBalance::Ring(r) => {
				let StakingLedgers {
					total_ring,
					active_ring,
					total_deposit_ring,
					active_deposit_ring,
					deposit_items,
					..
				} = ledger;

				// if slashing ring, first slashing normal ring
				// then, slashing time-deposit ring
				// TODO: check one more time (may be removed later)
				let total_value = r.min(*active_ring);
				let normal_active_value = total_value.min(*active_ring - *active_deposit_ring);

				// to prevent overflow
				// first slash normal bonded ring
				<RingPool<T>>::mutate(|r| *r -= normal_active_value);
				*active_ring -= normal_active_value;
				*total_ring -= normal_active_value;

				// bonded + unlocking
				// first slash active normal ring
				let mut value_left = total_value - normal_active_value;
				// then slash active time-promise ring
				// from the nearest expire time
				if !value_left.is_zero() {
					// sorted by expire_time from far to near
					deposit_items.sort_unstable_by_key(|item| {
						u64::max_value() - item.expire_time.clone().saturated_into::<u64>()
					});
					deposit_items.drain_filter(|item| {
						if value_left.is_zero() {
							return false;
						}

						let value_removed = value_left.min(item.value);

						*total_ring -= value_removed;
						*active_ring -= value_removed;
						*total_deposit_ring -= value_removed;
						*active_deposit_ring -= value_removed;

						item.value -= value_removed;
						value_left -= value_removed;

						<RingPool<T>>::mutate(|ring| *ring -= value_removed);

						item.value.is_zero()
					});
				}

				Self::update_ledger(controller, ledger, StakingBalance::Ring(0.into()));
				(total_value, 0.into())
			}
			StakingBalance::Kton(k) => {
				// check one more time
				// TODO: may be removed later
				let active_value = k.min(ledger.active_kton);
				// first slash active kton
				ledger.active_kton -= active_value;
				ledger.total_kton -= active_value;
				<KtonPool<T>>::mutate(|k| *k -= active_value);

				Self::update_ledger(controller, ledger, StakingBalance::Kton(0.into()));
				(0.into(), active_value)
			}
		}
	}

	fn new_session(session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		if ForceNewEra::take() || session_index % T::SessionsPerEra::get() == 0 {
			Self::new_era()
		} else {
			None
		}
	}

	/// The era has changed - enact new staking set.
	///
	/// NOTE: This always happens immediately before a session change to ensure that new validators
	/// get a chance to set their session keys.
	fn new_era() -> Option<Vec<T::AccountId>> {
		let reward = Self::session_reward() * Self::current_era_total_reward();
		if !reward.is_zero() {
			let validators = Self::current_elected();
			let len = validators.len() as u32; // validators length can never overflow u64
			let len: RingBalanceOf<T> = len.max(1).into();
			let block_reward_per_validator = reward / len;
			for v in validators.iter() {
				Self::reward_validator(v, block_reward_per_validator);
			}
			Self::deposit_event(RawEvent::Reward(block_reward_per_validator));

			// TODO: reward to treasury
		}

		// Increment current era.
		CurrentEra::mutate(|s| *s += 1);

		// check if ok to change epoch
		if Self::current_era() % T::ErasPerEpoch::get() == 0 {
			Self::new_epoch();
		}

		// Reassign all Stakers.
		let (_, maybe_new_validators) = Self::select_validators();

		maybe_new_validators
	}

	fn new_epoch() {
		EpochIndex::mutate(|e| *e += 1);
		let next_era_reward = utils::compute_current_era_reward::<T>();
		if !next_era_reward.is_zero() {
			<CurrentEraTotalReward<T>>::put(next_era_reward);
		}
	}

	fn reward_validator(stash: &T::AccountId, reward: RingBalanceOf<T>) {
		let off_the_table = Self::validators(stash).validator_payment_ratio * reward;
		let reward = reward - off_the_table;
		let mut imbalance = <RingPositiveImbalanceOf<T>>::zero();
		let validator_cut = if reward.is_zero() {
			Zero::zero()
		} else {
			let exposures = Self::stakers(stash);
			let total = exposures.total.max(One::one());

			for i in &exposures.others {
				let per_u64 = Perbill::from_rational_approximation(i.value, total);
				imbalance.maybe_subsume(Self::make_payout(&i.who, per_u64 * reward));
			}

			let per_u64 = Perbill::from_rational_approximation(exposures.own, total);
			per_u64 * reward
		};
		imbalance.maybe_subsume(Self::make_payout(stash, validator_cut + off_the_table));
		T::RingReward::on_unbalanced(imbalance);
	}

	/// Actually make a payment to a staker. This uses the currency's reward function
	/// to pay the right payee for the given staker account.
	fn make_payout(
		stash: &T::AccountId,
		amount: RingBalanceOf<T>,
	) -> Option<RingPositiveImbalanceOf<T>> {
		let dest = Self::payee(stash);
		match dest {
			RewardDestination::Controller => Self::bonded(stash)
				.and_then(|controller| T::Ring::deposit_into_existing(&controller, amount).ok()),
			RewardDestination::Stash => T::Ring::deposit_into_existing(stash, amount).ok(),
		}
	}

	// TODO: ready for hacking
	// power is a mixture of ring and kton
	fn slashable_balance_of(stash: &T::AccountId) -> ExtendedBalance {
		Self::bonded(stash)
			.and_then(Self::ledger)
			.map(|l| {
				l.active_ring.saturated_into::<ExtendedBalance>()
					+ l.active_kton.saturated_into::<ExtendedBalance>() * Self::kton_vote_weight()
						/ ACCURACY
			})
			.unwrap_or_default()
	}

	/// Select a new validator set from the assembled stakers and their role preferences.
	///
	/// Returns the new `SlotStake` value.
	fn select_validators() -> (ExtendedBalance, Option<Vec<T::AccountId>>) {
		let maybe_elected_set = elect::<T, _, _, _>(
			Self::validator_count() as usize,
			Self::minimum_validator_count().max(1) as usize,
			<Validators<T>>::enumerate(),
			<Nominators<T>>::enumerate(),
			Self::slashable_balance_of,
		);

		if let Some(elected_set) = maybe_elected_set {
			let elected_stashes = elected_set.0;
			let assignments = elected_set.1;

			// The return value of this is safe to be converted to u64.
			// The original balance, `b` is within the scope of u64. It is just extended to u128
			// to be properly multiplied by a ratio, which will lead to another value
			// less than u64 for sure. The result can then be safely passed to `to_balance`.
			// For now the backward convert is used. A simple `TryFrom<u64>` is also safe.
			let ratio_of = |b, p| (p as ExtendedBalance).saturating_mul(b) / ACCURACY;

			// Compute the actual stake from nominator's ratio.
			let assignments_with_stakes = assignments
				.iter()
				.map(|(n, a)| {
					(
						n.clone(),
						Self::slashable_balance_of(n),
						a.iter()
							.map(|(acc, r)| {
								(acc.clone(), *r, ratio_of(Self::slashable_balance_of(n), *r))
							})
							.collect::<Vec<Assignment<T>>>(),
					)
				})
				.collect::<Vec<(T::AccountId, ExtendedBalance, Vec<Assignment<T>>)>>();

			// update elected candidate exposures.
			let mut exposures = <ExpoMap<T>>::new();
			elected_stashes
				.iter()
				.map(|e| (e, Self::slashable_balance_of(e)))
				.for_each(|(e, s)| {
					let item = Exposures {
						own: s,
						total: s,
						..Default::default()
					};
					exposures.insert(e.clone(), item);
				});

			for (n, _, assignment) in &assignments_with_stakes {
				for (c, _, s) in assignment {
					if let Some(expo) = exposures.get_mut(c) {
						// NOTE: simple example where this saturates:
						// candidate with max_value stake. 1 nominator with max_value stake.
						// Nuked. Sadly there is not much that we can do about this.
						// See this test: phragmen_should_not_overflow_xxx()
						expo.total = expo.total.saturating_add(*s);
						expo.others.push(IndividualExpo {
							who: n.clone(),
							value: *s,
						});
					}
				}
			}

			if cfg!(feature = "equalize") {
				let tolerance = 0_u128;
				let iterations = 2_usize;
				let mut assignments_with_votes = assignments_with_stakes
					.iter()
					.map(|a| {
						(
							a.0.clone(),
							a.1,
							a.2.iter().map(|e| (e.0.clone(), e.1, e.2)).collect::<Vec<(
								T::AccountId,
								ExtendedBalance,
								ExtendedBalance,
							)>>(),
						)
					})
					.collect::<Vec<(
						T::AccountId,
						ExtendedBalance,
						Vec<(T::AccountId, ExtendedBalance, ExtendedBalance)>,
					)>>();
				equalize::<T>(
					&mut assignments_with_votes,
					&mut exposures,
					tolerance,
					iterations,
				);
			}

			// Clear Stakers and reduce their slash_count.
			for v in Self::current_elected().iter() {
				<Stakers<T>>::remove(v);
				let slash_count = <SlashCount<T>>::take(v);
				if slash_count > 1 {
					<SlashCount<T>>::insert(v, slash_count - 1);
				}
			}

			// Populate Stakers and figure out the minimum stake behind a slot.
			let mut slot_stake = ExtendedBalance::max_value();
			for (c, e) in exposures.iter() {
				if e.total < slot_stake {
					slot_stake = e.total;
				}
				<Stakers<T>>::insert(c.clone(), e.clone());
			}

			// Update slot stake.
			SlotStake::put(&slot_stake);

			// Set the new validator set in sessions.
			<CurrentElected<T>>::put(&elected_stashes);
			let validators = elected_stashes
				.into_iter()
				.map(|s| Self::bonded(s).unwrap_or_default())
				.collect::<Vec<_>>();
			(slot_stake, Some(validators))
		} else {
			// There were not enough candidates for even our minimal level of functionality.
			// This is bad.
			// We should probably disable all functionality except for block production
			// and let the chain keep producing blocks until we can decide on a sufficiently
			// substantial set.
			// TODO: #2494
			(Self::slot_stake(), None)
		}
	}

	fn apply_force_new_era() {
		ForceNewEra::put(true);
	}

	/// Call when a validator is determined to be offline. `count` is the
	/// number of offenses the validator has committed.
	///
	/// NOTE: This is called with the controller (not the stash) account id.
	pub fn on_offline_validator(controller: T::AccountId, count: usize) {
		let stash = if let Some(l) = Self::ledger(&controller) {
			l.stash
		} else {
			return;
		};

		// Early exit if validator is invulnerable.
		if Self::invulnerables().contains(&stash) {
			return;
		}

		let slash_count = Self::slash_count(&stash);
		let new_slash_count = slash_count + count as u32;
		<SlashCount<T>>::insert(&stash, new_slash_count);
		let grace = Self::offline_slash_grace();

		if RECENT_OFFLINE_COUNT > 0 {
			let item = (
				stash.clone(),
				<system::Module<T>>::block_number(),
				count as u32,
			);
			<RecentlyOffline<T>>::mutate(|v| {
				if v.len() >= RECENT_OFFLINE_COUNT {
					*v.iter_mut()
						.min_by(|(_, block_a, _), (_, block_b, _)| block_a.cmp(&block_b))
						.expect("v is non-empty; qed") = item;
				} else {
					v.push(item);
				}
			});
		}

		if <Validators<T>>::exists(&stash) {
			let prefs = Self::validators(&stash);
			let unstake_threshold = prefs.unstake_threshold.min(MAX_UNSTAKE_THRESHOLD);
			let max_slashes = grace + unstake_threshold;

			let event = if new_slash_count > max_slashes {
				let offline_slash_ratio_base = *Self::offline_slash().encode_as();
				// slash_ratio is ensured to be less than 1 in slash_validator
				// don't worry here.
				let slash_ratio_in_u32 = offline_slash_ratio_base
					.checked_shl(unstake_threshold)
					.unwrap_or_default();
				Self::slash_validator(&stash, slash_ratio_in_u32);
				<Validators<T>>::remove(&stash);
				let _ = <session::Module<T>>::disable(&controller);

				RawEvent::OfflineSlash(stash.clone(), slash_ratio_in_u32)
			} else {
				RawEvent::OfflineWarning(stash.clone(), slash_count)
			};

			Self::deposit_event(event);
		}
	}

	// total_kton * kton_vote_weight / ACCURACY = total_ring
	// it ensures that when rewarding validators
	// reward to ring_pool will be the same with the
	// reward to kton_pool
	// that means 50% reward is distributed to ring holders,
	// another 50% reward is distributed to kton holders
	fn kton_vote_weight() -> ExtendedBalance {
		let total_ring = Self::ring_pool().saturated_into::<ExtendedBalance>();
		// to avoid 'attempt to divide by zero'
		let total_kton = Self::kton_pool().saturated_into::<ExtendedBalance>().max(1);
		// total_ring and total_kton are within the scope of u64
		// so it is safe to multiply ACCURACY when extended to u128
		total_ring * ACCURACY / total_kton
	}
}

impl<T: Trait> OnSessionEnding<T::AccountId> for Module<T> {
	fn on_session_ending(i: SessionIndex) -> Option<Vec<T::AccountId>> {
		Self::new_session(i + 1)
	}
}

impl<T: Trait> OnFreeBalanceZero<T::AccountId> for Module<T> {
	fn on_free_balance_zero(stash: &T::AccountId) {
		if let Some(controller) = <Bonded<T>>::take(stash) {
			<Ledger<T>>::remove(&controller);
		}
		<Payee<T>>::remove(stash);
		<SlashCount<T>>::remove(stash);
		<Validators<T>>::remove(stash);
		<Nominators<T>>::remove(stash);
	}
}
