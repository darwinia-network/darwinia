//! # Staking Module
//!
//! The Staking module is used to manage funds at stake by network maintainers.
//!
//! - [`staking::Trait`](./trait.Trait.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! The Staking module is the means by which a set of network maintainers (known as _authorities_
//! in some contexts and _validators_ in others) are chosen based upon those who voluntarily place
//! funds under deposit. Under deposit, those funds are rewarded under normal operation but are
//! held at pain of _slash_ (expropriation) should the staked maintainer be found not to be
//! discharging its duties properly.
//!
//! ### Terminology
//! <!-- Original author of paragraph: @gavofyork -->
//!
//! - Staking: The process of locking up funds for some time, placing them at risk of slashing
//! (loss) in order to become a rewarded maintainer of the network.
//! - Validating: The process of running a node to actively maintain the network, either by
//! producing blocks or guaranteeing finality of the chain.
//! - Nominating: The process of placing staked funds behind one or more validators in order to
//! share in any reward, and punishment, they take.
//! - Stash account: The account holding an owner's funds used for staking.
//! - Controller account: The account that controls an owner's funds for staking.
//! - Era: A (whole) number of sessions, which is the period that the validator set (and each
//! validator's active nominator set) is recalculated and where rewards are paid out.
//! - Slash: The punishment of a staker by reducing its funds.
//!
//! ### Goals
//! <!-- Original author of paragraph: @gavofyork -->
//!
//! The staking system in Darwinia NPoS is designed to make the following possible:
//!
//! - Stake funds that are controlled by a cold wallet.
//! - Withdraw some, or deposit more, funds without interrupting the role of an entity.
//! - Switch between roles (nominator, validator, idle) with minimal overhead.
//!
//! ### Scenarios
//!
//! #### Staking
//!
//! Almost any interaction with the Staking module requires a process of _**bonding**_ (also known
//! as being a _staker_). To become *bonded*, a fund-holding account known as the _stash account_,
//! which holds some or all of the funds that become frozen in place as part of the staking process,
//! is paired with an active **controller** account, which issues instructions on how they shall be
//! used.
//!
//! An account pair can become bonded using the [`bond`](./enum.Call.html#variant.bond) call.
//!
//! Stash accounts can change their associated controller using the
//! [`set_controller`](./enum.Call.html#variant.set_controller) call.
//!
//! There are three possible roles that any staked account pair can be in: `Validator`, `Nominator`
//! and `Idle` (defined in [`StakerStatus`](./enum.StakerStatus.html)). There are three
//! corresponding instructions to change between roles, namely:
//! [`validate`](./enum.Call.html#variant.validate), [`nominate`](./enum.Call.html#variant.nominate),
//! and [`chill`](./enum.Call.html#variant.chill).
//!
//! #### Validating
//!
//! A **validator** takes the role of either validating blocks or ensuring their finality,
//! maintaining the veracity of the network. A validator should avoid both any sort of malicious
//! misbehavior and going offline. Bonded accounts that state interest in being a validator do NOT
//! get immediately chosen as a validator. Instead, they are declared as a _candidate_ and they
//! _might_ get elected at the _next era_ as a validator. The result of the election is determined
//! by nominators and their votes.
//!
//! An account can become a validator candidate via the
//! [`validate`](./enum.Call.html#variant.validate) call.
//!
//! #### Nomination
//!
//! A **nominator** does not take any _direct_ role in maintaining the network, instead, it votes on
//! a set of validators  to be elected. Once interest in nomination is stated by an account, it
//! takes effect at the next election round. The funds in the nominator's stash account indicate the
//! _weight_ of its vote. Both the rewards and any punishment that a validator earns are shared
//! between the validator and its nominators. This rule incentivizes the nominators to NOT vote for
//! the misbehaving/offline validators as much as possible, simply because the nominators will also
//! lose funds if they vote poorly.
//!
//! An account can become a nominator via the [`nominate`](enum.Call.html#variant.nominate) call.
//!
//! #### Rewards and Slash
//!
//! The **reward and slashing** procedure is the core of the Staking module, attempting to _embrace
//! valid behavior_ while _punishing any misbehavior or lack of availability_.
//!
//! Reward must be claimed by stakers for each era before it gets too old by $HISTORY_DEPTH using
//! `payout_nominator` and `payout_validator` calls.
//! Only the [`T::MaxNominatorRewardedPerValidator`] biggest stakers can claim their reward. This
//! limit the i/o cost to compute nominators payout.
//!
//! Slashing can occur at any point in time, once misbehavior is reported. Once slashing is
//! determined, a value is deducted from the balance of the validator and all the nominators who
//! voted for this validator (values are deducted from the _stash_ account of the slashed entity).
//!
//! Slashing logic is further described in the documentation of the `slashing` module.
//!
//! Similar to slashing, rewards are also shared among a validator and its associated nominators.
//! Yet, the reward funds are not always transferred to the stash account and can be configured.
//! See [Reward Calculation](#reward-calculation) for more details.
//!
//! #### Chilling
//!
//! Finally, any of the roles above can choose to step back temporarily and just chill for a while.
//! This means that if they are a nominator, they will not be considered as voters anymore and if
//! they are validators, they will no longer be a candidate for the next election.
//!
//! An account can step back via the [`chill`](enum.Call.html#variant.chill) call.
//!
//! ### Session managing
//!
//! The module implement the trait `SessionManager`. Which is the only API to query new validator
//! set and allowing these validator set to be rewarded once their era is ended.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! The dispatchable functions of the Staking module enable the steps needed for entities to accept
//! and change their role, alongside some helper functions to get/set the metadata of the module.
//!
//! ### Public Functions
//!
//! The Staking module contains many public storage items and (im)mutable functions.
//!
//! ## Usage
//!
//! ### Example: Rewarding a validator by id.
//!
//! ```
//! use frame_support::{decl_module, dispatch};
//! use frame_system::{self as system, ensure_signed};
//! use darwinia_staking as staking;
//!
//! pub trait Trait: staking::Trait {}
//!
//! decl_module! {
//! 	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
//!			/// Reward a validator.
//! 		pub fn reward_myself(origin) -> dispatch::DispatchResult {
//! 			let reported = ensure_signed(origin)?;
//! 			<staking::Module<T>>::reward_by_ids(vec![(reported, 10)]);
//! 			Ok(())
//! 		}
//! 	}
//! }
//! # fn main() { }
//! ```
//!
//! ## Implementation Details
//!
//! ### Reward Calculation
//!
//! Validators and nominators are rewarded at the end of each era. The total reward of an era is
//! calculated using the era duration and the staking rate (the total amount of tokens staked by
//! nominators and validators, divided by the total token supply). It aims to incentivize toward a
//! defined staking rate. The full specification can be found
//! [here](https://research.web3.foundation/en/latest/polkadot/Token%20Economics.html#inflation-model).
//!
//! Total reward is split among validators and their nominators depending on the number of points
//! they received during the era. Points are added to a validator using
//! [`reward_by_ids`](./enum.Call.html#variant.reward_by_ids) or
//! [`reward_by_indices`](./enum.Call.html#variant.reward_by_indices).
//!
//! [`Module`](./struct.Module.html) implements
//! [`pallet_authorship::EventHandler`](../pallet_authorship/trait.EventHandler.html) to add reward points
//! to block producer and block producer of referenced uncles.
//!
//! The validator and its nominator split their reward as following:
//!
//! The validator can declare an amount, named
//! [`commission`](./struct.ValidatorPrefs.html#structfield.commission), that does not
//! get shared with the nominators at each reward payout through its
//! [`ValidatorPrefs`](./struct.ValidatorPrefs.html). This value gets deducted from the total reward
//! that is paid to the validator and its nominators. The remaining portion is split among the
//! validator and all of the nominators that nominated the validator, proportional to the value
//! staked behind this validator (_i.e._ dividing the
//! [`own`](./struct.Exposure.html#structfield.own) or
//! [`others`](./struct.Exposure.html#structfield.others) by
//! [`total`](./struct.Exposure.html#structfield.total) in [`Exposure`](./struct.Exposure.html)).
//!
//! All entities who receive a reward have the option to choose their reward destination
//! through the [`Payee`](./struct.Payee.html) storage item (see
//! [`set_payee`](enum.Call.html#variant.set_payee)), to be one of the following:
//!
//! - Controller account, (obviously) not increasing the staked value.
//! - Stash account, not increasing the staked value.
//! - Stash account, also increasing the staked value.
//!
//! ### Additional Fund Management Operations
//!
//! Any funds already placed into stash can be the target of the following operations:
//!
//! The controller account can free a portion (or all) of the funds using the
//! [`unbond`](enum.Call.html#variant.unbond) call. Note that the funds are not immediately
//! accessible. Instead, a duration denoted by [`BondingDurationInEra`](./struct.BondingDurationInEra.html)
//! (in number of eras) must pass until the funds can actually be removed. Once the
//! `BondingDurationInEra` is over, the [`withdraw_unbonded`](./enum.Call.html#variant.withdraw_unbonded)
//! call can be used to actually withdraw the funds.
//!
//! Note that there is a limitation to the number of fund-chunks that can be scheduled to be
//! unlocked in the future via [`unbond`](enum.Call.html#variant.unbond). In case this maximum
//! (`MAX_UNLOCKING_CHUNKS`) is reached, the bonded account _must_ first wait until a successful
//! call to `withdraw_unbonded` to remove some of the chunks.
//!
//! ### Election Algorithm
//!
//! The current election algorithm is implemented based on Phragmén.
//! The reference implementation can be found
//! [here](https://github.com/w3f/consensus/tree/master/NPoS).
//!
//! The election algorithm, aside from electing the validators with the most stake value and votes,
//! tries to divide the nominator votes among candidates in an equal manner. To further assure this,
//! an optional post-processing can be applied that iteratively normalizes the nominator staked
//! values until the total difference among votes of a particular nominator are less than a
//! threshold.
//!
//! ## GenesisConfig
//!
//! The Staking module depends on the [`GenesisConfig`](./struct.GenesisConfig.html).
//! The `GenesisConfig` is optional and allow to set some initial stakers.
//!
//! ## Related Modules
//!
//! - [Balances](../pallet_balances/index.html): Used to manage values at stake.
//! - [Session](../pallet_session/index.html): Used to manage sessions. Also, a list of new validators
//! is stored in the Session module's `Validators` at the end of each era.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(drain_filter)]
#![recursion_limit = "128"]

#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod darwinia_tests;
#[cfg(test)]
mod substrate_tests;

mod inflation;
mod slashing;

mod types {
	use crate::*;

	/// Counter for the number of eras that have passed.
	pub type EraIndex = u32;
	/// Counter for the number of "reward" points earned by a given validator.
	pub type RewardPoint = u32;

	/// Balance of an account.
	pub type Balance = u128;
	/// Type used for expressing timestamp.
	pub type Moment = Timestamp;

	pub type AccountId<T> = <T as frame_system::Trait>::AccountId;

	pub type RingBalance<T> = <RingCurrency<T> as Currency<AccountId<T>>>::Balance;
	pub type RingPositiveImbalance<T> = <RingCurrency<T> as Currency<AccountId<T>>>::PositiveImbalance;
	pub type RingNegativeImbalance<T> = <RingCurrency<T> as Currency<AccountId<T>>>::NegativeImbalance;

	pub type KtonBalance<T> = <KtonCurrency<T> as Currency<AccountId<T>>>::Balance;
	pub type KtonPositiveImbalance<T> = <KtonCurrency<T> as Currency<AccountId<T>>>::PositiveImbalance;
	pub type KtonNegativeImbalance<T> = <KtonCurrency<T> as Currency<AccountId<T>>>::NegativeImbalance;

	pub type StakingLedgerT<T> =
		StakingLedger<AccountId<T>, RingBalance<T>, KtonBalance<T>, BlockNumber<T>, MomentT<T>>;
	pub type StakingBalanceT<T> = StakingBalance<RingBalance<T>, KtonBalance<T>>;

	pub type MomentT<T> = <TimeT<T> as Time>::Moment;

	/// A timestamp: milliseconds since the unix epoch.
	/// `u64` is enough to represent a duration of half a billion years, when the
	/// time scale is milliseconds.
	type Timestamp = u64;

	type BlockNumber<T> = <T as frame_system::Trait>::BlockNumber;
	type TimeT<T> = <T as Trait>::Time;
	type RingCurrency<T> = <T as Trait>::RingCurrency;
	type KtonCurrency<T> = <T as Trait>::KtonCurrency;
}

pub use types::EraIndex;

use codec::{Decode, Encode, HasCompact};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	storage::IterableStorageMap,
	traits::{Currency, Get, Imbalance, OnUnbalanced, Time},
	weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::{
	traits::{AtLeast32Bit, CheckedSub, Convert, EnsureOrigin, SaturatedConversion, Saturating, StaticLookup, Zero},
	DispatchResult, PerThing, Perbill, Perquintill, RuntimeDebug,
};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_staking::{
	offence::{Offence, OffenceDetails, OffenceError, OnOffenceHandler, ReportOffence},
	SessionIndex,
};
use sp_std::{collections::btree_map::BTreeMap, convert::TryInto, marker::PhantomData, prelude::*};

use darwinia_phragmen::{PhragmenStakedAssignment, Power, Votes};
use darwinia_support::{
	balance::lock::*,
	traits::{OnDepositRedeem, OnUnbalancedKton},
};
use types::*;

const DEFAULT_MINIMUM_VALIDATOR_COUNT: u32 = 4;
const MONTH_IN_MINUTES: Moment = 30 * 24 * 60;
const MONTH_IN_MILLISECONDS: Moment = MONTH_IN_MINUTES * 60 * 1000;
const MAX_NOMINATIONS: usize = 16;
const MAX_UNLOCKING_CHUNKS: usize = 32;
const STAKING_ID: LockIdentifier = *b"staking ";

// --- enum ---

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
	Staked { promise_month: Moment },
	/// Pay into the stash account, not increasing the amount at stake.
	Stash,
	/// Pay into the controller account.
	Controller,
}

impl Default for RewardDestination {
	fn default() -> Self {
		RewardDestination::Staked { promise_month: 0 }
	}
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

/// To unify *Ring* and *Kton* balances.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum StakingBalance<RingBalance, KtonBalance>
where
	RingBalance: HasCompact,
	KtonBalance: HasCompact,
{
	RingBalance(RingBalance),
	KtonBalance(KtonBalance),
}

impl<RingBalance, KtonBalance> Default for StakingBalance<RingBalance, KtonBalance>
where
	RingBalance: Default + HasCompact,
	KtonBalance: Default + HasCompact,
{
	fn default() -> Self {
		StakingBalance::RingBalance(Default::default())
	}
}

// --- struct ---

/// Information regarding the active era (era in used in session).
#[derive(Debug, Encode, Decode)]
pub struct ActiveEraInfo<Moment> {
	/// Index of era.
	index: EraIndex,
	/// Moment of start
	///
	/// Start can be none if start hasn't been set for the era yet,
	/// Start is set on the first on_finalize of the era to guarantee usage of `Time`.
	start: Option<Moment>,
}

/// Reward points of an era. Used to split era total payout between validators.
///
/// This points will be used to reward validators and their respective nominators.
#[derive(PartialEq, Encode, Decode, Default, Debug)]
pub struct EraRewardPoints<AccountId: Ord> {
	/// Total number of points. Equals the sum of reward points for each validator.
	total: RewardPoint,
	/// The reward points earned by a given validator.
	individual: BTreeMap<AccountId, RewardPoint>,
}

/// Preference of what happens regarding validation.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ValidatorPrefs {
	/// Reward that validator takes up-front; only the rest is split between themselves and
	/// nominators.
	#[codec(compact)]
	pub commission: Perbill,
}

impl Default for ValidatorPrefs {
	fn default() -> Self {
		ValidatorPrefs {
			commission: Default::default(),
		}
	}
}

/// The ledger of a (bonded) stash.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct StakingLedger<AccountId, RingBalance, KtonBalance, BlockNumber, Timestamp>
where
	RingBalance: HasCompact,
	KtonBalance: HasCompact,
{
	/// The stash account whose balance is actually locked and at stake.
	pub stash: AccountId,

	/// The total amount of the stash's *RING* that will be at stake in any forthcoming
	/// rounds.
	#[codec(compact)]
	pub active_ring: RingBalance,
	// active time-deposit ring
	#[codec(compact)]
	pub active_deposit_ring: RingBalance,

	/// The total amount of the stash's *KTON* that will be at stake in any forthcoming
	/// rounds.
	#[codec(compact)]
	pub active_kton: KtonBalance,

	// If you deposit *RING* for a minimum period,
	// you can get *KTON* as bonus which can also be used for staking.
	pub deposit_items: Vec<TimeDepositItem<RingBalance, Timestamp>>,

	// TODO doc
	pub ring_staking_lock: StakingLock<RingBalance, BlockNumber>,
	// TODO doc
	pub kton_staking_lock: StakingLock<KtonBalance, BlockNumber>,

	/// The latest and highest era which the staker has claimed reward for.
	pub last_reward: Option<EraIndex>,
}

impl<AccountId, RingBalance, KtonBalance, BlockNumber, Timestamp>
	StakingLedger<AccountId, RingBalance, KtonBalance, BlockNumber, Timestamp>
where
	RingBalance: AtLeast32Bit + Saturating + Copy,
	KtonBalance: AtLeast32Bit + Saturating + Copy,
	BlockNumber: PartialOrd,
	Timestamp: PartialOrd,
{
	/// Slash the validator for a given amount of balance. This can grow the value
	/// of the slash in the case that the validator has less than `minimum_balance`
	/// active funds. Returns the amount of funds actually slashed.
	///
	/// Slashes from `active` funds first, and then `unlocking`, starting with the
	/// chunks that are closest to unlocking.
	fn slash(
		&mut self,
		slash_ring: RingBalance,
		slash_kton: KtonBalance,
		bn: BlockNumber,
		ts: Timestamp,
	) -> (RingBalance, KtonBalance) {
		let slash_out_of = |active_ring: &mut RingBalance,
		                    active_deposit_ring: &mut RingBalance,
		                    deposit_item: &mut Vec<TimeDepositItem<RingBalance, Timestamp>>,
		                    active_kton: &mut KtonBalance,
		                    slash_ring: &mut RingBalance,
		                    slash_kton: &mut KtonBalance| {
			let slashable_active_ring = (*slash_ring).min(*active_ring);
			let slashable_active_kton = (*slash_kton).min(*active_kton);

			if !slashable_active_ring.is_zero() {
				let slashable_normal_ring = *active_ring - *active_deposit_ring;
				if let Some(mut slashable_deposit_ring) = slashable_active_ring.checked_sub(&slashable_normal_ring) {
					deposit_item.drain_filter(|item| {
						if ts >= item.expire_time {
							true
						} else {
							if slashable_deposit_ring.is_zero() {
								false
							} else {
								if let Some(new_slashable_deposit_ring) =
									slashable_deposit_ring.checked_sub(&item.value)
								{
									slashable_deposit_ring = new_slashable_deposit_ring;
									true
								} else {
									item.value -= sp_std::mem::replace(&mut slashable_deposit_ring, Zero::zero());
									false
								}
							}
						}
					});
				}
				*active_ring -= slashable_active_ring;
				*slash_ring -= slashable_active_ring;
			}

			if !slashable_active_kton.is_zero() {
				*active_kton -= slashable_active_kton;
				*slash_kton -= slashable_active_kton;
			}
		};

		let (mut apply_slash_ring, mut apply_slash_kton) = (slash_ring, slash_kton);
		let StakingLedger {
			active_ring,
			active_deposit_ring,
			deposit_items,
			active_kton,
			ring_staking_lock,
			kton_staking_lock,
			..
		} = self;

		slash_out_of(
			active_ring,
			active_deposit_ring,
			deposit_items,
			active_kton,
			&mut apply_slash_ring,
			&mut apply_slash_kton,
		);

		if !apply_slash_ring.is_zero() {
			ring_staking_lock.unbondings.drain_filter(|lock| {
				if bn >= lock.until {
					true
				} else {
					if apply_slash_ring.is_zero() {
						false
					} else {
						if apply_slash_ring > lock.amount {
							apply_slash_ring -= lock.amount;
							true
						} else {
							lock.amount -= sp_std::mem::replace(&mut apply_slash_ring, Zero::zero());
							false
						}
					}
				}
			});
		}
		if !apply_slash_kton.is_zero() {
			kton_staking_lock.unbondings.drain_filter(|lock| {
				if bn >= lock.until {
					true
				} else {
					if apply_slash_kton.is_zero() {
						false
					} else {
						if apply_slash_kton > lock.amount {
							apply_slash_kton -= lock.amount;

							true
						} else {
							lock.amount -= sp_std::mem::replace(&mut apply_slash_kton, Zero::zero());
							false
						}
					}
				}
			});
		}

		(slash_ring - apply_slash_ring, slash_kton - apply_slash_kton)
	}
}

/// The *Ring* under deposit.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct TimeDepositItem<RingBalance: HasCompact, Moment> {
	#[codec(compact)]
	pub value: RingBalance,
	#[codec(compact)]
	pub start_time: Moment,
	#[codec(compact)]
	pub expire_time: Moment,
}

/// A record of the nominations made by a specific account.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Nominations<AccountId> {
	/// The targets of nomination.
	pub targets: Vec<AccountId>,
	/// The era the nominations were submitted.
	///
	/// Except for initial nominations which are considered submitted at era 0.
	pub submitted_in: EraIndex,
	/// Whether the nominations have been suppressed.
	pub suppressed: bool,
}

/// A snapshot of the stake backing a single validator in the system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Exposure<AccountId, RingBalance, KtonBalance>
where
	RingBalance: HasCompact,
	KtonBalance: HasCompact,
{
	/// The validator's own stash that is exposed.
	#[codec(compact)]
	pub own_ring_balance: RingBalance,
	#[codec(compact)]
	pub own_kton_balance: KtonBalance,
	pub own_power: Power,
	/// The total balance backing this validator.
	pub total_power: Power,
	/// The portions of nominators stashes that are exposed.
	pub others: Vec<IndividualExposure<AccountId, RingBalance, KtonBalance>>,
}

/// The amount of exposure (to slashing) than an individual nominator has.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct IndividualExposure<AccountId, RingBalance, KtonBalance>
where
	RingBalance: HasCompact,
	KtonBalance: HasCompact,
{
	/// The stash account of the nominator in question.
	who: AccountId,
	/// Amount of funds exposed.
	#[codec(compact)]
	ring_balance: RingBalance,
	#[codec(compact)]
	kton_balance: KtonBalance,
	power: Power,
}

/// A pending slash record. The value of the slash has been computed but not applied yet,
/// rather deferred for several eras.
#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct UnappliedSlash<AccountId, RingBalance, KtonBalance> {
	/// The stash ID of the offending validator.
	validator: AccountId,
	/// The validator's own slash.
	own: slashing::RK<RingBalance, KtonBalance>,
	/// All other slashed stakers and amounts.
	others: Vec<(AccountId, slashing::RK<RingBalance, KtonBalance>)>,
	/// Reporters of the offence; bounty payout recipients.
	reporters: Vec<AccountId>,
	/// The amount of payout.
	payout: slashing::RK<RingBalance, KtonBalance>,
}

// --- trait ---

/// Means for interacting with a specialized version of the `session` trait.
///
/// This is needed because `Staking` sets the `ValidatorIdOf` of the `pallet_session::Trait`
pub trait SessionInterface<AccountId>: frame_system::Trait {
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

impl<T: Trait> SessionInterface<AccountId<T>> for T
where
	T: pallet_session::Trait<ValidatorId = AccountId<T>>,
	T: pallet_session::historical::Trait<
		FullIdentification = Exposure<AccountId<T>, RingBalance<T>, KtonBalance<T>>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<AccountId<T>>,
	T::SessionManager: pallet_session::SessionManager<AccountId<T>>,
	T::ValidatorIdOf: Convert<AccountId<T>, Option<AccountId<T>>>,
{
	fn disable_validator(validator: &AccountId<T>) -> Result<bool, ()> {
		<pallet_session::Module<T>>::disable(validator)
	}

	fn validators() -> Vec<AccountId<T>> {
		<pallet_session::Module<T>>::validators()
	}

	fn prune_historical_up_to(up_to: SessionIndex) {
		<pallet_session::historical::Module<T>>::prune_up_to(up_to);
	}
}

pub trait Trait: frame_system::Trait {
	/// Time used for computing era duration.
	///
	/// It is guaranteed to start being called from the first `on_finalize`. Thus value at genesis
	/// is not used.
	type Time: Time;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// Number of sessions per era.
	type SessionsPerEra: Get<SessionIndex>;

	/// Number of eras that staked funds must remain bonded for.
	type BondingDurationInEra: Get<EraIndex>;
	/// Number of eras that staked funds must remain bonded for.
	type BondingDurationInBlockNumber: Get<Self::BlockNumber>;

	/// Number of eras that slashes are deferred by, after computation. This
	/// should be less than the bonding duration. Set to 0 if slashes should be
	/// applied immediately, without opportunity for intervention.
	type SlashDeferDuration: Get<EraIndex>;

	/// The origin which can cancel a deferred slash. Root can always do this.
	type SlashCancelOrigin: EnsureOrigin<Self::Origin>;

	/// Interface for interacting with a session module.
	type SessionInterface: self::SessionInterface<Self::AccountId>;

	/// The maximum number of nominator rewarded for each validator.
	///
	/// For each validator only the `$MaxNominatorRewardedPerValidator` biggest stakers can claim
	/// their reward. This used to limit the i/o cost for the nominator payout.
	type MaxNominatorRewardedPerValidator: Get<u32>;

	/// The *RING* currency.
	type RingCurrency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	/// Tokens have been minted and are unused for validator-reward.
	type RingRewardRemainder: OnUnbalanced<RingNegativeImbalance<Self>>;
	/// Handler for the unbalanced *RING* reduction when slashing a staker.
	type RingSlash: OnUnbalanced<RingNegativeImbalance<Self>>;
	/// Handler for the unbalanced *RING* increment when rewarding a staker.
	type RingReward: OnUnbalanced<RingPositiveImbalance<Self>>;

	/// The *KTON* currency.
	type KtonCurrency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	// FIXME: Ugly hack due to https://github.com/rust-lang/rust/issues/31844#issuecomment-557918823
	/// Handler for the unbalanced *KTON* reduction when slashing a staker.
	type KtonSlash: OnUnbalancedKton<KtonNegativeImbalance<Self>>;
	/// Handler for the unbalanced *KTON* increment when rewarding a staker.
	type KtonReward: OnUnbalanced<KtonPositiveImbalance<Self>>;

	// TODO: doc
	type Cap: Get<RingBalance<Self>>;
	// TODO: doc
	type TotalPower: Get<Power>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Staking {
		/// Number of era to keep in history.
		///
		/// Information is kept for eras in `[current_era - history_depth; current_era]
		///
		/// Must be more than the number of era delayed by session otherwise.
		/// i.e. active era must always be in history.
		/// i.e. `active_era > current_era - history_depth` must be guaranteed.
		HistoryDepth get(fn history_depth) config(): u32 = 336;

		/// The ideal number of staking participants.
		pub ValidatorCount get(fn validator_count) config(): u32;

		/// Minimum number of staking participants before emergency conditions are imposed.
		pub MinimumValidatorCount
			get(fn minimum_validator_count) config()
			: u32 = DEFAULT_MINIMUM_VALIDATOR_COUNT;

		/// Any validators that may never be slashed or forcibly kicked. It's a Vec since they're
		/// easy to initialize and the performance hit is minimal (we expect no more than four
		/// invulnerables) and restricted to testnets.
		pub Invulnerables get(fn invulnerables) config(): Vec<T::AccountId>;

		/// Map from all locked "stash" accounts to the controller account.
		pub Bonded get(fn bonded): map hasher(twox_64_concat) T::AccountId => Option<T::AccountId>;

		/// Map from all (unlocked) "controller" accounts to the info regarding the staking.
		pub Ledger get(fn ledger): map hasher(blake2_128_concat) T::AccountId => Option<StakingLedgerT<T>>;

		/// Where the reward payment should be made. Keyed by stash.
		pub Payee get(fn payee): map hasher(twox_64_concat) T::AccountId => RewardDestination;

		/// The map from (wannabe) validator stash key to the preferences of that validator.
		pub Validators
			get(fn validators)
			: map hasher(twox_64_concat) T::AccountId => ValidatorPrefs;

		/// The map from nominator stash key to the set of stash keys of all validators to nominate.
		pub Nominators
			get(fn nominators)
			: map hasher(twox_64_concat) T::AccountId => Option<Nominations<T::AccountId>>;

		/// The current era index.
		///
		/// This is the latest planned era, depending on how session module queues the validator
		/// set, it might be active or not.
		pub CurrentEra get(fn current_era): Option<EraIndex>;

		/// The active era information, it holds index and start.
		///
		/// The active era is the era currently rewarded.
		/// Validator set of this era must be equal to `SessionInterface::validators`.
		pub ActiveEra get(fn active_era): Option<ActiveEraInfo<MomentT<T>>>;

		/// The session index at which the era start for the last `HISTORY_DEPTH` eras
		pub ErasStartSessionIndex
			get(fn eras_start_session_index)
			: map hasher(twox_64_concat) EraIndex => Option<SessionIndex>;

		/// Exposure of validator at era.
		///
		/// This is keyed first by the era index to allow bulk deletion and then the stash account.
		///
		/// Is it removed after `HISTORY_DEPTH` eras.
		/// If stakers hasn't been set or has been removed then empty exposure is returned.
		pub ErasStakers
			get(fn eras_stakers)
			: double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
				=> Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>;

		/// Clipped Exposure of validator at era.
		///
		/// This is similar to [`ErasStakers`] but number of nominators exposed is reduce to the
		/// `T::MaxNominatorRewardedPerValidator` biggest stakers.
		/// This is used to limit the i/o cost for the nominator payout.
		///
		/// This is keyed fist by the era index to allow bulk deletion and then the stash account.
		///
		/// Is it removed after `HISTORY_DEPTH` eras.
		/// If stakers hasn't been set or has been removed then empty exposure is returned.
		pub ErasStakersClipped
			get(fn eras_stakers_clipped)
			: double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
				=> Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>;

		/// Similarly to `ErasStakers` this holds the preferences of validators.
		///
		/// This is keyed fist by the era index to allow bulk deletion and then the stash account.
		///
		/// Is it removed after `HISTORY_DEPTH` eras.
		// If prefs hasn't been set or has been removed then 0 commission is returned.
		pub ErasValidatorPrefs
			get(fn eras_validator_prefs)
			: double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
				=> ValidatorPrefs;

		/// The total validator era payout for the last `HISTORY_DEPTH` eras.
		///
		/// Eras that haven't finished yet or has been removed doesn't have reward.
		pub ErasValidatorReward
			get(fn eras_validator_reward)
			: map hasher(twox_64_concat) EraIndex => Option<RingBalance<T>>;

		/// Rewards for the last `HISTORY_DEPTH` eras.
		/// If reward hasn't been set or has been removed then 0 reward is returned.
		pub ErasRewardPoints
			get(fn eras_reward_points)
			: map hasher(twox_64_concat) EraIndex => EraRewardPoints<T::AccountId>;

		/// The total amount staked for the last `HISTORY_DEPTH` eras.
		/// If total hasn't been set or has been removed then 0 stake is returned.
		pub ErasTotalStake
			get(fn eras_total_stake)
			: map hasher(twox_64_concat) EraIndex => Power;

		/// True if the next session change will be a new era regardless of index.
		pub ForceEra get(fn force_era) config(): Forcing;

		/// The percentage of the slash that is distributed to reporters.
		///
		/// The rest of the slashed value is handled by the `Slash`.
		pub SlashRewardFraction get(fn slash_reward_fraction) config(): Perbill;

		/// The amount of currency given to reporters of a slash event which was
		/// canceled by extraordinary circumstances (e.g. governance).
		pub CanceledSlashPayout get(fn canceled_payout) config(): Power;

		/// All unapplied slashes that are queued for later.
		pub UnappliedSlashes
			: map hasher(twox_64_concat) EraIndex
				=> Vec<UnappliedSlash<T::AccountId, RingBalance<T>, KtonBalance<T>>>;

		/// A mapping from still-bonded eras to the first session index of that era.
		///
		/// Must contains information for eras for the range:
		/// `[active_era - bounding_duration; active_era]`
		BondedEras: Vec<(EraIndex, SessionIndex)>;

		/// All slashing events on validators, mapped by era to the highest slash proportion
		/// and slash value of the era.
		ValidatorSlashInEra
			: double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
				=> Option<(Perbill, slashing::RKT<T>)>;

		/// All slashing events on nominators, mapped by era to the highest slash value of the era.
		NominatorSlashInEra
			: double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
				=> Option<slashing::RKT<T>>;

		/// Slashing spans for stash accounts.
		SlashingSpans: map hasher(twox_64_concat) T::AccountId => Option<slashing::SlashingSpans>;

		/// Records information about the maximum slash of a stash within a slashing span,
		/// as well as how much reward has been paid out.
		SpanSlash
			: map hasher(twox_64_concat) (T::AccountId, slashing::SpanIndex)
				=> slashing::SpanRecord<RingBalance<T>, KtonBalance<T>>;

		/// The earliest era for which we have a pending, unapplied slash.
		EarliestUnappliedSlash: Option<EraIndex>;

		// --- custom ---

		// --- immutable ---

		// TODO: doc
		pub LivingTime get(fn living_time): MomentT<T>;

		/// The percentage of the total payout that is distributed to validators and nominators
		///
		/// The reset might go to Treasury or something else.
		pub PayoutFraction get(fn payout_fraction) config(): Perbill;

		// --- mutable ---

		/// Total *Ring* in pool.
		pub RingPool get(fn ring_pool): RingBalance<T>;
		/// Total *Kton* in pool.
		pub KtonPool get(fn kton_pool): KtonBalance<T>;
	}
	add_extra_genesis {
		config(stakers): Vec<(T::AccountId, T::AccountId, RingBalance<T>, StakerStatus<T::AccountId>)>;
		build(|config: &GenesisConfig<T>| {
			for &(ref stash, ref controller, r, ref status) in &config.stakers {
				assert!(
					T::RingCurrency::free_balance(&stash) >= r,
					"Stash does not have enough balance to bond.",
				);
				let _ = <Module<T>>::bond(
					T::Origin::from(Some(stash.to_owned()).into()),
					T::Lookup::unlookup(controller.to_owned()),
					StakingBalance::RingBalance(r),
					RewardDestination::Staked { promise_month: 0 },
					0,
				);
				let _ = match status {
					StakerStatus::Validator => {
						<Module<T>>::validate(
							T::Origin::from(Some(controller.to_owned()).into()),
							Default::default(),
						)
					},
					StakerStatus::Nominator(votes) => {
						<Module<T>>::nominate(
							T::Origin::from(Some(controller.to_owned()).into()),
							votes.iter().map(|l| T::Lookup::unlookup(l.to_owned())).collect(),
						)
					}, _ => Ok(())
				};
			}
		});
	}
}

decl_event!(
	pub enum Event<T>
	where
		<T as frame_system::Trait>::AccountId,
		<T as frame_system::Trait>::BlockNumber,
		RingBalance = RingBalance<T>,
		KtonBalance = KtonBalance<T>,
		MomentT = MomentT<T>,
	{
		/// Bond succeed.
		/// `amount` in `RingBalance<T>`, `start_time` in `MomentT<T>`, `expired_time` in `MomentT<T>`
		BondRing(RingBalance, MomentT, MomentT),
		/// Bond succeed.
		/// `amount`
		BondKton(KtonBalance),

		/// Unbond succeed.
		/// `amount` in `RingBalance<T>`, `now` in `BlockNumber`
		UnbondRing(RingBalance, BlockNumber),
		/// Unbond succeed.
		/// `amount` om `KtonBalance<T>`, `now` in `BlockNumber`
		UnbondKton(KtonBalance, BlockNumber),

		/// The staker has been rewarded by this amount. AccountId is controller account.
		Reward(AccountId, RingBalance),

		/// One validator (and its nominators) has been slashed by the given amount.
		Slash(AccountId, RingBalance, KtonBalance),
		/// An old slashing report from a prior era was discarded because it could
		/// not be processed.
		OldSlashingReportDiscarded(SessionIndex),
	}
);

decl_error! {
	/// Error for the staking module.
	pub enum Error for Module<T: Trait> {
		/// Not a controller account.
		NotController,
		/// Not a stash account.
		NotStash,
		/// Stash is already bonded.
		AlreadyBonded,
		/// Controller is already paired.
		AlreadyPaired,
		/// Targets cannot be empty.
		EmptyTargets,
		/// Duplicate index.
		DuplicateIndex,
		/// Slash record index out of bounds.
		InvalidSlashIndex,
		/// Can not bond with value less than minimum balance.
		InsufficientValue,
		/// Can not schedule more unlock chunks.
		NoMoreChunks,
		/// Attempting to target a stash that still has funds.
		FundedTarget,
		/// Invalid era to reward.
		InvalidEraToReward,
		/// Invalid number of nominations.
		InvalidNumberOfNominations,
		/// Items are not sorted and unique.
		NotSortedAndUnique,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Number of sessions per era.
		const SessionsPerEra: SessionIndex = T::SessionsPerEra::get();

		/// Number of eras that staked funds must remain bonded for.
		const BondingDurationInEra: EraIndex = T::BondingDurationInEra::get();
		/// Number of BlockNumbers that staked funds must remain bonded for.
		const BondingDurationInBlockNumber: T::BlockNumber = T::BondingDurationInBlockNumber::get();

		// TODO: doc
		const Cap: RingBalance<T> = T::Cap::get();

		// TODO: doc
		const TotalPower: Power = T::TotalPower::get();

		type Error = Error<T>;

		fn deposit_event() = default;

		fn on_finalize() {
			if let Some(mut active_era) = Self::active_era() {
				if active_era.start.is_none() {
					active_era.start = Some(T::Time::now());
					<ActiveEra<T>>::put(active_era);
				}
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
			value: StakingBalanceT<T>,
			payee: RewardDestination,
			promise_month: Moment
		) {
			let stash = ensure_signed(origin)?;
			ensure!(!<Bonded<T>>::contains_key(&stash), <Error<T>>::AlreadyBonded);

			let controller = T::Lookup::lookup(controller)?;
			ensure!(!<Ledger<T>>::contains_key(&controller), <Error<T>>::AlreadyPaired);

			let ledger = StakingLedger {
				stash: stash.clone(),
				last_reward: Self::current_era(),
				..Default::default()
			};
			let promise_month = promise_month.min(36);

			match value {
				StakingBalance::RingBalance(r) => {
					// reject a bond which is considered to be _dust_.
					ensure!(
						r >= T::RingCurrency::minimum_balance(),
						<Error<T>>::InsufficientValue,
					);

					let usable_balance = T::RingCurrency::usable_balance(&stash);
					let value = r.min(usable_balance);
					let (start_time, expire_time) = Self::bond_ring(
						&stash,
						&controller,
						value,
						promise_month,
						ledger,
					);

					<RingPool<T>>::mutate(|r| *r += value);
					Self::deposit_event(RawEvent::BondRing(value, start_time, expire_time));
				},
				StakingBalance::KtonBalance(k) => {
					// reject a bond which is considered to be _dust_.
					ensure!(
						k >= T::KtonCurrency::minimum_balance(),
						<Error<T>>::InsufficientValue,
					);

					let usable_balance = T::KtonCurrency::usable_balance(&stash);
					let value = k.min(usable_balance);

					Self::bond_kton(&controller, value, ledger);

					<KtonPool<T>>::mutate(|k| *k += value);
					Self::deposit_event(RawEvent::BondKton(value));
				},
			}

			// You're auto-bonded forever, here. We might improve this by only bonding when
			// you actually validate/nominate and remove once you unbond __everything__.
			<Bonded<T>>::insert(&stash, &controller);
			<Payee<T>>::insert(&stash, payee);

			<frame_system::Module<T>>::inc_ref(&stash);
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
		fn bond_extra(origin, max_additional: StakingBalanceT<T>, promise_month: Moment) {
			let stash = ensure_signed(origin)?;
			let controller = Self::bonded(&stash).ok_or(<Error<T>>::NotStash)?;
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let promise_month = promise_month.min(36);

			match max_additional {
				 StakingBalance::RingBalance(r) => {
					let extra = T::RingCurrency::usable_balance(&stash);
					let extra = extra.min(r);
					let (start_time, expire_time) = Self::bond_ring(
						&stash,
						&controller,
						extra,
						promise_month,
						ledger,
					);

					<RingPool<T>>::mutate(|r| *r += extra);
					Self::deposit_event(RawEvent::BondRing(extra, start_time, expire_time));
				},
				StakingBalance::KtonBalance(k) => {
					let extra = T::KtonCurrency::usable_balance(&stash);
					let extra = extra.min(k);

					Self::bond_kton(&controller, extra, ledger);

					<KtonPool<T>>::mutate(|k| *k += extra);
					Self::deposit_event(RawEvent::BondKton(extra));
				},
			}
		}

		/// Deposit some extra amount ring, and return kton to the controller.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - O(1).
		/// - One DB entry.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn deposit_extra(origin, value: RingBalance<T>, promise_month: Moment) {
			let stash = ensure_signed(origin)?;
			let controller = Self::bonded(&stash).ok_or(<Error<T>>::NotStash)?;
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let start_time = T::Time::now();
			let expire_time = start_time + <MomentT<T>>::saturated_from((promise_month * MONTH_IN_MILLISECONDS).into());
			let promise_month = promise_month.max(3).min(36);
			let mut ledger = Self::clear_mature_deposits(ledger);
			let StakingLedger {
				stash,
				active_ring,
				active_deposit_ring,
				deposit_items,
				..
			} = &mut ledger;
			let value = value.min(*active_ring - *active_deposit_ring);
			let kton_return = inflation::compute_kton_return::<T>(value, promise_month);
			let kton_positive_imbalance = T::KtonCurrency::deposit_creating(stash, kton_return);

			T::KtonReward::on_unbalanced(kton_positive_imbalance);
			*active_deposit_ring += value;
			deposit_items.push(TimeDepositItem {
				value,
				start_time,
				expire_time,
			});

			<Ledger<T>>::insert(&controller, ledger);
			Self::deposit_event(RawEvent::BondRing(value, start_time, expire_time));
		}

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
		/// After all pledged Ring and Kton are unbonded, the bonded accounts, namely stash and
		/// controller, will also be unbonded.  Once user want to bond again, the `bond` method
		/// should be called. If there are still pledged Ring or Kton and user want to bond more
		/// values, the `bond_extra` method should be called.
		///
		/// # <weight>
		/// - Independent of the arguments. Limited but potentially exploitable complexity.
		/// - Contains a limited number of reads.
		/// - Each call (requires the remainder of the bonded balance to be above `minimum_balance`)
		///   will cause a new entry to be inserted into a vector (`StakingLock.unbondings`) kept in storage.
		/// - One DB entry.
		/// </weight>
		///
		/// Only active normal ring can be unbond
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn unbond(origin, value: StakingBalanceT<T>) {
			let controller = ensure_signed(origin)?;
			let mut ledger = Self::clear_mature_deposits(Self::ledger(&controller).ok_or(<Error<T>>::NotController)?);
			let StakingLedger {
				active_ring,
				active_deposit_ring,
				active_kton,
				ring_staking_lock,
				kton_staking_lock,
				..
			} = &mut ledger;
			let now = <frame_system::Module<T>>::block_number();

			ring_staking_lock.update(now);
			kton_staking_lock.update(now);

			// Due to the macro parser, we've to add a bracket.
			// Actually, this's totally wrong:
			//     `a as u32 + b as u32 < c`
			// Workaround:
			//     1. `(a as u32 + b as u32) < c`
			//     2. `let c_ = a as u32 + b as u32; c_ < c`
			ensure!(
				(ring_staking_lock.unbondings.len() + kton_staking_lock.unbondings.len()) < MAX_UNLOCKING_CHUNKS,
				<Error<T>>::NoMoreChunks,
			);

			let mut unbond_ring: RingBalance<T> = Zero::zero();
			let mut unbond_kton: KtonBalance<T> = Zero::zero();

			match value {
				StakingBalance::RingBalance(r) => {
					// Only active normal ring can be unbond:
					// `active_ring = active_normal_ring + active_deposit_ring`
					let active_normal_ring = *active_ring - *active_deposit_ring;
					unbond_ring = r.min(active_normal_ring);

					if !unbond_ring.is_zero() {
						*active_ring -= unbond_ring;

						// Avoid there being a dust balance left in the staking system.
						if (*active_ring < T::RingCurrency::minimum_balance())
							&& (*active_kton < T::KtonCurrency::minimum_balance()) {
							unbond_ring += *active_ring;
							unbond_kton += *active_kton;

							*active_ring = Zero::zero();
							*active_kton = Zero::zero();
						}

						ring_staking_lock.unbondings.push(Unbonding {
							amount: unbond_ring,
							until: now + T::BondingDurationInBlockNumber::get(),
						});

						<RingPool<T>>::mutate(|r| *r -= unbond_ring);
						Self::deposit_event(RawEvent::UnbondRing(unbond_ring, now));

						if !unbond_kton.is_zero() {
							kton_staking_lock.unbondings.push(Unbonding {
								amount: unbond_kton,
								until: now + T::BondingDurationInBlockNumber::get(),
							});

							<KtonPool<T>>::mutate(|k| *k -= unbond_kton);
							Self::deposit_event(RawEvent::UnbondKton(unbond_kton, now));
						}
					}
				},
				StakingBalance::KtonBalance(k) => {
					unbond_kton = k.min(*active_kton);

					if !unbond_kton.is_zero() {
						*active_kton -= unbond_kton;

						// Avoid there being a dust balance left in the staking system.
						if (*active_kton < T::KtonCurrency::minimum_balance())
							&& (*active_ring < T::RingCurrency::minimum_balance()) {
							unbond_kton += *active_kton;
							unbond_ring += *active_ring;

							*active_kton = Zero::zero();
							*active_ring = Zero::zero();
						}

						kton_staking_lock.unbondings.push(Unbonding {
							amount: unbond_kton,
							until: now + T::BondingDurationInBlockNumber::get(),
						});


						<KtonPool<T>>::mutate(|k| *k -= unbond_kton);
						Self::deposit_event(RawEvent::UnbondKton(unbond_kton, now));

						if !unbond_ring.is_zero() {
							ring_staking_lock.unbondings.push(Unbonding {
								amount: unbond_ring,
								until: now + T::BondingDurationInBlockNumber::get(),
							});

							<RingPool<T>>::mutate(|k| *k -= unbond_ring);
							Self::deposit_event(RawEvent::UnbondRing(unbond_ring, now));
						}
					}
				},
			}

			Self::update_ledger(&controller, &mut ledger);

			let StakingLedger {
				stash,
				active_ring,
				active_kton,
				..
			} = ledger;

			// All bonded *RING* and *KTON* is withdrawing, then remove Ledger to save storage
			if active_ring.is_zero() && active_kton.is_zero() {
				// TODO:
				//
				// `OnKilledAccount` would be a method to collect the locks.
				//
				// These locks are still in the system, and should be removed after 14 days
				//
				// There two situations should be considered after the 14 days
				// - the user never bond again, so the locks should be released.
				// - the user is bonded again in the 14 days, so the after 14 days
				//   the lock should not be removed
				//
				// If the locks are not deleted, this lock will waste the storage in the future
				// blocks.
				//
				// T::Ring::remove_lock(STAKING_ID, &stash);
				// T::Kton::remove_lock(STAKING_ID, &stash);
				Self::kill_stash(&stash)?;
			}
		}

		/// Stash accounts can get their ring back after the depositing time exceeded,
		/// and the ring getting back is still in staking status.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - One storage read.
		/// - One storage write.
		/// - Writes are limited to the `origin` account key.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn claim_mature_deposits(origin) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::clear_mature_deposits(Self::ledger(&controller).ok_or(<Error<T>>::NotController)?);

			<Ledger<T>>::insert(controller, ledger);
		}

		/// Claim deposits while the depositing time has not been exceeded, the ring
		/// will not be slashed, but the account is required to pay KTON as punish.
		///
		/// Refer to https://talk.darwinia.network/topics/55
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - One storage read.
		/// - One storage write.
		/// - Writes are limited to the `origin` account key.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(750_000)]
		fn try_claim_deposits_with_punish(origin, expire_time: MomentT<T>) {
			let controller = ensure_signed(origin)?;
			let mut ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let now = T::Time::now();

			if expire_time <= now {
				return Ok(());
			}

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
					let plan_duration_in_months = {
						let plan_duration_in_ts = (item.expire_time - item.start_time).saturated_into::<Moment>();
						plan_duration_in_ts / MONTH_IN_MILLISECONDS
					};
					let passed_duration_in_months = {
						let passed_duration_in_ts = (now - item.start_time).saturated_into::<Moment>();
						passed_duration_in_ts / MONTH_IN_MILLISECONDS
					};

					(
						inflation::compute_kton_return::<T>(item.value, plan_duration_in_months)
						-
						inflation::compute_kton_return::<T>(item.value, passed_duration_in_months)
					).max(1.into()) * 3.into()
				};

				// check total free balance and locked one
				// strict on punishing in kton
				if T::KtonCurrency::free_balance(stash)
					.checked_sub(&kton_slash)
					.and_then(|new_balance| {
						T::KtonCurrency::ensure_can_withdraw(
							stash,
							kton_slash,
							WithdrawReason::Transfer.into(),
							new_balance
						).ok()
					})
					.is_some()
				{
					*active_deposit_ring = active_deposit_ring.saturating_sub(item.value);

					let imbalance = T::KtonCurrency::slash(stash, kton_slash).0;
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
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let stash = &ledger.stash;

			<Nominators<T>>::remove(stash);
			<Validators<T>>::insert(stash, prefs);
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
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let stash = &ledger.stash;

			ensure!(!targets.is_empty(), <Error<T>>::EmptyTargets);

			let targets = targets.into_iter()
				.take(MAX_NOMINATIONS)
				.map(|t| T::Lookup::lookup(t))
				.collect::<Result<Vec<T::AccountId>, _>>()?;
			let nominations = Nominations {
				targets,
				// initial nominations are considered submitted at era 0. See `Nominations` doc
				submitted_in: Self::current_era().unwrap_or(0),
				suppressed: false,
			};

			<Validators<T>>::remove(stash);
			<Nominators<T>>::insert(stash, &nominations);
		}

		/// Declare no desire to either validate or nominate.
		///
		/// Effects will be felt at the beginning of the next era.
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
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;

			Self::chill_stash(&ledger.stash);
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
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
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
			let old_controller = Self::bonded(&stash).ok_or(<Error<T>>::NotStash)?;
			let controller = T::Lookup::lookup(controller)?;

			ensure!(!<Ledger<T>>::contains_key(&controller), <Error<T>>::AlreadyPaired);

			if controller != old_controller {
				<Bonded<T>>::insert(&stash, &controller);
				if let Some(l) = <Ledger<T>>::take(&old_controller) {
					<Ledger<T>>::insert(&controller, l);
				}
			}
		}

		// ----- Root calls.

		/// The ideal number of validators.
		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		fn set_validator_count(origin, #[compact] new: u32) {
			ensure_root(origin)?;
			ValidatorCount::put(new);
		}

		/// Force there to be no new eras indefinitely.
		///
		/// # <weight>
		/// - No arguments.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
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
		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		fn force_new_era(origin) {
			ensure_root(origin)?;
			ForceEra::put(Forcing::ForceNew);
		}

		/// Set the validators who cannot be slashed (if any).
		#[weight = SimpleDispatchInfo::FixedNormal(10_000)]
		fn set_invulnerables(origin, validators: Vec<T::AccountId>) {
			ensure_root(origin)?;
			<Invulnerables<T>>::put(validators);
		}

		/// Force a current staker to become completely unstaked, immediately.
		#[weight = SimpleDispatchInfo::FixedNormal(10_000)]
		fn force_unstake(origin, stash: T::AccountId) {
			ensure_root(origin)?;

			// remove all staking-related information.
			Self::kill_stash(&stash)?;

			// remove the lock.
			T::RingCurrency::remove_lock(STAKING_ID, &stash);
			T::KtonCurrency::remove_lock(STAKING_ID, &stash);
		}

		/// Force there to be a new era at the end of sessions indefinitely.
		///
		/// # <weight>
		/// - One storage write
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		fn force_new_era_always(origin) {
			ensure_root(origin)?;
			ForceEra::put(Forcing::ForceAlways);
		}

		/// Cancel enactment of a deferred slash. Can be called by either the root origin or
		/// the `T::SlashCancelOrigin`.
		/// passing the era and indices of the slashes for that era to kill.
		///
		/// # <weight>
		/// - One storage write.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
		fn cancel_deferred_slash(origin, era: EraIndex, slash_indices: Vec<u32>) {
			T::SlashCancelOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(ensure_root)?;

			ensure!(!slash_indices.is_empty(), <Error<T>>::EmptyTargets);
			ensure!(Self::is_sorted_and_unique(&slash_indices), <Error<T>>::NotSortedAndUnique);

			let mut unapplied = <Self as Store>::UnappliedSlashes::get(&era);
			let last_item = slash_indices[slash_indices.len() - 1];
			ensure!((last_item as usize) < unapplied.len(), <Error<T>>::InvalidSlashIndex);

			for (removed, index) in slash_indices.into_iter().enumerate() {
				let index = (index as usize) - removed;
				unapplied.remove(index);
			}

			<Self as Store>::UnappliedSlashes::insert(&era, &unapplied);
		}

		/// Make one nominator's payout for one era.
		///
		/// - `who` is the controller account of the nominator to pay out.
		/// - `era` may not be lower than one following the most recently paid era. If it is higher,
		///   then it indicates an instruction to skip the payout of all previous eras.
		/// - `validators` is the list of all validators that `who` had exposure to during `era`.
		///   If it is incomplete, then less than the full reward will be paid out.
		///   It must not exceed `MAX_NOMINATIONS`.
		///
		/// WARNING: once an era is payed for a validator such validator can't claim the payout of
		/// previous era.
		///
		/// WARNING: Incorrect arguments here can result in loss of payout. Be very careful.
		///
		/// # <weight>
		/// - Number of storage read of `O(validators)`; `validators` is the argument of the call,
		///   and is bounded by `MAX_NOMINATIONS`.
		/// - Each storage read is `O(N)` size and decode complexity; `N` is the  maximum
		///   nominations that can be given to a single validator.
		/// - Computation complexity: `O(MAX_NOMINATIONS * logN)`; `MAX_NOMINATIONS` is the
		///   maximum number of validators that may be nominated by a single nominator, it is
		///   bounded only economically (all nominators are required to place a minimum stake).
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn payout_nominator(origin, era: EraIndex, validators: Vec<(T::AccountId, u32)>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_payout_nominator(who, era, validators)
		}

		/// Make one validator's payout for one era.
		///
		/// - `who` is the controller account of the validator to pay out.
		/// - `era` may not be lower than one following the most recently paid era. If it is higher,
		///   then it indicates an instruction to skip the payout of all previous eras.
		///
		/// WARNING: once an era is payed for a validator such validator can't claim the payout of
		/// previous era.
		///
		/// WARNING: Incorrect arguments here can result in loss of payout. Be very careful.
		///
		/// # <weight>
		/// - Time complexity: O(1).
		/// - Contains a limited number of reads and writes.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn payout_validator(origin, era: EraIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_payout_validator(who, era)
		}

		/// Set history_depth value.
		///
		/// Origin must be root.
		#[weight = SimpleDispatchInfo::FixedOperational(500_000)]
		fn set_history_depth(origin, #[compact] new_history_depth: EraIndex) {
			ensure_root(origin)?;
			if let Some(current_era) = Self::current_era() {
				HistoryDepth::mutate(|history_depth| {
					let last_kept = current_era.checked_sub(*history_depth).unwrap_or(0);
					let new_last_kept = current_era.checked_sub(new_history_depth).unwrap_or(0);
					for era_index in last_kept..new_last_kept {
						Self::clear_era_information(era_index);
					}
					*history_depth = new_history_depth
				})
			}
		}

		/// Remove all data structure concerning a staker/stash once its balance is zero.
		/// This is essentially equivalent to `withdraw_unbonded` except it can be called by anyone
		/// and the target `stash` must have no funds left.
		///
		/// This can be called from any origin.
		///
		/// - `stash`: The stash account to reap. Its balance must be zero.
		fn reap_stash(_origin, stash: T::AccountId) {
			ensure!(T::RingCurrency::total_balance(&stash).is_zero(), <Error<T>>::FundedTarget);
			ensure!(T::KtonCurrency::total_balance(&stash).is_zero(), <Error<T>>::FundedTarget);

			Self::kill_stash(&stash)?;
			T::RingCurrency::remove_lock(STAKING_ID, &stash);
			T::KtonCurrency::remove_lock(STAKING_ID, &stash);
		}
	}
}

impl<T: Trait> Module<T> {
	// PUBLIC IMMUTABLES

	// power is a mixture of ring and kton
	// For *RING* power = ring_ratio * POWER_COUNT / 2
	// For *KTON* power = kton_ratio * POWER_COUNT / 2
	pub fn currency_to_power<S: TryInto<Balance>>(active: S, pool: S) -> Power {
		(Perquintill::from_rational_approximation(
			active.saturated_into::<Balance>(),
			pool.saturated_into::<Balance>().max(1),
		) * (T::TotalPower::get() as Balance / 2)) as _
	}

	/// The total power that can be slashed from a stash account as of right now.
	pub fn power_of(stash: &T::AccountId) -> Power {
		Self::bonded(stash)
			.and_then(Self::ledger)
			.map(|l| {
				Self::currency_to_power::<_>(l.active_ring, Self::ring_pool())
					+ Self::currency_to_power::<_>(l.active_kton, Self::kton_pool())
			})
			.unwrap_or_default()
	}

	pub fn stake_of(stash: &T::AccountId) -> (RingBalance<T>, KtonBalance<T>) {
		Self::bonded(stash)
			.and_then(Self::ledger)
			.map(|l| (l.active_ring, l.active_kton))
			.unwrap_or_default()
	}

	/// Check that list is sorted and has no duplicates.
	fn is_sorted_and_unique(list: &Vec<u32>) -> bool {
		list.windows(2).all(|w| w[0] < w[1])
	}

	// Update the ledger while bonding ring and compute the kton should return.
	fn bond_ring(
		stash: &T::AccountId,
		controller: &T::AccountId,
		value: RingBalance<T>,
		promise_month: Moment,
		mut ledger: StakingLedgerT<T>,
	) -> (MomentT<T>, MomentT<T>) {
		let start_time = T::Time::now();
		let mut expire_time = start_time;

		ledger.active_ring = ledger.active_ring.saturating_add(value);
		// if stash promise to a extra-lock
		// there will be extra reward, kton, which
		// can also be use to stake.
		if promise_month >= 3 {
			expire_time += <MomentT<T>>::saturated_from((promise_month * MONTH_IN_MILLISECONDS).into());
			ledger.active_deposit_ring += value;
			// for now, kton_return is free
			// mint kton
			let kton_return = inflation::compute_kton_return::<T>(value, promise_month);
			let kton_positive_imbalance = T::KtonCurrency::deposit_creating(&stash, kton_return);

			T::KtonReward::on_unbalanced(kton_positive_imbalance);
			ledger.deposit_items.push(TimeDepositItem {
				value,
				start_time,
				expire_time,
			});
		}

		Self::update_ledger(&controller, &mut ledger);

		(start_time, expire_time)
	}

	fn bond_ring_for_deposit_redeem(
		controller: &T::AccountId,
		value: RingBalance<T>,
		start: Moment,
		promise_month: Moment,
		mut ledger: StakingLedgerT<T>,
	) -> (MomentT<T>, MomentT<T>) {
		let start_time = <MomentT<T>>::saturated_from(start.into());
		let expire_time = start_time + <MomentT<T>>::saturated_from((promise_month * MONTH_IN_MILLISECONDS).into());

		ledger.active_ring = ledger.active_ring.saturating_add(value);
		ledger.active_deposit_ring = ledger.active_deposit_ring.saturating_add(value);
		ledger.deposit_items.push(TimeDepositItem {
			value,
			start_time,
			expire_time,
		});

		Self::update_ledger(&controller, &mut ledger);

		(start_time, expire_time)
	}

	// Update the ledger while bonding controller with kton.
	fn bond_kton(controller: &T::AccountId, value: KtonBalance<T>, mut ledger: StakingLedgerT<T>) {
		ledger.active_kton += value;
		Self::update_ledger(&controller, &mut ledger);
	}

	// TODO: doc
	pub fn clear_mature_deposits(mut ledger: StakingLedgerT<T>) -> StakingLedgerT<T> {
		let now = T::Time::now();
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

	// MUTABLES (DANGEROUS)

	fn do_payout_nominator(who: T::AccountId, era: EraIndex, validators: Vec<(T::AccountId, u32)>) -> DispatchResult {
		// validators len must not exceed `MAX_NOMINATIONS` to avoid querying more validator
		// exposure than necessary.
		ensure!(
			validators.len() <= MAX_NOMINATIONS,
			<Error<T>>::InvalidNumberOfNominations
		);

		// Note: if era has no reward to be claimed, era may be future. better not to update
		// `nominator_ledger.last_reward` in this case.
		let era_payout = <ErasValidatorReward<T>>::get(&era).ok_or_else(|| <Error<T>>::InvalidEraToReward)?;

		let mut nominator_ledger = <Ledger<T>>::get(&who).ok_or_else(|| <Error<T>>::NotController)?;

		if nominator_ledger
			.last_reward
			.map(|last_reward| last_reward >= era)
			.unwrap_or(false)
		{
			return Err(<Error<T>>::InvalidEraToReward.into());
		}

		nominator_ledger.last_reward = Some(era);
		<Ledger<T>>::insert(&who, &nominator_ledger);

		let mut reward = Perbill::zero();
		let era_reward_points = <ErasRewardPoints<T>>::get(&era);

		for (validator, nominator_index) in validators.into_iter() {
			let commission = Self::eras_validator_prefs(&era, &validator).commission;
			let validator_exposure = <ErasStakersClipped<T>>::get(&era, &validator);

			if let Some(nominator_exposure) = validator_exposure.others.get(nominator_index as usize) {
				if nominator_exposure.who != nominator_ledger.stash {
					continue;
				}

				let nominator_exposure_part =
					Perbill::from_rational_approximation(nominator_exposure.power, validator_exposure.total_power);
				let validator_point = era_reward_points
					.individual
					.get(&validator)
					.map(|points| *points)
					.unwrap_or_else(|| Zero::zero());
				let validator_point_part =
					Perbill::from_rational_approximation(validator_point, era_reward_points.total);
				reward = reward.saturating_add(
					validator_point_part
						.saturating_mul(Perbill::one().saturating_sub(commission))
						.saturating_mul(nominator_exposure_part),
				);
			}
		}

		if let Some(imbalance) = Self::make_payout(&nominator_ledger.stash, reward * era_payout) {
			Self::deposit_event(RawEvent::Reward(who, imbalance.peek()));
		}

		Ok(())
	}

	fn do_payout_validator(who: T::AccountId, era: EraIndex) -> DispatchResult {
		// Note: if era has no reward to be claimed, era may be future. better not to update
		// `ledger.last_reward` in this case.
		let era_payout = <ErasValidatorReward<T>>::get(&era).ok_or_else(|| <Error<T>>::InvalidEraToReward)?;

		let mut ledger = <Ledger<T>>::get(&who).ok_or_else(|| <Error<T>>::NotController)?;
		if ledger
			.last_reward
			.map(|last_reward| last_reward >= era)
			.unwrap_or(false)
		{
			return Err(<Error<T>>::InvalidEraToReward.into());
		}

		ledger.last_reward = Some(era);
		<Ledger<T>>::insert(&who, &ledger);

		let era_reward_points = <ErasRewardPoints<T>>::get(&era);
		let commission = Self::eras_validator_prefs(&era, &ledger.stash).commission;
		let exposure = <ErasStakers<T>>::get(&era, &ledger.stash);

		let exposure_part = Perbill::from_rational_approximation(exposure.own_power, exposure.total_power);
		let validator_point = era_reward_points
			.individual
			.get(&ledger.stash)
			.map(|points| *points)
			.unwrap_or_else(|| Zero::zero());
		let validator_point_part = Perbill::from_rational_approximation(validator_point, era_reward_points.total);
		let reward = validator_point_part.saturating_mul(
			commission.saturating_add(Perbill::one().saturating_sub(commission).saturating_mul(exposure_part)),
		);

		if let Some(imbalance) = Self::make_payout(&ledger.stash, reward * era_payout) {
			Self::deposit_event(RawEvent::Reward(who, imbalance.peek()));
		}

		Ok(())
	}

	/// Update the ledger for a controller. This will also update the stash lock. The lock will
	/// will lock the entire funds except paying for further transactions.
	fn update_ledger(controller: &T::AccountId, ledger: &mut StakingLedgerT<T>) {
		let StakingLedger {
			active_ring,
			active_kton,
			ring_staking_lock,
			kton_staking_lock,
			..
		} = ledger;

		if *active_ring != ring_staking_lock.staking_amount {
			ring_staking_lock.staking_amount = *active_ring;

			T::RingCurrency::set_lock(
				STAKING_ID,
				&ledger.stash,
				LockFor::Staking(ledger.ring_staking_lock.clone()),
				WithdrawReasons::all(),
			);
		}

		if *active_kton != kton_staking_lock.staking_amount {
			kton_staking_lock.staking_amount = *active_kton;

			T::KtonCurrency::set_lock(
				STAKING_ID,
				&ledger.stash,
				LockFor::Staking(ledger.kton_staking_lock.clone()),
				WithdrawReasons::all(),
			);
		}

		<Ledger<T>>::insert(controller, ledger);
	}

	/// Chill a stash account.
	fn chill_stash(stash: &T::AccountId) {
		<Validators<T>>::remove(stash);
		<Nominators<T>>::remove(stash);
	}

	/// Actually make a payment to a staker. This uses the currency's reward function
	/// to pay the right payee for the given staker account.
	fn make_payout(stash: &T::AccountId, amount: RingBalance<T>) -> Option<RingPositiveImbalance<T>> {
		let dest = Self::payee(stash);
		match dest {
			RewardDestination::Controller => Self::bonded(stash)
				.and_then(|controller| T::RingCurrency::deposit_into_existing(&controller, amount).ok()),
			RewardDestination::Stash => T::RingCurrency::deposit_into_existing(stash, amount).ok(),
			// TODO month
			RewardDestination::Staked { promise_month: _ } => Self::bonded(stash)
				.and_then(|c| Self::ledger(&c).map(|l| (c, l)))
				.and_then(|(c, mut l)| {
					l.active_ring += amount;

					let r = T::RingCurrency::deposit_into_existing(stash, amount).ok();
					Self::update_ledger(&c, &mut l);
					r
				}),
		}
	}

	/// Plan a new session potentially trigger a new era.
	fn new_session(session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		if let Some(current_era) = Self::current_era() {
			// Initial era has been set.

			let current_era_start_session_index = Self::eras_start_session_index(current_era).unwrap_or_else(|| {
				frame_support::print("Error: start_session_index must be set for current_era");
				0
			});

			let era_length = session_index.checked_sub(current_era_start_session_index).unwrap_or(0); // Must never happen.

			match ForceEra::get() {
				Forcing::ForceNew => ForceEra::kill(),
				Forcing::ForceAlways => (),
				Forcing::NotForcing if era_length >= T::SessionsPerEra::get() => (),
				_ => return None,
			}
			Self::new_era(session_index)
		} else {
			// Set initial era
			Self::new_era(session_index)
		}
	}

	/// Start a session potentially starting an era.
	fn start_session(start_session: SessionIndex) {
		let next_active_era = Self::active_era().map(|e| e.index + 1).unwrap_or(0);
		if let Some(next_active_era_start_session_index) = Self::eras_start_session_index(next_active_era) {
			if next_active_era_start_session_index == start_session {
				Self::start_era(start_session);
			} else if next_active_era_start_session_index < start_session {
				// This arm should never happen, but better handle it than to stall the
				// staking pallet.
				frame_support::print("Warning: A session appears to have been skipped.");
				Self::start_era(start_session);
			}
		}
	}

	/// End a session potentially ending an era.
	fn end_session(session_index: SessionIndex) {
		if let Some(active_era) = Self::active_era() {
			let next_active_era_start_session_index = Self::eras_start_session_index(active_era.index + 1)
				.unwrap_or_else(|| {
					frame_support::print("Error: start_session_index must be set for active_era + 1");
					0
				});

			if next_active_era_start_session_index == session_index + 1 {
				Self::end_era(active_era, session_index);
			}
		}
	}

	/// * Increment `active_era.index`,
	/// * reset `active_era.start`,
	/// * update `BondedEras` and apply slashes.
	fn start_era(start_session: SessionIndex) {
		let active_era = <ActiveEra<T>>::mutate(|active_era| {
			let new_index = active_era.as_ref().map(|info| info.index + 1).unwrap_or(0);
			*active_era = Some(ActiveEraInfo {
				index: new_index,
				// Set new active era start in next `on_finalize`. To guarantee usage of `Time`
				start: None,
			});
			new_index
		});

		let bonding_duration = T::BondingDurationInEra::get();

		BondedEras::mutate(|bonded| {
			bonded.push((active_era, start_session));

			if active_era > bonding_duration {
				let first_kept = active_era - bonding_duration;

				// prune out everything that's from before the first-kept index.
				let n_to_prune = bonded.iter().take_while(|&&(era_idx, _)| era_idx < first_kept).count();

				// kill slashing metadata.
				for (pruned_era, _) in bonded.drain(..n_to_prune) {
					slashing::clear_era_metadata::<T>(pruned_era);
				}

				if let Some(&(_, first_session)) = bonded.first() {
					T::SessionInterface::prune_historical_up_to(first_session);
				}
			}
		});

		Self::apply_unapplied_slashes(active_era);
	}

	/// Compute payout for era.
	fn end_era(active_era: ActiveEraInfo<MomentT<T>>, _session_index: SessionIndex) {
		// Note: active_era_start can be None if end era is called during genesis config.
		if let Some(active_era_start) = active_era.start {
			let now = T::Time::now();

			let living_time = Self::living_time();
			let era_duration = now - active_era_start;

			let (total_payout, _max_payout) = inflation::compute_total_payout::<T>(
				era_duration,
				Self::living_time(),
				T::Cap::get().saturating_sub(T::RingCurrency::total_issuance()),
				PayoutFraction::get(),
			);

			<LivingTime<T>>::put(living_time + era_duration);
			// Set ending era reward.
			<ErasValidatorReward<T>>::insert(&active_era.index, total_payout);
		}
	}

	/// Plan a new era. Return the potential new staking set.
	fn new_era(start_session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		// Increment or set current era.
		let current_era = CurrentEra::mutate(|s| {
			*s = Some(s.map(|s| s + 1).unwrap_or(0));
			s.unwrap()
		});
		ErasStartSessionIndex::insert(&current_era, &start_session_index);

		// Clean old era information.
		if let Some(old_era) = current_era.checked_sub(Self::history_depth() + 1) {
			Self::clear_era_information(old_era);
		}

		// Set staking information for new era.
		let maybe_new_validators = Self::select_validators(current_era);

		maybe_new_validators
	}

	/// Clear all era information for given era.
	fn clear_era_information(era_index: EraIndex) {
		<ErasStakers<T>>::remove_prefix(era_index);
		<ErasStakersClipped<T>>::remove_prefix(era_index);
		<ErasValidatorPrefs<T>>::remove_prefix(era_index);
		<ErasValidatorReward<T>>::remove(era_index);
		<ErasRewardPoints<T>>::remove(era_index);
		ErasTotalStake::remove(era_index);
		ErasStartSessionIndex::remove(era_index);
	}

	/// Apply previously-unapplied slashes on the beginning of a new era, after a delay.
	fn apply_unapplied_slashes(active_era: EraIndex) {
		let slash_defer_duration = T::SlashDeferDuration::get();
		<Self as Store>::EarliestUnappliedSlash::mutate(|earliest| {
			if let Some(ref mut earliest) = earliest {
				let keep_from = active_era.saturating_sub(slash_defer_duration);
				for era in (*earliest)..keep_from {
					let era_slashes = <Self as Store>::UnappliedSlashes::take(&era);
					for slash in era_slashes {
						slashing::apply_slash::<T>(slash);
					}
				}

				*earliest = (*earliest).max(keep_from)
			}
		})
	}

	/// Select a new validator set from the assembled stakers and their role preferences, and store
	/// staking information for the new current era.
	///
	/// Fill the storages `ErasStakers`, `ErasStakersClipped`, `ErasValidatorPrefs` and
	/// `ErasTotalStake` for current era.
	///
	/// Returns a set of newly selected _stash_ IDs.
	///
	/// Assumes storage is coherent with the declaration.
	fn select_validators(current_era: EraIndex) -> Option<Vec<T::AccountId>> {
		let mut all_nominators: Vec<(T::AccountId, Vec<T::AccountId>)> = vec![];
		let mut all_validators_and_prefs = BTreeMap::new();
		let mut all_validators = Vec::new();
		for (validator, preference) in <Validators<T>>::iter() {
			let self_vote = (validator.clone(), vec![validator.clone()]);
			all_nominators.push(self_vote);
			all_validators_and_prefs.insert(validator.clone(), preference);
			all_validators.push(validator);
		}
		let nominator_votes = <Nominators<T>>::iter().map(|(nominator, nominations)| {
			let Nominations {
				submitted_in,
				mut targets,
				suppressed: _,
			} = nominations;

			// Filter out nomination targets which were nominated before the most recent
			// non-zero slash.
			targets.retain(|stash| {
				<Self as Store>::SlashingSpans::get(&stash)
					.map_or(true, |spans| submitted_in >= spans.last_nonzero_slash())
			});

			(nominator, targets)
		});
		all_nominators.extend(nominator_votes);

		let maybe_phragmen_result = darwinia_phragmen::elect::<_, Perbill, _>(
			Self::validator_count() as usize,
			Self::minimum_validator_count().max(1) as usize,
			all_validators,
			all_nominators,
			Self::power_of,
		);

		if let Some(phragmen_result) = maybe_phragmen_result {
			let elected_stashes = phragmen_result
				.winners
				.into_iter()
				.map(|(s, _)| s)
				.collect::<Vec<T::AccountId>>();
			let assignments = phragmen_result.assignments;

			let to_power = |v: Votes| v as Power;

			let supports = darwinia_phragmen::build_support_map::<_, Perbill, _, _, _, _>(
				&elected_stashes,
				&assignments,
				Self::power_of,
				Self::stake_of,
			);

			// Populate stakers information and figure out the total staked.
			let mut total_staked = 0;
			for (c, s) in supports.into_iter() {
				// build `struct exposure` from `support`
				let mut own_ring_balance: RingBalance<T> = Zero::zero();
				let mut own_kton_balance: KtonBalance<T> = Zero::zero();
				let mut own_power = 0;
				let mut total_power = 0;
				let mut others = vec![];
				s.voters.into_iter().for_each(
					|PhragmenStakedAssignment {
					     account_id,
					     ring_balance,
					     kton_balance,
					     votes,
					 }| {
						let power = to_power(votes);
						if c == account_id {
							own_ring_balance = own_ring_balance.saturating_add(ring_balance);
							own_kton_balance = own_kton_balance.saturating_add(kton_balance);
							own_power += power;
						} else {
							others.push(IndividualExposure {
								who: account_id,
								ring_balance,
								kton_balance,
								power,
							});
						}
						total_power += power;
					},
				);

				total_staked += total_power;

				let exposure = Exposure {
					own_ring_balance,
					own_kton_balance,
					own_power,
					total_power,
					others,
				};

				<ErasStakers<T>>::insert(&current_era, &c, &exposure);

				let mut exposure_clipped = exposure;
				let clipped_max_len = T::MaxNominatorRewardedPerValidator::get() as usize;
				if exposure_clipped.others.len() > clipped_max_len {
					exposure_clipped
						.others
						.sort_unstable_by(|a, b| a.power.cmp(&b.power).reverse());
					exposure_clipped.others.truncate(clipped_max_len);
				}

				<ErasStakersClipped<T>>::insert(&current_era, &c, exposure_clipped);
			}

			// Insert current era staking informations
			ErasTotalStake::insert(&current_era, total_staked);
			let default_pref = ValidatorPrefs::default();
			for stash in &elected_stashes {
				let pref = all_validators_and_prefs.get(stash).unwrap_or(&default_pref); // Must never happen, but better to be safe.
				<ErasValidatorPrefs<T>>::insert(&current_era, stash, pref);
			}

			// In order to keep the property required by `n_session_ending`
			// that we must return the new validator set even if it's the same as the old,
			// as long as any underlying economic conditions have changed, we don't attempt
			// to do any optimization where we compare against the prior set.
			Some(elected_stashes)
		} else {
			// There were not enough candidates for even our minimal level of functionality.
			// This is bad.
			// We should probably disable all functionality except for block production
			// and let the chain keep producing blocks until we can decide on a sufficiently
			// substantial set.
			// TODO: #2494
			None
		}
	}

	/// Remove all associated data of a stash account from the staking system.
	///
	/// Assumes storage is upgraded before calling.
	///
	/// This is called:
	/// - after a `withdraw_unbond()` call that frees all of a stash's bonded balance.
	/// - through `reap_stash()` if the balance has fallen to zero (through slashing).
	fn kill_stash(stash: &T::AccountId) -> DispatchResult {
		let controller = <Bonded<T>>::take(stash).ok_or(<Error<T>>::NotStash)?;
		<Ledger<T>>::remove(&controller);

		<Payee<T>>::remove(stash);
		<Validators<T>>::remove(stash);
		<Nominators<T>>::remove(stash);

		slashing::clear_stash_metadata::<T>(stash);

		<frame_system::Module<T>>::dec_ref(stash);

		Ok(())
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
	pub fn reward_by_ids(validators_points: impl IntoIterator<Item = (T::AccountId, u32)>) {
		if let Some(active_era) = Self::active_era() {
			<ErasRewardPoints<T>>::mutate(active_era.index, |era_rewards| {
				for (validator, points) in validators_points.into_iter() {
					*era_rewards.individual.entry(validator).or_default() += points;
					era_rewards.total += points;
				}
			});
		}
	}

	/// Ensures that at the end of the current session there will be a new era.
	fn ensure_new_era() {
		match ForceEra::get() {
			Forcing::ForceAlways | Forcing::ForceNew => (),
			_ => ForceEra::put(Forcing::ForceNew),
		}
	}
}

impl<T: Trait> pallet_session::SessionManager<T::AccountId> for Module<T> {
	fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		Self::new_session(new_index)
	}
	fn end_session(end_index: SessionIndex) {
		Self::end_session(end_index)
	}
	fn start_session(start_index: SessionIndex) {
		Self::start_session(start_index)
	}
}

/// This implementation has the same constrains as the implementation of
/// `pallet_session::SessionManager`.
impl<T: Trait>
	pallet_session::historical::SessionManager<T::AccountId, Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>>
	for Module<T>
{
	fn new_session(
		new_index: SessionIndex,
	) -> Option<Vec<(T::AccountId, Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>)>> {
		<Self as pallet_session::SessionManager<_>>::new_session(new_index).map(|validators| {
			let current_era = Self::current_era()
				// Must be some as a new era has been created.
				.unwrap_or(0);

			validators
				.into_iter()
				.map(|v| {
					let exposure = Self::eras_stakers(current_era, &v);
					(v, exposure)
				})
				.collect()
		})
	}
	fn start_session(start_index: SessionIndex) {
		<Self as pallet_session::SessionManager<_>>::start_session(start_index)
	}
	fn end_session(end_index: SessionIndex) {
		<Self as pallet_session::SessionManager<_>>::end_session(end_index)
	}
}

/// Add reward points to block authors:
/// * 20 points to the block producer for producing a (non-uncle) block in the relay chain,
/// * 2 points to the block producer for each reference to a previously unreferenced uncle, and
/// * 1 point to the producer of each referenced uncle block.
impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Module<T>
where
	T: Trait + pallet_authorship::Trait + pallet_session::Trait,
{
	fn note_author(author: T::AccountId) {
		Self::reward_by_ids(vec![(author, 20)]);
	}
	fn note_uncle(author: T::AccountId, _age: T::BlockNumber) {
		Self::reward_by_ids(vec![(<pallet_authorship::Module<T>>::author(), 2), (author, 1)]);
	}
}

/// A `Convert` implementation that finds the stash of the given controller account,
/// if any.
pub struct StashOf<T>(PhantomData<T>);

impl<T: Trait> Convert<T::AccountId, Option<T::AccountId>> for StashOf<T> {
	fn convert(controller: T::AccountId) -> Option<T::AccountId> {
		<Module<T>>::ledger(&controller).map(|l| l.stash)
	}
}

/// A typed conversion from stash account ID to the active exposure of nominators
/// on that account.
///
/// Active exposure is the exposure of the validator set currently validating, i.e. in
/// `active_era`. It can differ from the latest planned exposure in `current_era`.
pub struct ExposureOf<T>(PhantomData<T>);

impl<T: Trait> Convert<T::AccountId, Option<Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>>> for ExposureOf<T> {
	fn convert(validator: T::AccountId) -> Option<Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>> {
		if let Some(active_era) = <Module<T>>::active_era() {
			Some(<Module<T>>::eras_stakers(active_era.index, &validator))
		} else {
			None
		}
	}
}

/// This is intended to be used with `FilterHistoricalOffences`.
impl<T: Trait> OnOffenceHandler<T::AccountId, pallet_session::historical::IdentificationTuple<T>> for Module<T>
where
	T: pallet_session::Trait<ValidatorId = AccountId<T>>,
	T: pallet_session::historical::Trait<
		FullIdentification = Exposure<<T as frame_system::Trait>::AccountId, RingBalance<T>, KtonBalance<T>>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<<T as frame_system::Trait>::AccountId>,
	T::SessionManager: pallet_session::SessionManager<<T as frame_system::Trait>::AccountId>,
	T::ValidatorIdOf: Convert<<T as frame_system::Trait>::AccountId, Option<<T as frame_system::Trait>::AccountId>>,
{
	fn on_offence(
		offenders: &[OffenceDetails<T::AccountId, pallet_session::historical::IdentificationTuple<T>>],
		slash_fraction: &[Perbill],
		slash_session: SessionIndex,
	) {
		let reward_proportion = SlashRewardFraction::get();

		let active_era = {
			let active_era = Self::active_era();
			if active_era.is_none() {
				return;
			}
			active_era.unwrap().index
		};
		let active_era_start_session_index = Self::eras_start_session_index(active_era).unwrap_or_else(|| {
			frame_support::print("Error: start_session_index must be set for current_era");
			0
		});

		let window_start = active_era.saturating_sub(T::BondingDurationInEra::get());

		// fast path for active-era report - most likely.
		// `slash_session` cannot be in a future active era. It must be in `active_era` or before.
		let slash_era = if slash_session >= active_era_start_session_index {
			active_era
		} else {
			// reverse because it's more likely to find reports from recent eras.
			match BondedEras::get()
				.iter()
				.rev()
				.filter(|&&(_, ref sesh)| sesh <= &slash_session)
				.next()
			{
				None => return, // before bonding period. defensive - should be filtered out.
				Some(&(ref slash_era, _)) => *slash_era,
			}
		};

		<Self as Store>::EarliestUnappliedSlash::mutate(|earliest| {
			if earliest.is_none() {
				*earliest = Some(active_era)
			}
		});

		let slash_defer_duration = T::SlashDeferDuration::get();

		for (details, slash_fraction) in offenders.iter().zip(slash_fraction) {
			let stash = &details.offender.0;
			let exposure = &details.offender.1;

			// Skip if the validator is invulnerable.
			if Self::invulnerables().contains(stash) {
				continue;
			}

			let unapplied = slashing::compute_slash::<T>(slashing::SlashParams {
				stash,
				slash: *slash_fraction,
				exposure,
				slash_era,
				window_start,
				now: active_era,
				reward_proportion,
			});

			if let Some(mut unapplied) = unapplied {
				unapplied.reporters = details.reporters.clone();
				if slash_defer_duration == 0 {
					// apply right away.
					slashing::apply_slash::<T>(unapplied);
				} else {
					// defer to end of some `slash_defer_duration` from now.
					<Self as Store>::UnappliedSlashes::mutate(active_era, move |for_later| for_later.push(unapplied));
				}
			}
		}
	}
}

/// Filter historical offences out and only allow those from the bonding period.
pub struct FilterHistoricalOffences<T, R> {
	_inner: PhantomData<(T, R)>,
}

impl<T, Reporter, Offender, R, O> ReportOffence<Reporter, Offender, O> for FilterHistoricalOffences<Module<T>, R>
where
	T: Trait,
	R: ReportOffence<Reporter, Offender, O>,
	O: Offence<Offender>,
{
	fn report_offence(reporters: Vec<Reporter>, offence: O) -> Result<(), OffenceError> {
		// disallow any slashing from before the current bonding period.
		let offence_session = offence.session_index();
		let bonded_eras = BondedEras::get();

		if bonded_eras
			.first()
			.filter(|(_, start)| offence_session >= *start)
			.is_some()
		{
			R::report_offence(reporters, offence)
		} else {
			<Module<T>>::deposit_event(RawEvent::OldSlashingReportDiscarded(offence_session));
			Ok(())
		}
	}
}

impl<T: Trait> OnDepositRedeem<T::AccountId> for Module<T> {
	type Balance = RingBalance<T>;
	type Moment = Moment;

	fn on_deposit_redeem(
		start_time: Self::Moment,
		months: Self::Moment,
		amount: Self::Balance,
		stash: &T::AccountId,
	) -> DispatchResult {
		let controller = Self::bonded(&stash).ok_or(<Error<T>>::NotStash)?;
		let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;

		// The timestamp unit is different between Ethereum and Darwinia, converting from seconds to milliseconds
		let start_time = start_time * 1000;
		let promise_month = months.min(36);

		//		let stash_balance = T::Ring::free_balance(&stash);

		// TODO: Lock but no kton reward because this is a deposit redeem
		//		let extra = extra.min(r);

		let redeemed_positive_imbalance_ring = T::RingCurrency::deposit_into_existing(&stash, amount)?;

		T::RingReward::on_unbalanced(redeemed_positive_imbalance_ring);

		let (start_time, expire_time) =
			Self::bond_ring_for_deposit_redeem(&controller, amount, start_time, promise_month, ledger);

		<RingPool<T>>::mutate(|r| *r += amount);
		// TODO: Should we deposit an different event?
		<Module<T>>::deposit_event(RawEvent::BondRing(amount, start_time, expire_time));

		Ok(())
	}
}
