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
#![feature(drain_filter)]

pub mod inflation;

mod err {
	pub const CONTROLLER_INVALID: &'static str = "Controller Account - INVALID";
	pub const CONTROLLER_ALREADY_PAIRED: &'static str = "Controller Account - ALREADY PAIRED";

	pub const STASH_INVALID: &'static str = "Stash Account - INVALID";
	pub const STASH_ALREADY_BONDED: &'static str = "Stash Account - ALREADY BONDED";

	pub const UNLOCK_CHUNKS_REACH_MAX: &'static str = "Unlock Chunks - REACH MAX VALUE 32";

	pub const CLAIM_DEPOSITS_EXPIRE_TIME_INVALID: &'static str =
		"Claim Deposits With Punish - NOTHING TO CLAIM AT THIS TIME";
	pub const NODE_NAME_INVALID: &'static str = "Node Name - INVALID";
	pub const TARGETS_INVALID: &'static str = "Targets - CAN NOT BE EMPTY";
}

#[allow(unused)]
#[cfg(all(feature = "std", test))]
mod mock;
#[cfg(all(feature = "std", test))]
mod tests;

use codec::{Decode, Encode, HasCompact};
use rstd::{borrow::ToOwned, prelude::*, result};
use session::{historical::OnSessionEnding, SelectInitialValidators};
use sr_primitives::{
	traits::{Bounded, CheckedSub, Convert, One, SaturatedConversion, Saturating, StaticLookup, Zero},
	weights::SimpleDispatchInfo,
	Perbill, Perquintill, RuntimeDebug,
};
#[cfg(feature = "std")]
use sr_primitives::{Deserialize, Serialize};
use sr_staking_primitives::{
	offence::{Offence, OffenceDetails, OnOffenceHandler, ReportOffence},
	SessionIndex,
};
use srml_support::{
	decl_event, decl_module, decl_storage, ensure,
	traits::{Currency, Get, Imbalance, OnFreeBalanceZero, OnUnbalanced, Time},
};
use system::{ensure_root, ensure_signed};

use darwinia_support::{
	LockIdentifier, LockableCurrency, NormalLock, StakingLock, WithdrawLock, WithdrawReason, WithdrawReasons,
};
use phragmen::{build_support_map, elect, equalize, ExtendedBalance, PhragmenStakedAssignment};

pub type Balance = u128;
pub type Moment = u64;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

/// Counter for the number of "reward" points earned by a given validator.
pub type Points = u32;

const DEFAULT_MINIMUM_VALIDATOR_COUNT: u32 = 4;
const MAX_NOMINATIONS: usize = 16;
const MAX_UNLOCKING_CHUNKS: u32 = 32;
const MONTH_IN_MILLISECONDS: Moment = 30 * 24 * 60 * 60 * 1000;
const STAKING_ID: LockIdentifier = *b"staking ";

/// Reward points of an era. Used to split era total payout between validators.
#[derive(Encode, Decode, Default)]
pub struct EraPoints {
	/// Total number of points. Equals the sum of reward points for each validator.
	total: Points,
	/// The reward points earned by a given validator. The index of this vec corresponds to the
	/// index into the current validator set.
	individual: Vec<Points>,
}

impl EraPoints {
	/// Add the reward to the validator at the given index. Index must be valid
	/// (i.e. `index < current_elected.len()`).
	fn add_points_to_index(&mut self, index: u32, points: Points) {
		if let Some(new_total) = self.total.checked_add(points) {
			self.total = new_total;
			self.individual
				.resize((index as usize + 1).max(self.individual.len()), 0);
			self.individual[index as usize] += points; // Addition is less than total
		}
	}
}

/// Indicates the initial status of the staker.
#[derive(RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum StakerStatus<AccountId> {
	/// Chilling.
	Idle,
	/// Declared desire in validating or already participating in it.
	Validator,
	/// Nominating for a group of other stakers.
	Nominator(Vec<AccountId>),
}

/// A destination account for payment.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug)]
pub enum RewardDestination {
	/// Pay into the stash account, increasing the amount at stake accordingly.
	/// for now, we don't use this.
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

/// Preference of what happens on a slash event.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ValidatorPrefs {
	pub node_name: Vec<u8>,
	/// percent of Reward that validator takes up-front; only the rest is split between themselves and
	/// nominators.
	#[codec(compact)]
	pub validator_payment_ratio: u32,
}

impl ValidatorPrefs {
	// TODO
	/// Filter Ad, curse words...
	fn node_name_is_valid(&self) -> bool {
		if self.node_name.is_empty() {
			return false;
		}

		true
	}
}

impl Default for ValidatorPrefs {
	fn default() -> Self {
		ValidatorPrefs {
			node_name: vec![],
			validator_payment_ratio: 0,
		}
	}
}

/// To unify *Ring* and *Kton* balance.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum StakingBalance<Ring, Kton>
where
	Ring: HasCompact,
	Kton: HasCompact,
{
	Ring(Ring),
	Kton(Kton),
}

impl<Ring, Kton> Default for StakingBalance<Ring, Kton>
where
	Ring: Default + HasCompact,
	Kton: Default + HasCompact,
{
	fn default() -> Self {
		StakingBalance::Ring(Default::default())
	}
}

/// The *Ring* under deposit.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct TimeDepositItem<Ring: HasCompact, Moment> {
	#[codec(compact)]
	value: Ring,
	#[codec(compact)]
	start_time: Moment,
	#[codec(compact)]
	expire_time: Moment,
}

/// The ledger of a (bonded) stash.
#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingLedger<AccountId, Ring: HasCompact, Kton: HasCompact, Moment> {
	/// The stash account whose balance is actually locked and at stake.
	pub stash: AccountId,

	/// The total amount of the stash's balance that will be at stake in any forthcoming
	/// rounds.
	#[codec(compact)]
	pub active_ring: Ring,
	// active time-deposit ring
	#[codec(compact)]
	pub active_deposit_ring: Ring,

	/// The total amount of the stash's balance that will be at stake in any forthcoming
	/// rounds.
	#[codec(compact)]
	pub active_kton: Kton,
	// time-deposit items:
	// if you deposit ring for a minimum period,
	// you can get KTON as bonus
	// which can also be used for staking
	pub deposit_items: Vec<TimeDepositItem<Ring, Moment>>,

	pub ring_staking_lock: StakingLock<Ring, Moment>,
	pub kton_staking_lock: StakingLock<Kton, Moment>,
}

/// The amount of exposure (to slashing) than an individual nominator has.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct IndividualExposure<AccountId, Power: HasCompact> {
	/// The stash account of the nominator in question.
	who: AccountId,
	/// Amount of funds exposed.
	#[codec(compact)]
	value: Power,
}

/// A snapshot of the stake backing a single validator in the system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Exposure<AccountId, Power: HasCompact> {
	/// The total balance backing this validator.
	#[codec(compact)]
	pub total: Power,
	/// The validator's own stash that is exposed.
	#[codec(compact)]
	pub own: Power,
	/// The portions of nominators stashes that are exposed.
	pub others: Vec<IndividualExposure<AccountId, Power>>,
}

/// A slashing event occurred, slashing a validator for a given amount of balance.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct SlashJournalEntry<AccountId, Power: HasCompact> {
	who: AccountId,
	#[codec(compact)]
	amount: Power,
	// the amount of `who`'s own exposure that was slashed
	#[codec(compact)]
	own_slash: Power,
}

type RingBalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;
type RingPositiveImbalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type RingNegativeImbalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

type KtonBalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::Balance;
type KtonPositiveImbalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type KtonNegativeImbalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

type MomentOf<T> = <<T as Trait>::Time as Time>::Moment;

/// Means for interacting with a specialized version of the `session` trait.
///
/// This is needed because `Staking` sets the `ValidatorIdOf` of the `session::Trait`
pub trait SessionInterface<AccountId>: system::Trait {
	/// Disable a given validator by stash ID.
	///
	/// Returns `true` if new era should be forced at the end of this session.
	/// This allows preventing a situation where there is too many validators
	/// disabled and block production stalls.
	fn disable_validator(validator: &AccountId) -> Result<bool, ()>;
	/// Get the validators from session.
	fn validators() -> Vec<AccountId>;
	/// Prune historical session tries up to but not including the given index.
	fn prune_historical_up_to(up_to: SessionIndex);
}

impl<T: Trait> SessionInterface<<T as system::Trait>::AccountId> for T
where
	T: session::Trait<ValidatorId = <T as system::Trait>::AccountId>,
	T: session::historical::Trait<
		FullIdentification = Exposure<<T as system::Trait>::AccountId, ExtendedBalance>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: session::SessionHandler<<T as system::Trait>::AccountId>,
	T::OnSessionEnding: session::OnSessionEnding<<T as system::Trait>::AccountId>,
	T::SelectInitialValidators: session::SelectInitialValidators<<T as system::Trait>::AccountId>,
	T::ValidatorIdOf: Convert<<T as system::Trait>::AccountId, Option<<T as system::Trait>::AccountId>>,
{
	fn disable_validator(validator: &<T as system::Trait>::AccountId) -> Result<bool, ()> {
		<session::Module<T>>::disable(validator)
	}

	fn validators() -> Vec<<T as system::Trait>::AccountId> {
		<session::Module<T>>::validators()
	}

	fn prune_historical_up_to(up_to: SessionIndex) {
		<session::historical::Module<T>>::prune_up_to(up_to);
	}
}

pub trait Trait: timestamp::Trait + session::Trait {
	/// The staking balances.
	type Ring: LockableCurrency<Self::AccountId, Moment = Self::Moment>;
	/// The staking balances.
	type Kton: LockableCurrency<Self::AccountId, Moment = Self::Moment>;

	/// Time used for computing era duration.
	type Time: Time;

	/// Convert a balance into a number used for election calculation.
	/// This must fit into a `u64` but is allowed to be sensibly lossy.
	/// TODO: #1377
	/// The backward convert should be removed as the new Phragmen API returns ratio.
	/// The post-processing needs it but will be moved to off-chain. TODO: #2908
	type CurrencyToVote: Convert<ExtendedBalance, u64> + Convert<u128, ExtendedBalance>;

	/// Tokens have been minted and are unused for validator-reward.
	type RingRewardRemainder: OnUnbalanced<RingNegativeImbalanceOf<Self>>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// Handler for the unbalanced increment when rewarding a staker.
	type RingReward: OnUnbalanced<RingPositiveImbalanceOf<Self>>;
	/// Handler for the unbalanced increment when rewarding a staker.
	type KtonReward: OnUnbalanced<KtonPositiveImbalanceOf<Self>>;

	/// Handler for the unbalanced reduction when slashing a staker.
	type RingSlash: OnUnbalanced<RingNegativeImbalanceOf<Self>>;
	/// Handler for the unbalanced reduction when slashing a staker.
	type KtonSlash: OnUnbalanced<KtonNegativeImbalanceOf<Self>>;

	/// Number of sessions per era.
	type SessionsPerEra: Get<SessionIndex>;

	/// Number of seconds that staked funds must remain bonded for.
	type BondingDuration: Get<Self::Moment>;

	/// Interface for interacting with a session module.
	type SessionInterface: self::SessionInterface<Self::AccountId>;

	// custom
	type Cap: Get<<Self::Ring as Currency<Self::AccountId>>::Balance>;
	type GenesisTime: Get<MomentOf<Self>>;
}

/// Mode of era-forcing.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Forcing {
	/// Not forcing anything - just let whatever happen.
	NotForcing,
	/// Force a new era, then reset to `NotForcing` as soon as it is done.
	ForceNew,
	/// Avoid a new era indefinitely.
	ForceNone,
	/// Force a new era at the end of all sessions indefinitely.
	ForceAlways,
}

impl Default for Forcing {
	fn default() -> Self {
		Forcing::NotForcing
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Staking {
		/// The ideal number of staking participants.
		pub ValidatorCount get(fn validator_count) config(): u32;

		/// Minimum number of staking participants before emergency conditions are imposed.
		pub MinimumValidatorCount get(fn minimum_validator_count) config(): u32 = DEFAULT_MINIMUM_VALIDATOR_COUNT;

		/// Any validators that may never be slashed or forcibly kicked. It's a Vec since they're
		/// easy to initialize and the performance hit is minimal (we expect no more than four
		/// invulnerables) and restricted to testnets.
		pub Invulnerables get(fn invulnerables) config(): Vec<T::AccountId>;

		/// Map from all locked "stash" accounts to the controller account.
		pub Bonded get(fn bonded): map T::AccountId => Option<T::AccountId>;

		/// Map from all (unlocked) "controller" accounts to the info regarding the staking.
		pub Ledger get(fn ledger): map T::AccountId => Option<StakingLedger<T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>, T::Moment>>;

		/// Where the reward payment should be made. Keyed by stash.
		pub Payee get(fn payee): map T::AccountId => RewardDestination;

		/// The map from (wannabe) validator stash key to the preferences of that validator.
		pub Validators get(fn validators): linked_map T::AccountId => ValidatorPrefs;

		/// The map from nominator stash key to the set of stash keys of all validators to nominate.
		pub Nominators get(fn nominators): linked_map T::AccountId => Vec<T::AccountId>;

		/// Nominators for a particular account that is in action right now. You can't iterate
		/// through validators here, but you can find them in the Session module.
		///
		/// This is keyed by the stash account.
		pub Stakers get(fn stakers): map T::AccountId => Exposure<T::AccountId, ExtendedBalance>;

		/// The currently elected validator set keyed by stash account ID.
		pub CurrentElected get(fn current_elected): Vec<T::AccountId>;

		/// The current era index.
		pub CurrentEra get(fn current_era) config(): EraIndex;

		/// The start of the current era.
		pub CurrentEraStart get(fn current_era_start): MomentOf<T>;

		/// The session index at which the current era started.
		pub CurrentEraStartSessionIndex get(fn current_era_start_session_index): SessionIndex;

		/// Rewards for the current era. Using indices of current elected set.
		CurrentEraPointsEarned get(fn current_era_reward): EraPoints;

		/// The amount of balance actively at stake for each validator slot, currently.
		///
		/// This is used to derive rewards and punishments.
		pub SlotStake get(fn slot_stake) build(|config: &GenesisConfig<T>| {
			config.stakers.iter().map(|&(_, _, value, _)| value.saturated_into()).min().unwrap_or_default()
		}): ExtendedBalance;

		/// True if the next session change will be a new era regardless of index.
		pub ForceEra get(fn force_era) config(): Forcing;

		/// The percentage of the slash that is distributed to reporters.
		///
		/// The rest of the slashed value is handled by the `Slash`.
		pub SlashRewardFraction get(fn slash_reward_fraction) config(): Perbill;

		/// Total *Ring* in pool.
		pub RingPool get(fn ring_pool): RingBalanceOf<T>;
		/// Total *Kton* in pool.
		pub KtonPool get(fn kton_pool): KtonBalanceOf<T>;

		/// A mapping from still-bonded eras to the first session index of that era.
		BondedEras: Vec<(EraIndex, SessionIndex)>;

		/// All slashes that have occurred in a given era.
		EraSlashJournal get(fn era_slash_journal): map EraIndex => Vec<SlashJournalEntry<T::AccountId, ExtendedBalance>>;
	}

	add_extra_genesis {
		config(stakers): Vec<(T::AccountId, T::AccountId, RingBalanceOf<T>, StakerStatus<T::AccountId>)>;
		build(|config: &GenesisConfig<T>| {
			for &(ref stash, ref controller, ring, ref status) in &config.stakers {
				assert!(T::Ring::free_balance(&stash) >= ring);
				let _ = <Module<T>>::bond(
					T::Origin::from(Some(stash.clone()).into()),
					T::Lookup::unlookup(controller.clone()),
					StakingBalance::Ring(ring),
					RewardDestination::Stash,
					0,
				);
				let _ = match status {
					StakerStatus::Validator => {
						<Module<T>>::validate(
							T::Origin::from(Some(controller.clone()).into()),
							ValidatorPrefs {
								node_name: "Darwinia Node".bytes().collect(),
								..Default::default()
							},
						)
					},
					StakerStatus::Nominator(votes) => {
						<Module<T>>::nominate(
							T::Origin::from(Some(controller.clone()).into()),
							votes.iter().map(|l| {T::Lookup::unlookup(l.clone())}).collect(),
						)
					},
					_ => Ok(())
				};
			}
		});
	}
}

decl_event!(
    pub enum Event<T> 
    where
    	<T as system::Trait>::AccountId
    {
		/// All validators have been rewarded by the first balance; the second is the remainder
		/// from the maximum amount of reward.
		Reward(Balance, Balance),

		// TODO: refactor to Balance later?
		/// One validator (and its nominators) has been slashed by the given amount.
		Slash(AccountId, ExtendedBalance),
		/// An old slashing report from a prior era was discarded because it could
		/// not be processed.
		OldSlashingReportDiscarded(SessionIndex),

		/// NodeName changed.
	    NodeNameUpdated,
	    
	    /// Bond succeed.
	    /// `amount`, `now`, `duration` in month
	    Bond(Balance, Moment, Moment),
	    
	    /// Unbond succeed.
	    /// `amount`, `now`
		Unbond(Balance, Moment),
	    
	    // Develop
		//	    Print(u128),
    }
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Number of sessions per era.
		const SessionsPerEra: SessionIndex = T::SessionsPerEra::get();

		/// Number of eras that staked funds must remain bonded for.
		const BondingDuration: T::Moment = T::BondingDuration::get();

		fn deposit_event() = default;

		fn on_finalize() {
			// Set the start of the first era.
			if !<CurrentEraStart<T>>::exists() {
				<CurrentEraStart<T>>::put(T::Time::now());
			}
		}

		/// Take the origin account as a stash and lock up `value` of its balance. `controller` will
		/// be the account that controls it.
		///
		/// `value` must be more than the `minimum_balance` specified by `T::Currency`.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash account.
		///
		/// # <weight>
		/// - Independent of the arguments. Moderate complexity.
		/// - O(1).
		/// - Three extra DB entries.
		///
		/// NOTE: Two of the storage writes (`Self::bonded`, `Self::payee`) are _never_ cleaned unless
		/// the `origin` falls below _existential deposit_ and gets removed as dust.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn bond(
			origin,
			controller: <T::Lookup as StaticLookup>::Source,
			value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
			payee: RewardDestination,
			promise_month: Moment
		) {
			let stash = ensure_signed(origin)?;
			ensure!(!<Bonded<T>>::exists(&stash), err::STASH_ALREADY_BONDED);

			let controller = T::Lookup::lookup(controller)?;
			ensure!(!<Ledger<T>>::exists(&controller), err::CONTROLLER_ALREADY_PAIRED);

			// You're auto-bonded forever, here. We might improve this by only bonding when
			// you actually validate/nominate and remove once you unbond __everything__.
			<Bonded<T>>::insert(&stash, &controller);
			<Payee<T>>::insert(&stash, payee);

			let ledger = StakingLedger {
				stash: stash.clone(),
				..Default::default()
			};
			let bond_amount;
			let now = <timestamp::Module<T>>::now().saturated_into::<Moment>();
			let promise_month = promise_month.min(36);

			match value {
				StakingBalance::Ring(r) => {
					let stash_balance = T::Ring::free_balance(&stash);
					let value = r.min(stash_balance);
					bond_amount = value.saturated_into();

					Self::bond_helper_in_ring(&stash, &controller, value, promise_month, ledger);

					<RingPool<T>>::mutate(|r| *r += value);
				},
				StakingBalance::Kton(k) => {
					let stash_balance = T::Kton::free_balance(&stash);
					let value = k.min(stash_balance);
					bond_amount = value.saturated_into();

					Self::bond_helper_in_kton(&controller, value, ledger);

					<KtonPool<T>>::mutate(|k| *k += value);
				},
			}

			<Module<T>>::deposit_event(RawEvent::Bond(bond_amount, now, promise_month));
		}

		/// Add some extra amount that have appeared in the stash `free_balance` into the balance up
		/// for staking.
		///
		/// Use this if there are additional funds in your stash account that you wish to bond.
		/// Unlike [`bond`] or [`unbond`] this function does not impose any limitation on the amount
		/// that can be added.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - O(1).
		/// - One DB entry.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn bond_extra(
			origin,
			value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
			promise_month: Moment
		) {
			let stash = ensure_signed(origin)?;
			let controller = Self::bonded(&stash).ok_or(err::STASH_INVALID)?;
			let ledger = Self::ledger(&controller).ok_or(err::CONTROLLER_INVALID)?;
			let now = <timestamp::Module<T>>::now().saturated_into::<Moment>();
			let promise_month = promise_month.min(36);

			match value {
				 StakingBalance::Ring(r) => {
					let stash_balance = T::Ring::free_balance(&stash);
					if let Some(extra) = stash_balance.checked_sub(&ledger.active_ring) {
						let extra = extra.min(r);

						Self::bond_helper_in_ring(&stash, &controller, extra, promise_month, ledger);

						<RingPool<T>>::mutate(|r| *r += extra);
						<Module<T>>::deposit_event(RawEvent::Bond(extra.saturated_into(), now, promise_month));
					}
				},
				StakingBalance::Kton(k) => {
					let stash_balance = T::Kton::free_balance(&stash);
					if let Some(extra) = stash_balance.checked_sub(&ledger.active_kton) {
						let extra = extra.min(k);

						Self::bond_helper_in_kton(&controller, extra, ledger);

						<KtonPool<T>>::mutate(|k| *k += extra);
						<Module<T>>::deposit_event(RawEvent::Bond(extra.saturated_into(), now, promise_month));
					}
				},
			}
		}

		// TODO: doc
		fn deposit_extra(origin, value: RingBalanceOf<T>, promise_month: Moment) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(err::CONTROLLER_INVALID)?;
			let promise_month = promise_month.max(3).min(36);

			let now = <timestamp::Module<T>>::now();
			let mut ledger = Self::clear_mature_deposits(ledger);
			let StakingLedger {
				stash,
				active_ring,
				active_deposit_ring,
				deposit_items,
				..
			} = &mut ledger;
			let value = value.min(*active_ring - *active_deposit_ring);
			// for now, kton_return is free
			// mint kton
			let kton_return = inflation::compute_kton_return::<T>(value, promise_month);
			let kton_positive_imbalance = T::Kton::deposit_creating(stash, kton_return);

			T::KtonReward::on_unbalanced(kton_positive_imbalance);
			*active_deposit_ring += value;
			deposit_items.push(TimeDepositItem {
				value,
				start_time: now,
				expire_time: now + T::Moment::saturated_from((promise_month * MONTH_IN_MILLISECONDS).into()),
			});

			<Ledger<T>>::insert(&controller, ledger);
			<Module<T>>::deposit_event(RawEvent::Bond(value.saturated_into(), now.saturated_into::<Moment>(), promise_month));
		}

		/// for normal_ring or normal_kton, follow the original substrate pattern
		/// for time_deposit_ring, transform it into normal_ring first
		/// modify time_deposit_items and time_deposit_ring amount

		/// Schedule a portion of the stash to be unlocked ready for transfer out after the bond
		/// period ends. If this leaves an amount actively bonded less than
		/// T::Currency::minimum_balance(), then it is increased to the full amount.
		///
		/// Once the unlock period is done, the funds will be withdrew automatically and ready for transfer.
		///
		/// No more than a limited number of unlocking chunks (see `MAX_UNLOCKING_CHUNKS`)
		/// can co-exists at the same time. In that case,  [`StakingLock::shrink`] need
		/// to be called first to remove some of the chunks (if possible).
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// # <weight>
		/// - Independent of the arguments. Limited but potentially exploitable complexity.
		/// - Contains a limited number of reads.
		/// - Each call (requires the remainder of the bonded balance to be above `minimum_balance`)
		///   will cause a new entry to be inserted into a vector (`StakingLock.unbondings`) kept in storage.
		/// - One DB entry.
		/// </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(400_000)]
		fn unbond(origin, value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>) {
			let controller = ensure_signed(origin)?;
			let mut ledger = Self::clear_mature_deposits(Self::ledger(&controller).ok_or(err::CONTROLLER_INVALID)?);
			let StakingLedger {
				active_ring,
				active_deposit_ring,
				active_kton,
				ring_staking_lock,
				kton_staking_lock,
				..
			} = &mut ledger;
			let now = <timestamp::Module<T>>::now();

			ring_staking_lock.shrink(now);
			kton_staking_lock.shrink(now);

			// due to the macro parser, we've to add a bracket
			// actually, this's totally wrong:
			//     `a as u32 + b as u32 < c`
			// workaround:
			//     1. `(a as u32 + b as u32) < c`
			//     2. `let c_ = a as u32 + b as u32; c_ < c`
			ensure!(
				(ring_staking_lock.unbondings.len() as u32 + kton_staking_lock.unbondings.len() as u32) < MAX_UNLOCKING_CHUNKS,
				err::UNLOCK_CHUNKS_REACH_MAX,
			);

			match value {
				StakingBalance::Ring(r) => {
					// only active normal ring can be unbond
					// active_ring = active_normal_ring + active_deposit_ring
					let active_normal_ring = *active_ring - *active_deposit_ring;
					let available_unbond_ring = r.min(active_normal_ring);

					if !available_unbond_ring.is_zero() {
						*active_ring -= available_unbond_ring;
						ring_staking_lock.unbondings.push(NormalLock {
							amount: available_unbond_ring,
							until: now + T::BondingDuration::get(),
						});

						Self::update_ledger(&controller, &mut ledger, value);

						<RingPool<T>>::mutate(|r| *r -= available_unbond_ring);
						<Module<T>>::deposit_event(RawEvent::Unbond(available_unbond_ring.saturated_into(), now.saturated_into::<Moment>()));
					}
				},
				StakingBalance::Kton(k) => {
					let unbond_kton = k.min(*active_kton);

					if !unbond_kton.is_zero() {
						*active_kton -= unbond_kton;
						kton_staking_lock.unbondings.push(NormalLock {
							amount: unbond_kton,
							until: now + T::BondingDuration::get(),
						});

						Self::update_ledger(&controller, &mut ledger, value);

						<KtonPool<T>>::mutate(|k| *k -= unbond_kton);
						<Module<T>>::deposit_event(RawEvent::Unbond(unbond_kton.saturated_into(), now.saturated_into::<Moment>()));
					}
				},
			}
		}

		// TODO: doc
		fn claim_mature_deposits(origin) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::clear_mature_deposits(Self::ledger(&controller).ok_or(err::CONTROLLER_INVALID)?);

			<Ledger<T>>::insert(controller, ledger);
		}

		// TODO: doc
		fn try_claim_deposits_with_punish(origin, expire_time: T::Moment) {
			let controller = ensure_signed(origin)?;
			let mut ledger = Self::ledger(&controller).ok_or(err::CONTROLLER_INVALID)?;
			let now = <timestamp::Module<T>>::now();

			ensure!(expire_time > now, err::CLAIM_DEPOSITS_EXPIRE_TIME_INVALID);

			let StakingLedger {
				stash,
				active_deposit_ring,
				deposit_items,
				..
			} = &mut ledger;

			deposit_items.retain(|item| {
				if item.expire_time != expire_time {
					return true;
				}

				let kton_slash = {
					let passed_duration = (now - item.start_time).saturated_into::<Moment>() / MONTH_IN_MILLISECONDS;
					let plan_duration = (item.expire_time - item.start_time).saturated_into::<Moment>() / MONTH_IN_MILLISECONDS;

					(
						inflation::compute_kton_return::<T>(item.value, plan_duration)
						-
						inflation::compute_kton_return::<T>(item.value, passed_duration)
					).max(1.into()) * 3.into()
				};

				// check total free balance and locked one
				// strict on punishing in kton
				if T::Kton::free_balance(stash)
					.checked_sub(&kton_slash)
					.and_then(|new_balance| {
						T::Kton::ensure_can_withdraw(
							stash,
							kton_slash,
							WithdrawReason::Transfer.into(),
							new_balance
						).ok()
					})
					.is_some()
				{
					*active_deposit_ring = active_deposit_ring.saturating_sub(item.value);

					let (imbalance, _) = T::Kton::slash(stash, kton_slash);
					T::KtonSlash::on_unbalanced(imbalance);

					false
				} else {
					true
				}
			});

			<Ledger<T>>::insert(&controller, ledger);
		}

		/// Declare the desire to validate for the origin controller.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains a limited number of reads.
		/// - Writes are limited to the `origin` account key.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(750_000)]
		fn validate(origin, prefs: ValidatorPrefs) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(err::CONTROLLER_INVALID)?;

			ensure!(prefs.node_name_is_valid(), err::NODE_NAME_INVALID);

			let stash = &ledger.stash;
			let mut prefs = prefs;
			// at most 100%
			prefs.validator_payment_ratio = prefs.validator_payment_ratio.min(100);

			<Nominators<T>>::remove(stash);
			<Validators<T>>::mutate(stash, |prefs_| {
				let exists = !prefs_.node_name.is_empty();
				*prefs_ = prefs;
				if exists {
					Self::deposit_event(RawEvent::NodeNameUpdated);
				}
			});
		}

		/// Declare the desire to nominate `targets` for the origin controller.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// # <weight>
		/// - The transaction's complexity is proportional to the size of `targets`,
		/// which is capped at `MAX_NOMINATIONS`.
		/// - Both the reads and writes follow a similar pattern.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(750_000)]
		fn nominate(origin, targets: Vec<<T::Lookup as StaticLookup>::Source>) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(err::CONTROLLER_INVALID)?;
			let stash = &ledger.stash;

			ensure!(!targets.is_empty(), err::TARGETS_INVALID);

			let targets = targets.into_iter()
				.take(MAX_NOMINATIONS)
				.map(T::Lookup::lookup)
				.collect::<result::Result<Vec<T::AccountId>, _>>()?;

			<Validators<T>>::remove(stash);
			<Nominators<T>>::insert(stash, targets);
		}

		/// Declare no desire to either validate or nominate.
		///
		/// Effects will be felt at the beginning of the next era.、
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains one read.
		/// - Writes are limited to the `origin` account key.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn chill(origin) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(err::CONTROLLER_INVALID)?;
			let stash = &ledger.stash;

			<Validators<T>>::remove(stash);
			<Nominators<T>>::remove(stash);
		}

		/// (Re-)set the payment target for a controller.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains a limited number of reads.
		/// - Writes are limited to the `origin` account key.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn set_payee(origin, payee: RewardDestination) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(err::CONTROLLER_INVALID)?;
			let stash = &ledger.stash;

			<Payee<T>>::insert(stash, payee);
		}

		/// (Re-)set the controller of a stash.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains a limited number of reads.
		/// - Writes are limited to the `origin` account key.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(750_000)]
		fn set_controller(origin, controller: <T::Lookup as StaticLookup>::Source) {
			let stash = ensure_signed(origin)?;
			let old_controller = Self::bonded(&stash).ok_or(err::STASH_INVALID)?;
			let controller = T::Lookup::lookup(controller)?;

			ensure!(!<Ledger<T>>::exists(&controller), err::CONTROLLER_ALREADY_PAIRED);

			if controller != old_controller {
				<Bonded<T>>::insert(&stash, &controller);
				if let Some(l) = <Ledger<T>>::take(&old_controller) {
					<Ledger<T>>::insert(&controller, l);
				}
			}
		}

		/// The ideal number of validators.
		fn set_validator_count(origin, #[compact] new: u32) {
			ensure_root(origin)?;
			ValidatorCount::put(new);
		}

		// ----- Root calls.

		/// Force there to be no new eras indefinitely.
		///
		/// # <weight>
		/// - No arguments.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FreeOperational]
		fn force_no_eras(origin) {
			ensure_root(origin)?;
			ForceEra::put(Forcing::ForceNone);
		}

		/// Force there to be a new era at the end of the next session. After this, it will be
		/// reset to normal (non-forced) behaviour.
		///
		/// # <weight>
		/// - No arguments.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FreeOperational]
		fn force_new_era(origin) {
			ensure_root(origin)?;
			ForceEra::put(Forcing::ForceNone);
		}

		/// Set the validators who cannot be slashed (if any).
		fn set_invulnerables(origin, validators: Vec<T::AccountId>) {
			ensure_root(origin)?;
			<Invulnerables<T>>::put(validators);
		}

		/// Force a current staker to become completely unstaked, immediately.
		#[weight = SimpleDispatchInfo::FreeOperational]
		fn force_unstake(origin, stash: T::AccountId) {
			ensure_root(origin)?;

			// remove the lock.
			T::Ring::remove_lock(STAKING_ID, &stash);
			T::Kton::remove_lock(STAKING_ID, &stash);
			// remove all staking-related information.
			Self::kill_stash(&stash);
		}

		/// Force there to be a new era at the end of sessions indefinitely.
		///
		/// # <weight>
		/// - One storage write
		/// # </weight>
		#[weight = SimpleDispatchInfo::FreeOperational]
		fn force_new_era_always(origin) {
			ensure_root(origin)?;
			ForceEra::put(Forcing::ForceAlways);
		}
	}
}

impl<T: Trait> Module<T> {
	// PUBLIC IMMUTABLES

	// TODO: doc
	pub fn clear_mature_deposits(
		mut ledger: StakingLedger<T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>, T::Moment>,
	) -> StakingLedger<T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>, T::Moment> {
		let now = <timestamp::Module<T>>::now();
		let StakingLedger {
			active_deposit_ring,
			deposit_items,
			..
		} = &mut ledger;

		deposit_items.retain(|item| {
			if item.expire_time > now {
				true
			} else {
				*active_deposit_ring = active_deposit_ring.saturating_sub(item.value);
				false
			}
		});

		ledger
	}

	// TODO: doc
	fn bond_helper_in_ring(
		stash: &T::AccountId,
		controller: &T::AccountId,
		value: RingBalanceOf<T>,
		promise_month: Moment,
		mut ledger: StakingLedger<T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>, T::Moment>,
	) {
		// if stash promise to a extra-lock
		// there will be extra reward, kton, which
		// can also be use to stake.
		if promise_month >= 3 {
			ledger.active_deposit_ring += value;
			// for now, kton_return is free
			// mint kton
			let kton_return = inflation::compute_kton_return::<T>(value, promise_month);
			let kton_positive_imbalance = T::Kton::deposit_creating(&stash, kton_return);
			T::KtonReward::on_unbalanced(kton_positive_imbalance);
			let now = <timestamp::Module<T>>::now();
			ledger.deposit_items.push(TimeDepositItem {
				value,
				start_time: now,
				expire_time: now + T::Moment::saturated_from((promise_month * MONTH_IN_MILLISECONDS).into()),
			});
		}
		ledger.active_ring = ledger.active_ring.saturating_add(value);

		Self::update_ledger(&controller, &mut ledger, StakingBalance::Ring(value));
	}

	// TODO: doc
	fn bond_helper_in_kton(
		controller: &T::AccountId,
		value: KtonBalanceOf<T>,
		mut ledger: StakingLedger<T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>, T::Moment>,
	) {
		ledger.active_kton += value;

		Self::update_ledger(&controller, &mut ledger, StakingBalance::Kton(value));
	}

	// TODO: there is reserve balance in Balance.Slash, we assuming it is zero for now.
	fn slash_individual(
		stash: &T::AccountId,
		slash_ratio: Perbill,
	) -> (RingNegativeImbalanceOf<T>, KtonNegativeImbalanceOf<T>, ExtendedBalance) {
		let controller = Self::bonded(stash).unwrap();
		let mut ledger = Self::ledger(&controller).unwrap();

		let (ring_imbalance, _) = if !ledger.active_ring.is_zero() {
			let slashable_ring = slash_ratio * ledger.active_ring;
			let value_slashed = Self::slash_helper(&controller, &mut ledger, StakingBalance::Ring(slashable_ring));
			T::Ring::slash(stash, value_slashed.0)
		} else {
			(<RingNegativeImbalanceOf<T>>::zero(), Zero::zero())
		};
		let (kton_imbalance, _) = if !ledger.active_kton.is_zero() {
			let slashable_kton = slash_ratio * ledger.active_kton;
			let value_slashed = Self::slash_helper(&controller, &mut ledger, StakingBalance::Kton(slashable_kton));
			T::Kton::slash(stash, value_slashed.1)
		} else {
			(<KtonNegativeImbalanceOf<T>>::zero(), Zero::zero())
		};

		(ring_imbalance, kton_imbalance, 0)
	}

	// TODO: doc
	fn power_of(stash: &T::AccountId) -> ExtendedBalance {
		// power is a mixture of ring and kton
		// power = ring_ratio * POWER_COUNT / 2 + kton_ratio * POWER_COUNT / 2
		fn calc_power<S: rstd::convert::TryInto<u128>>(active: S, pool: S) -> ExtendedBalance {
			const HALF_POWER_COUNT: u128 = 1_000_000_000 / 2;

			Perquintill::from_rational_approximation(
				active.saturated_into::<ExtendedBalance>(),
				pool.saturated_into::<ExtendedBalance>().max(1),
			) * HALF_POWER_COUNT
		}

		Self::bonded(stash)
			.and_then(Self::ledger)
			.map(|l| calc_power(l.active_ring, Self::ring_pool()) + calc_power(l.active_kton, Self::kton_pool()))
			.unwrap_or_default()
	}

	// MUTABLES (DANGEROUS)

	/// Update the ledger for a controller. This will also update the stash lock. The lock will
	/// will lock the entire funds except paying for further transactions.
	fn update_ledger(
		controller: &T::AccountId,
		ledger: &mut StakingLedger<T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>, T::Moment>,
		staking_balance: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
	) {
		match staking_balance {
			StakingBalance::Ring(_r) => {
				ledger.ring_staking_lock.staking_amount = ledger.active_ring;

				T::Ring::set_lock(
					STAKING_ID,
					&ledger.stash,
					WithdrawLock::WithStaking(ledger.ring_staking_lock.clone()),
					WithdrawReasons::all(),
				);
			}
			StakingBalance::Kton(_k) => {
				ledger.kton_staking_lock.staking_amount = ledger.active_kton;

				T::Kton::set_lock(
					STAKING_ID,
					&ledger.stash,
					WithdrawLock::WithStaking(ledger.kton_staking_lock.clone()),
					WithdrawReasons::all(),
				);
			}
		}

		<Ledger<T>>::insert(controller, ledger);
	}

	/// Slash a given validator by a specific amount with given (historical) exposure.
	///
	/// Removes the slash from the validator's balance by preference,
	/// and reduces the nominators' balance if needed.
	///
	/// Returns the resulting `NegativeImbalance` to allow distributing the slashed amount and
	/// pushes an entry onto the slash journal.
	fn slash_validator(
		stash: &T::AccountId,
		slash: ExtendedBalance,
		exposure: &Exposure<T::AccountId, ExtendedBalance>,
		journal: &mut Vec<SlashJournalEntry<T::AccountId, ExtendedBalance>>,
	) -> (RingNegativeImbalanceOf<T>, KtonNegativeImbalanceOf<T>) {
		// The amount we are actually going to slash (can't be bigger than the validator's total
		// exposure)
		let slash = slash.min(exposure.total);

		// limit what we'll slash of the stash's own to only what's in
		// the exposure.
		//
		// note: this is fine only because we limit reports of the current era.
		// otherwise, these funds may have already been slashed due to something
		// reported from a prior era.
		let already_slashed_own = journal
			.iter()
			.filter(|entry| &entry.who == stash)
			.map(|entry| entry.own_slash)
			.fold(ExtendedBalance::zero(), |a, c| a.saturating_add(c));

		let own_remaining = exposure.own.saturating_sub(already_slashed_own);

		// The amount we'll slash from the validator's stash directly.
		let own_slash = own_remaining.min(slash);
		let (mut ring_imbalance, mut kton_imbalance, missing) =
			Self::slash_individual(stash, Perbill::from_rational_approximation(own_slash, exposure.own));
		let own_slash = own_slash - missing;
		// The amount remaining that we can't slash from the validator,
		// that must be taken from the nominators.
		let rest_slash = slash - own_slash;
		if !rest_slash.is_zero() {
			// The total to be slashed from the nominators.
			let total = exposure.total - exposure.own;
			if !total.is_zero() {
				for i in exposure.others.iter() {
					let per_u64 = Perbill::from_rational_approximation(i.value, total);
					// best effort - not much that can be done on fail.
					// imbalance.subsume(T::Currency::slash(&i.who, per_u64 * rest_slash).0)
					let (r, k, _) = Self::slash_individual(
						&i.who,
						Perbill::from_rational_approximation(per_u64 * rest_slash, i.value),
					);

					ring_imbalance.subsume(r);
					kton_imbalance.subsume(k);
				}
			}
		}

		journal.push(SlashJournalEntry {
			who: stash.to_owned(),
			own_slash,
			amount: slash,
		});

		// trigger the event
		Self::deposit_event(RawEvent::Slash(stash.to_owned(), slash));

		(ring_imbalance, kton_imbalance)
	}

	// TODO: doc
	fn slash_helper(
		controller: &T::AccountId,
		ledger: &mut StakingLedger<T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>, T::Moment>,
		value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
	) -> (RingBalanceOf<T>, KtonBalanceOf<T>) {
		match value {
			StakingBalance::Ring(r) => {
				let StakingLedger {
					active_ring,
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

				// bonded + unbondings
				// first slash active normal ring
				let mut value_left = total_value - normal_active_value;
				// then slash active time-promise ring
				// from the nearest expire time
				if !value_left.is_zero() {
					// sorted by expire_time from far to near
					deposit_items.sort_unstable_by_key(|item| T::Moment::max_value() - item.expire_time);
					deposit_items.drain_filter(|item| {
						if value_left.is_zero() {
							return false;
						}

						let value_removed = value_left.min(item.value);

						*active_ring -= value_removed;
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

				<KtonPool<T>>::mutate(|k| *k -= active_value);

				Self::update_ledger(controller, ledger, StakingBalance::Kton(0.into()));
				(0.into(), active_value)
			}
		}
	}

	/// Actually make a payment to a staker. This uses the currency's reward function
	/// to pay the right payee for the given staker account.
	fn make_payout(stash: &T::AccountId, amount: RingBalanceOf<T>) -> Option<RingPositiveImbalanceOf<T>> {
		let dest = Self::payee(stash);
		match dest {
			RewardDestination::Controller => {
				Self::bonded(stash).and_then(|controller| T::Ring::deposit_into_existing(&controller, amount).ok())
			}
			RewardDestination::Stash => T::Ring::deposit_into_existing(stash, amount).ok(),
		}
	}

	/// Reward a given validator by a specific amount. Add the reward to the validator's, and its
	/// nominators' balance, pro-rata based on their exposure, after having removed the validator's
	/// pre-payout cut.
	fn reward_validator(stash: &T::AccountId, reward: RingBalanceOf<T>) -> RingPositiveImbalanceOf<T> {
		let off_the_table = Perbill::from_percent(Self::validators(stash).validator_payment_ratio) * reward;
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

		imbalance
	}

	/// Session has just ended. Provide the validator set for the next session if it's an era-end, along
	/// with the exposure of the prior validator set.
	fn new_session(
		session_index: SessionIndex,
	) -> Option<(
		Vec<T::AccountId>,
		Vec<(T::AccountId, Exposure<T::AccountId, ExtendedBalance>)>,
	)> {
		let era_length = session_index
			.checked_sub(Self::current_era_start_session_index())
			.unwrap_or(0);
		match ForceEra::get() {
			Forcing::ForceNew => ForceEra::kill(),
			Forcing::ForceAlways => (),
			Forcing::NotForcing if era_length >= T::SessionsPerEra::get() => (),
			_ => return None,
		}
		let validators = T::SessionInterface::validators();
		let prior = validators
			.into_iter()
			.map(|v| {
				let e = Self::stakers(&v);
				(v, e)
			})
			.collect();

		Self::new_era(session_index).map(move |new| (new, prior))
	}

	/// The era has changed - enact new staking set.
	///
	/// NOTE: This always happens immediately before a session change to ensure that new validators
	/// get a chance to set their session keys.
	fn new_era(start_session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		// Payout
		let points = CurrentEraPointsEarned::take();
		let now = T::Time::now();
		let previous_era_start = <CurrentEraStart<T>>::mutate(|v| rstd::mem::replace(v, now));
		let era_duration = now - previous_era_start;
		if !era_duration.is_zero() {
			let validators = Self::current_elected();

			// TODO: All reward will give to payouts.
			//			let validator_len: ExtendedBalance = (validators.len() as u32).into();
			//			let total_rewarded_stake = Self::slot_stake() * validator_len;

			//			Self::deposit_event(RawEvent::Print(era_duration.saturated_into::<u128>()));
			//			Self::deposit_event(RawEvent::Print((T::Time::now() - T::GenesisTime::get()).saturated_into::<u128>()));
			//			Self::deposit_event(RawEvent::Print((T::Cap::get() - T::Ring::total_issuance()).saturated_into::<u128>()));

			let (total_payout, max_payout) = inflation::compute_total_payout::<T>(
				era_duration.saturated_into::<Moment>(),
				(T::Time::now() - T::GenesisTime::get()).saturated_into::<Moment>(),
				(T::Cap::get() - T::Ring::total_issuance()).saturated_into::<Balance>(),
			);

			let mut total_imbalance = <RingPositiveImbalanceOf<T>>::zero();
			for (v, p) in validators.iter().zip(points.individual.into_iter()) {
				if p != 0 {
					let reward = Perbill::from_rational_approximation(p, points.total) * total_payout;
					total_imbalance.subsume(Self::reward_validator(v, reward));
				}
			}

			//			assert!(total_imbalance.peek() == total_payout);
			let total_payout = total_imbalance.peek();

			let rest = max_payout.saturating_sub(total_payout);
			Self::deposit_event(RawEvent::Reward(total_payout.saturated_into(), rest.saturated_into()));

			T::RingReward::on_unbalanced(total_imbalance);
			T::RingRewardRemainder::on_unbalanced(T::Ring::issue(rest));
		}

		// Increment current era.
		let current_era = CurrentEra::mutate(|s| {
			*s += 1;
			*s
		});

		// prune journal for last era.
		<EraSlashJournal<T>>::remove(current_era - 1);

		CurrentEraStartSessionIndex::mutate(|v| {
			*v = start_session_index;
		});
		let bonding_era = {
			const BONDING_DURATION_ERA_TO_SECS_RATIO: Moment = 300;
			(T::BondingDuration::get().saturated_into::<Moment>() / BONDING_DURATION_ERA_TO_SECS_RATIO) as EraIndex
		};

		if current_era > bonding_era {
			let first_kept = current_era - bonding_era;
			BondedEras::mutate(|bonded| {
				bonded.push((current_era, start_session_index));

				// prune out everything that's from before the first-kept index.
				let n_to_prune = bonded.iter().take_while(|&&(era_idx, _)| era_idx < first_kept).count();

				bonded.drain(..n_to_prune);

				if let Some(&(_, first_session)) = bonded.first() {
					T::SessionInterface::prune_historical_up_to(first_session);
				}
			})
		}

		// Reassign all Stakers.
		let (_slot_stake, maybe_new_validators) = Self::select_validators();

		maybe_new_validators
	}

	/// Select a new validator set from the assembled stakers and their role preferences.
	///
	/// Returns the new `SlotStake` value.
	fn select_validators() -> (ExtendedBalance, Option<Vec<T::AccountId>>) {
		let mut all_nominators: Vec<(T::AccountId, Vec<T::AccountId>)> = Vec::new();
		let all_validator_candidates_iter = <Validators<T>>::enumerate();
		let all_validators = all_validator_candidates_iter
			.map(|(who, _pref)| {
				let self_vote = (who.clone(), vec![who.clone()]);
				all_nominators.push(self_vote);
				who
			})
			.collect::<Vec<T::AccountId>>();
		all_nominators.extend(<Nominators<T>>::enumerate());

		let maybe_phragmen_result = elect::<_, _, _, T::CurrencyToVote>(
			Self::validator_count() as usize,
			Self::minimum_validator_count().max(1) as usize,
			all_validators,
			all_nominators,
			Self::power_of,
		);

		if let Some(phragmen_result) = maybe_phragmen_result {
			let elected_stashes = phragmen_result
				.winners
				.iter()
				.map(|(s, _)| s.clone())
				.collect::<Vec<T::AccountId>>();
			let assignments = phragmen_result.assignments;

			let to_votes = |b: ExtendedBalance| {
				<T::CurrencyToVote as Convert<ExtendedBalance, u64>>::convert(b) as ExtendedBalance
			};
			let to_balance =
				|e: ExtendedBalance| <T::CurrencyToVote as Convert<ExtendedBalance, ExtendedBalance>>::convert(e);

			let mut supports =
				build_support_map::<_, _, _, T::CurrencyToVote>(&elected_stashes, &assignments, Self::power_of);

			if cfg!(feature = "equalize") {
				let mut staked_assignments: Vec<(T::AccountId, Vec<PhragmenStakedAssignment<T::AccountId>>)> =
					Vec::with_capacity(assignments.len());
				for (n, assignment) in assignments.iter() {
					let mut staked_assignment: Vec<PhragmenStakedAssignment<T::AccountId>> =
						Vec::with_capacity(assignment.len());

					// If this is a self vote, then we don't need to equalise it at all. While the
					// staking system does not allow nomination and validation at the same time,
					// this must always be 100% support.
					if assignment.len() == 1 && assignment[0].0 == *n {
						continue;
					}
					for (c, per_thing) in assignment.iter() {
						let nominator_stake = to_votes(Self::power_of(n));
						let other_stake = *per_thing * nominator_stake;
						staked_assignment.push((c.clone(), other_stake));
					}
					staked_assignments.push((n.clone(), staked_assignment));
				}

				let tolerance = 0_u128;
				let iterations = 2_usize;
				equalize::<_, _, T::CurrencyToVote, _>(
					staked_assignments,
					&mut supports,
					tolerance,
					iterations,
					Self::power_of,
				);
			}

			// Clear Stakers.
			for v in Self::current_elected().iter() {
				<Stakers<T>>::remove(v);
			}

			// Populate Stakers and figure out the minimum stake behind a slot.
			let mut slot_stake = ExtendedBalance::max_value();
			for (c, s) in supports.into_iter() {
				// build `struct exposure` from `support`
				let exposure = Exposure {
					own: to_balance(s.own),
					// This might reasonably saturate and we cannot do much about it. The sum of
					// someone's stake might exceed the balance type if they have the maximum amount
					// of balance and receive some support. This is super unlikely to happen, yet
					// we simulate it in some tests.
					total: to_balance(s.total),
					others: s
						.others
						.into_iter()
						.map(|(who, value)| IndividualExposure {
							who,
							value: to_balance(value),
						})
						.collect::<Vec<IndividualExposure<_, _>>>(),
				};
				slot_stake = slot_stake.min(exposure.total);

				<Stakers<T>>::insert(&c, exposure);
			}

			// Update slot stake.
			<SlotStake>::put(&slot_stake);

			// Set the new validator set in sessions.
			<CurrentElected<T>>::put(&elected_stashes);

			// In order to keep the property required by `n_session_ending`
			// that we must return the new validator set even if it's the same as the old,
			// as long as any underlying economic conditions have changed, we don't attempt
			// to do any optimization where we compare against the prior set.
			(slot_stake, Some(elected_stashes))
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

	/// Remove all associated data of a stash account from the staking system.
	///
	/// This is called:
	/// - Immediately when an account's balance falls below existential deposit.
	/// - after a `withdraw_unbond()` call that frees all of a stash's bonded balance.
	fn kill_stash(stash: &T::AccountId) {
		if let Some(controller) = <Bonded<T>>::take(stash) {
			<Ledger<T>>::remove(&controller);
		}
		<Payee<T>>::remove(stash);
		<Validators<T>>::remove(stash);
		<Nominators<T>>::remove(stash);
	}

	/// Add reward points to validators using their stash account ID.
	///
	/// Validators are keyed by stash account ID and must be in the current elected set.
	///
	/// For each element in the iterator the given number of points in u32 is added to the
	/// validator, thus duplicates are handled.
	///
	/// At the end of the era each the total payout will be distributed among validator
	/// relatively to their points.
	///
	/// COMPLEXITY: Complexity is `number_of_validator_to_reward x current_elected_len`.
	/// If you need to reward lots of validator consider using `reward_by_indices`.
	pub fn reward_by_ids(validators_points: impl IntoIterator<Item = (T::AccountId, Points)>) {
		CurrentEraPointsEarned::mutate(|rewards| {
			let current_elected = <Module<T>>::current_elected();
			for (validator, points) in validators_points.into_iter() {
				if let Some(index) = current_elected.iter().position(|elected| *elected == validator) {
					rewards.add_points_to_index(index as u32, points);
				}
			}
		});
	}

	/// Add reward points to validators using their validator index.
	///
	/// For each element in the iterator the given number of points in u32 is added to the
	/// validator, thus duplicates are handled.
	pub fn reward_by_indices(validators_points: impl IntoIterator<Item = (u32, Points)>) {
		// TODO: This can be optimised once #3302 is implemented.
		let current_elected_len = <Module<T>>::current_elected().len() as u32;

		CurrentEraPointsEarned::mutate(|rewards| {
			for (validator_index, points) in validators_points.into_iter() {
				if validator_index < current_elected_len {
					rewards.add_points_to_index(validator_index, points);
				}
			}
		});
	}

	/// Ensures that at the end of the current session there will be a new era.
	fn ensure_new_era() {
		match ForceEra::get() {
			Forcing::ForceAlways | Forcing::ForceNew => (),
			_ => ForceEra::put(Forcing::ForceNew),
		}
	}
}

impl<T: Trait> session::OnSessionEnding<T::AccountId> for Module<T> {
	fn on_session_ending(_ending: SessionIndex, start_session: SessionIndex) -> Option<Vec<T::AccountId>> {
		Self::new_session(start_session - 1).map(|(new, _old)| new)
	}
}

impl<T: Trait> OnSessionEnding<T::AccountId, Exposure<T::AccountId, ExtendedBalance>> for Module<T> {
	fn on_session_ending(
		_ending: SessionIndex,
		start_session: SessionIndex,
	) -> Option<(
		Vec<T::AccountId>,
		Vec<(T::AccountId, Exposure<T::AccountId, ExtendedBalance>)>,
	)> {
		Self::new_session(start_session - 1)
	}
}

impl<T: Trait> OnFreeBalanceZero<T::AccountId> for Module<T> {
	fn on_free_balance_zero(stash: &T::AccountId) {
		Self::kill_stash(stash);
	}
}

/// Add reward points to block authors:
/// * 20 points to the block producer for producing a (non-uncle) block in the relay chain,
/// * 2 points to the block producer for each reference to a previously unreferenced uncle, and
/// * 1 point to the producer of each referenced uncle block.
impl<T: Trait + authorship::Trait> authorship::EventHandler<T::AccountId, T::BlockNumber> for Module<T> {
	fn note_author(author: T::AccountId) {
		Self::reward_by_ids(vec![(author, 20)]);
	}
	fn note_uncle(author: T::AccountId, _age: T::BlockNumber) {
		Self::reward_by_ids(vec![(<authorship::Module<T>>::author(), 2), (author, 1)])
	}
}

pub struct StashOf<T>(rstd::marker::PhantomData<T>);

impl<T: Trait> Convert<T::AccountId, Option<T::AccountId>> for StashOf<T> {
	fn convert(controller: T::AccountId) -> Option<T::AccountId> {
		<Module<T>>::ledger(&controller).map(|l| l.stash)
	}
}

/// A typed conversion from stash account ID to the current exposure of nominators
/// on that account.
pub struct ExposureOf<T>(rstd::marker::PhantomData<T>);

impl<T: Trait> Convert<T::AccountId, Option<Exposure<T::AccountId, ExtendedBalance>>> for ExposureOf<T> {
	fn convert(validator: T::AccountId) -> Option<Exposure<T::AccountId, ExtendedBalance>> {
		Some(<Module<T>>::stakers(&validator))
	}
}

impl<T: Trait> SelectInitialValidators<T::AccountId> for Module<T> {
	fn select_initial_validators() -> Option<Vec<T::AccountId>> {
		<Module<T>>::select_validators().1
	}
}

/// This is intended to be used with `FilterHistoricalOffences`.
impl<T: Trait> OnOffenceHandler<T::AccountId, session::historical::IdentificationTuple<T>> for Module<T>
where
	T: session::Trait<ValidatorId = <T as system::Trait>::AccountId>,
	T: session::historical::Trait<
		FullIdentification = Exposure<<T as system::Trait>::AccountId, ExtendedBalance>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: session::SessionHandler<<T as system::Trait>::AccountId>,
	T::OnSessionEnding: session::OnSessionEnding<<T as system::Trait>::AccountId>,
	T::SelectInitialValidators: session::SelectInitialValidators<<T as system::Trait>::AccountId>,
	T::ValidatorIdOf: Convert<<T as system::Trait>::AccountId, Option<<T as system::Trait>::AccountId>>,
{
	fn on_offence(
		offenders: &[OffenceDetails<T::AccountId, session::historical::IdentificationTuple<T>>],
		slash_fraction: &[Perbill],
	) {
		let mut ring_remaining_imbalance = <RingNegativeImbalanceOf<T>>::zero();
		let mut kton_remaining_imbalance = <KtonNegativeImbalanceOf<T>>::zero();
		let slash_reward_fraction = SlashRewardFraction::get();

		let era_now = Self::current_era();
		let mut journal = Self::era_slash_journal(era_now);
		for (details, slash_fraction) in offenders.iter().zip(slash_fraction) {
			let stash = &details.offender.0;
			let exposure = &details.offender.1;

			// Skip if the validator is invulnerable.
			if Self::invulnerables().contains(stash) {
				continue;
			}

			// Auto deselect validator on any offence and force a new era if they haven't previously
			// been deselected.
			if <Validators<T>>::exists(stash) {
				<Validators<T>>::remove(stash);
				Self::ensure_new_era();
			}

			// calculate the amount to slash
			let slash_exposure = exposure.total;
			let amount = *slash_fraction * slash_exposure;
			// in some cases `slash_fraction` can be just `0`,
			// which means we are not slashing this time.
			if amount.is_zero() {
				continue;
			}

			// make sure to disable validator till the end of this session
			if T::SessionInterface::disable_validator(stash).unwrap_or(false) {
				// force a new era, to select a new validator set
				Self::ensure_new_era();
			}
			// actually slash the validator
			let (ring_slashed_amount, kton_slash_amount) = Self::slash_validator(stash, amount, exposure, &mut journal);

			// distribute the rewards according to the slash
			// RING part
			let ring_slash_reward = slash_reward_fraction * ring_slashed_amount.peek();
			if !ring_slash_reward.is_zero() && !details.reporters.is_empty() {
				let (mut reward, rest) = ring_slashed_amount.split(ring_slash_reward);
				// split the reward between reporters equally. Division cannot fail because
				// we guarded against it in the enclosing if.
				let per_reporter = reward.peek() / (details.reporters.len() as u32).into();
				for reporter in &details.reporters {
					let (reporter_reward, rest) = reward.split(per_reporter);
					reward = rest;
					T::Ring::resolve_creating(reporter, reporter_reward);
				}
				// The rest goes to the treasury.
				ring_remaining_imbalance.subsume(reward);
				ring_remaining_imbalance.subsume(rest);
			} else {
				ring_remaining_imbalance.subsume(ring_slashed_amount);
			}

			// distribute the rewards according to the slash
			// KTON part
			let kton_slash_reward = slash_reward_fraction * kton_slash_amount.peek();
			if !kton_slash_reward.is_zero() && !details.reporters.is_empty() {
				let (mut reward, rest) = kton_slash_amount.split(kton_slash_reward);
				// split the reward between reporters equally. Division cannot fail because
				// we guarded against it in the enclosing if.
				let per_reporter = reward.peek() / (details.reporters.len() as u32).into();
				for reporter in &details.reporters {
					let (reporter_reward, rest) = reward.split(per_reporter);
					reward = rest;
					T::Kton::resolve_creating(reporter, reporter_reward);
				}
				// The rest goes to the treasury.
				kton_remaining_imbalance.subsume(reward);
				kton_remaining_imbalance.subsume(rest);
			} else {
				kton_remaining_imbalance.subsume(kton_slash_amount);
			}
		}
		<EraSlashJournal<T>>::insert(era_now, journal);

		// Handle the rest of imbalances
		T::RingSlash::on_unbalanced(ring_remaining_imbalance);
		T::KtonSlash::on_unbalanced(kton_remaining_imbalance);
	}
}

/// Filter historical offences out and only allow those from the current era.
pub struct FilterHistoricalOffences<T, R> {
	_inner: rstd::marker::PhantomData<(T, R)>,
}

impl<T, Reporter, Offender, R, O> ReportOffence<Reporter, Offender, O> for FilterHistoricalOffences<Module<T>, R>
where
	T: Trait,
	R: ReportOffence<Reporter, Offender, O>,
	O: Offence<Offender>,
{
	fn report_offence(reporters: Vec<Reporter>, offence: O) {
		// disallow any slashing from before the current era.
		let offence_session = offence.session_index();
		if offence_session >= <Module<T>>::current_era_start_session_index() {
			R::report_offence(reporters, offence)
		} else {
			<Module<T>>::deposit_event(RawEvent::OldSlashingReportDiscarded(offence_session))
		}
	}
}
