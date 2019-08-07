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

#[cfg(all(feature = "bench", test))]
extern crate test;


#[cfg(feature = "std")]
use runtime_io::with_storage;
use rstd::{prelude::*, result, collections::btree_map::BTreeMap};
use parity_codec::{HasCompact, Encode, Decode};
use srml_support::{
    StorageValue, StorageMap, EnumerableStorageMap, decl_module, decl_event,
    decl_storage, ensure, traits::{
        Currency, OnFreeBalanceZero, OnDilution, LockIdentifier, LockableCurrency,
        WithdrawReasons, OnUnbalanced, Imbalance, Get,
    },
};
use session::{OnSessionEnding, SessionIndex};
use primitives::Perbill;
use primitives::traits::{
    Convert, Zero, One, StaticLookup, CheckedSub, CheckedShl, Saturating, Bounded, SaturatedConversion,
};
#[cfg(feature = "std")]
use primitives::{Serialize, Deserialize};
use system::ensure_signed;

use dsupport::traits::OnMinted;

//use phragmen::{ACCURACY, elect, equalize, ExtendedBalance};


mod utils;

//#[cfg(any(feature = "bench", test))]
//mod mock;
//
//#[cfg(test)]
//mod tests;

//mod phragmen;

//#[cfg(all(feature = "bench", test))]
//mod benches;

const RECENT_OFFLINE_COUNT: usize = 32;
const DEFAULT_MINIMUM_VALIDATOR_COUNT: u32 = 4;
const MAX_NOMINATIONS: usize = 16;
const MAX_UNSTAKE_THRESHOLD: u32 = 10;
const MAX_UNLOCKING_CHUNKS: usize = 32;
const STAKING_ID: LockIdentifier = *b"staking ";

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;
// customed: counter for number of eras per epoch.
pub type ErasNums = u32;

pub type PowerBalance = u128;

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
pub struct ValidatorPrefs<Balance: HasCompact> {
    /// Validator should ensure this many more slashes than is necessary before being unstaked.
    #[codec(compact)]
    pub unstake_threshold: u32,
    /// Reward that validator takes up-front; only the rest is split between themselves and
    /// nominators.
    #[codec(compact)]
    pub validator_payment: Balance,
}

impl<B: Default + HasCompact + Copy> Default for ValidatorPrefs<B> {
    fn default() -> Self {
        ValidatorPrefs {
            unstake_threshold: 3,
            validator_payment: Default::default(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum StakingBalance<T: Trait> {
    Ring(RingBalanceOf<T>),
    Kton(KtonBalanceOf<T>),
}

impl<T: Trait> Default for StakingBalance<T> {
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
    DeprecatedStaked,
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
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RegularItem<RingBalance: HasCompact, Moment> {
    #[codec(compact)]
    value: RingBalance,
    #[codec(compact)]
    expire_time: Moment,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct StakingLedger<AccountId, RingBalance: HasCompact, KtonBalance: HasCompact, StakingBalance, Power, Moment> {
    pub stash: AccountId,
    #[codec(compact)]
    pub total_power: Power,
    #[codec(compact)]
    pub active_power: Power,
    // normal pattern: for ring
    #[codec(compact)]
    pub normal_ring: RingBalance,
    #[codec(compact)]
    pub normal_kton: KtonBalance,
    // regular pattern: for kton
    #[codec(compact)]
    pub regular_ring: RingBalance,
    pub regular_items: Vec<RegularItem<RingBalance, Moment>>,
    pub unlocking: Vec<UnlockChunk<StakingBalance>>,
}

impl <
    AccountId,
    RingBalance: HasCompact + Copy + Saturating,
    KtonBalance: HasCompact + Copy + Saturating,
    StakingBalanceOf,
    Power,
    Moment
> StakingLedger<AccountId, RingBalance, KtonBalance, StakingBalanceOf, Power, Moment> {

//    fn consolidate_unlocked(self, current_era: EraIndex) -> Self {
//        let mut total_power = self.total_power;
//        let mut normal_ring = self.normal_ring;
//
//        let unlocking = self.unlocking.into_iter().filter(|chunk| if chunk.era > current_era {
//            true
//        } else {
//            match value {
//                StakingBalance::Ring(r) => {
//
//                },
//
//                StakingBalance::Kton(k) => {
//
//                },
//            }
//        }).collect();
//    }
}

/// The amount of exposure (to slashing) than an individual nominator has.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct IndividualExposure<AccountId, Balance: HasCompact> {
    /// The stash account of the nominator in question.
    who: AccountId,
    /// Amount of funds exposed.
    #[codec(compact)]
    value: Balance,
}

/// A snapshot of the stake backing a single validator in the system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Exposure<AccountId, Balance: HasCompact> {
    /// The total balance backing this validator.
    #[codec(compact)]
    pub total: Balance,
    /// The validator's own stash that is exposed.
    #[codec(compact)]
    pub own: Balance,
    /// The portions of nominators stashes that are exposed.
    pub others: Vec<IndividualExposure<AccountId, Balance>>,
}


type RingBalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;
type KtonBalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::Balance;

// for ring
type PositiveImbalanceOf<T> =
<<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type NegativeImbalanceOf<T> =
<<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;


pub trait Trait: timestamp::Trait + session::Trait {
    type Ring: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
    type Kton: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;

    type Power: Get<PowerBalance> + Convert<RingBalanceOf<Self>, PowerBalance> + Convert<KtonBalanceOf<Self>, PowerBalance>;
    // basic token
    type CurrencyToVote: Convert<KtonBalanceOf<Self>, u64> + Convert<u128, KtonBalanceOf<Self>>;
    /// Some tokens minted.
    type OnRewardMinted: OnDilution<RingBalanceOf<Self>>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Handler for the unbalanced reduction when slashing a staker.
    type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

    /// Handler for the unbalanced increment when rewarding a staker.
    type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;

    /// Number of sessions per era.
    type SessionsPerEra: Get<SessionIndex>;

    /// Number of eras that staked funds must remain bonded for.
    type BondingDuration: Get<EraIndex>;

    // custom
    type Cap: Get<<Self::Ring as Currency<Self::AccountId>>::Balance>;
    type ErasPerEpoch: Get<ErasNums>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Staking {

		pub ValidatorCount get(validator_count) config(): u32;

		pub MinimumValidatorCount get(minimum_validator_count) config():
			u32 = DEFAULT_MINIMUM_VALIDATOR_COUNT;

		pub SessionReward get(session_reward) config(): Perbill = Perbill::from_parts(60);

		pub OfflineSlash get(offline_slash) config(): Perbill = Perbill::from_millionths(1000);

		pub OfflineSlashGrace get(offline_slash_grace) config(): u32;

		pub Invulnerables get(invulnerables) config(): Vec<T::AccountId>;

        pub Bonded get(bonded): map T::AccountId => Option<T::AccountId>;

        pub Ledger get(ledger): map T::AccountId => Option<StakingLedger<
            T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>, StakingBalance<T>,
            PowerBalance, T::Moment>>;

		pub Payee get(payee): map T::AccountId => RewardDestination;

		pub Validators get(validators): linked_map T::AccountId => ValidatorPrefs<RingBalanceOf<T>>;

		pub Nominators get(nominators): linked_map T::AccountId => Vec<T::AccountId>;

		pub Stakers get(stakers): map T::AccountId => Exposure<T::AccountId, KtonBalanceOf<T>>;

		pub CurrentElected get(current_elected): Vec<T::AccountId>;

		pub CurrentEra get(current_era) config(): EraIndex;

		pub CurrentSessionReward get(current_session_reward) config(): RingBalanceOf<T>;

		pub CurrentEraReward get(current_era_reward): RingBalanceOf<T>;

		pub SlotStake get(slot_stake) build(|config: &GenesisConfig<T>| {
			config.stakers.iter().map(|&(_, _, value, _)| value).min().unwrap_or_default()
		}): RingBalanceOf<T>;

		pub SlashCount get(slash_count): map T::AccountId => u32;

		pub RecentlyOffline get(recently_offline): Vec<(T::AccountId, T::BlockNumber, u32)>;

		pub ForceNewEra get(forcing_new_era): bool;

		pub EpochIndex get(epoch_index): T::BlockNumber = 0.into();

		pub ShouldOffline get(should_offline): Vec<T::AccountId>;
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
//					assert!(T::Currency::free_balance(&stash) >= balance);
					let _ = <Module<T>>::bond(
						T::Origin::from(Some(stash.clone()).into()),
						T::Lookup::unlookup(controller.clone()),
						balance,
						RewardDestination::Stash
					);
					let _ = match status {
						StakerStatus::Validator => {
							<Module<T>>::validate(
								T::Origin::from(Some(controller.clone()).into()),
								Default::default()
							)
						}, StakerStatus::Nominator(votes) => {
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
        Test(Balance, AccountId),
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
            value: StakingBalance<T>,
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

			<Bonded<T>>::insert(&stash, controller.clone());
			<Payee<T>>::insert(&stash, payee);

            let mut ledger = StakingLedger {stash: stash.clone(), ..Default::default()};
			match value {
			    StakingBalance::Ring(r) => {
			        let stash_balance = T::Ring::free_balance(&stash);
			        let value = r.min(stash_balance);
			        Self::bond_helper_in_ring(stash.clone(), controller.clone(), value, promise_month, ledger)},
			    StakingBalance::Kton(k) => {
			        let stash_balance = T::Kton::free_balance(&stash);
			        let value = k.min(stash_balance);
			        Self::bond_helper_in_kton(stash.clone(), controller.clone(), value, ledger);
			    },
			}
        }

        fn bond_extra(origin,
            value: StakingBalance<T>,
            promise_month: u32
        ) {
            let stash = ensure_signed(origin)?;
            ensure!( promise_month <= 36, "months at most is 36.");
			let controller = Self::bonded(&stash).ok_or("not a stash")?;
			let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let stash_balance = T::Ring::free_balance(&stash);
            match value {
                 StakingBalance::Ring(r) => {
                    let stash_balance = T::Ring::free_balance(&stash);
                    if let Some(extra) = stash_balance.checked_sub(&(ledger.normal_ring + ledger.regular_ring)) {
                        let extra = extra.min(r);
                        Self::bond_helper_in_ring(stash.clone(), controller.clone(), extra, promise_month, ledger);
                    }
                },

                StakingBalance::Kton(k) => {
                    let stash_balance = T::Kton::free_balance(&stash);
                    if let Some(extra) = stash_balance.checked_sub(&(ledger.normal_kton)) {
                        let extra = extra.min(k);
                       Self::bond_helper_in_kton(stash.clone(), controller.clone(), extra, ledger);
                    }
                },
            }
        }

        /// for normal_ring or normal_kton, follow the original substrate pattern
        /// for regular_ring, transform it into normal_ring first
        /// modify regular_items and regular_ring amount
        fn unbond(origin, value: StakingBalance<T>, is_regular: bool) {
            let controller = ensure_signed(origin)?;

            let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;
//            let regular_items = ledger.regular_items;
			ensure!(
				ledger.unlocking.len() < MAX_UNLOCKING_CHUNKS,
				"can not schedule more unlock chunks"
			);

		    match value {
		        StakingBalance::Ring(r) => {
		            if is_regular {
		                let now = <timestamp::Module<T>>::now();
		                let mut total_changed: RingBalanceOf<T> = Zero::zero();
                        /// for regular_ring, transform into normal one
                        let regular_items = ledger.regular_items.clone();
                        let new_regular_items = regular_items.into_iter()
                            .filter_map(|mut item| if item.expire_time > now {
                                Some(item)
                            } else {
                            // NOTE: value that a user wants to unbond must
                            // be big enough to unlock all regular_ring
                                let value = r.min(item.value);
                                ledger.regular_ring = ledger.regular_ring.saturating_sub(value);
                                ledger.normal_ring.saturating_add(value);
                                total_changed += value;
                                item.value -= value;
                                let res = if item.value.is_zero() {
                                    None
                                } else {
                                    Some(item)
                                };
                                res
                            }).collect::<Vec<_>>();
                        // reduce active power then
                        let dt_power = <T::Power as Convert<RingBalanceOf<T>, PowerBalance>>::convert(total_changed / 10000.into());
                        let dt_power = dt_power.min(ledger.active_power);
                        ledger.active_power -= dt_power;
                        ledger.regular_items = new_regular_items;
                        // update unlocking list
                        let era = Self::current_era() + T::BondingDuration::get();
				        ledger.unlocking.push(UnlockChunk { value: StakingBalance::Ring(total_changed), era });
		            } else {
		                // for normal_ring unbond
		                 let value = r.min(ledger.normal_ring);

		                let dt_power = <T::Power as Convert<RingBalanceOf<T>, PowerBalance>>::convert(value / 10000.into());
                        let dt_power = dt_power.min(ledger.active_power);
                        ledger.active_power -= dt_power;

		                let era = Self::current_era() + T::BondingDuration::get();
				        ledger.unlocking.push(UnlockChunk { value: StakingBalance::Ring(value), era });
		            }
		        },

		        StakingBalance::Kton(k) => {
                    let value = k.min(ledger.normal_kton);

                    // update active power
                    let dt_power = <T::Power as Convert<KtonBalanceOf<T>, PowerBalance>>::convert(value);
                    let dt_power = dt_power.min(ledger.active_power);
                    ledger.active_power -= dt_power;
                    let era = Self::current_era() + T::BondingDuration::get();
				    ledger.unlocking.push(UnlockChunk { value: StakingBalance::Kton(value), era });


		        },
		    }
        }
    }
}

impl<T: Trait> Module<T> {
    fn bond_helper_in_ring(
        stash: T::AccountId,
        controller: T::AccountId,
        value: RingBalanceOf<T>,
        promise_month: u32,
        mut ledger: StakingLedger<
            T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>, StakingBalance<T>,
            PowerBalance, T::Moment>,
    ) {

        // if stash promise to a extra-lock
        // there will be extra reward, kton, which
        // can also be use to stake.
        let regular_item = if !promise_month.is_zero() {
            let kton_return = utils::compute_kton_return::<T>(value, promise_month);
            ledger.regular_ring += value;

            // for now, kton_return is free
            // mint kton
            T::Kton::deposit_creating(&stash, kton_return);
            let const_month_in_seconds = 2592000;
            let expire_time = <timestamp::Module<T>>::now() + (const_month_in_seconds * promise_month).into();
            Some(RegularItem { value, expire_time })
        } else {
            ledger.normal_ring += value;
            None
        };
        if let Some(r) = regular_item {
            ledger.regular_items.push(r);
        }

        let power = <T::Power as Convert<RingBalanceOf<T>, PowerBalance>>::convert(value / 10000.into());
        ledger.total_power += power;
        ledger.active_power += power;

        Self::update_ledger(&controller, &ledger, StakingBalance::Ring(value));
    }

    fn bond_helper_in_kton(
        stash: T::AccountId,
        controller: T::AccountId,
        value: KtonBalanceOf<T>,
        mut ledger: StakingLedger<
            T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>, StakingBalance<T>,
            PowerBalance, T::Moment>,
    ) {

        let power = <T::Power as Convert<KtonBalanceOf<T>, PowerBalance>>::convert(value);
        ledger.total_power += power;
        ledger.active_power += power;

        ledger.normal_kton += value;

        Self::update_ledger(&controller, &ledger, StakingBalance::Kton(value));
    }

    fn update_ledger(
        controller: &T::AccountId,
        ledger: &StakingLedger<T::AccountId, RingBalanceOf<T>, KtonBalanceOf<T>,
            StakingBalance<T>, PowerBalance, T::Moment>,
        staking_balance: StakingBalance<T>
    ) {
        match staking_balance {
            StakingBalance::Ring(r) => T::Ring::set_lock(
                STAKING_ID,
                &ledger.stash,
                ledger.normal_ring + ledger.regular_ring,
                T::BlockNumber::max_value(),
                WithdrawReasons::all(),
            ),

            StakingBalance::Kton(k) => T::Kton::set_lock(
                STAKING_ID,
                &ledger.stash,
                ledger.normal_kton,
                T::BlockNumber::max_value(),
                WithdrawReasons::all(),
            ),
        }

        <Ledger<T>>::insert(controller, ledger);
    }
}


