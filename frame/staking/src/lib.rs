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
//! The staking system in Substrate NPoS is designed to make the following possible:
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
//! use darwinia_staking::{self as staking};
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
//! ### Slot Stake
//!
//! The term [`SlotStake`](./struct.Module.html#method.slot_stake) will be used throughout this
//! section. It refers to a value calculated at the end of each era, containing the _minimum value
//! at stake among all validators._ Note that a validator's value at stake might be a combination
//! of the validator's own stake and the votes it received. See [`Exposure`](./struct.Exposure.html)
//! for more details.
//!
//! ### Reward Calculation
//!
//! Validators and nominators are rewarded at the end of each era. The total reward of an era is
//! calculated using the era duration and the staking rate (the total amount of tokens staked by
//! nominators and validators, divided by the total token supply). It aims to incentivise toward a
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
#[cfg(test)]
mod tests;

mod inflation;
mod migration;
mod slashing;

mod types {
	use sp_std::vec::Vec;

	use crate::*;

	/// Counter for the number of eras that have passed.
	pub type EraIndex = u32;
	/// Counter for the number of "reward" points earned by a given validator.
	pub type Points = u32;

	/// Balance of an account.
	pub type Balance = u128;
	/// Type used for expressing timestamp.
	pub type Moment = Timestamp;

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

	pub type Rewards<T> = (RingBalance<T>, Vec<NominatorReward<AccountId<T>, RingBalance<T>>>);

	/// A timestamp: milliseconds since the unix epoch.
	/// `u64` is enough to represent a duration of half a billion years, when the
	/// time scale is milliseconds.
	type Timestamp = u64;

	type AccountId<T> = <T as system::Trait>::AccountId;
	type BlockNumber<T> = <T as system::Trait>::BlockNumber;
	type TimeT<T> = <T as Trait>::Time;
	type RingCurrency<T> = <T as Trait>::RingCurrency;
	type KtonCurrency<T> = <T as Trait>::KtonCurrency;
}

pub use types::{EraIndex, Points};

use codec::{Decode, Encode, HasCompact};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	traits::{Currency, Get, Imbalance, OnFreeBalanceZero, OnUnbalanced, Time},
	weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use pallet_session::{historical::OnSessionEnding, SelectInitialValidators};
use sp_runtime::{
	traits::{
		CheckedSub, Convert, EnsureOrigin, One, SaturatedConversion, Saturating, SimpleArithmetic, StaticLookup, Zero,
	},
	DispatchResult, Perbill, Perquintill, RuntimeDebug,
};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_staking::{
	offence::{Offence, OffenceDetails, OnOffenceHandler, ReportOffence},
	SessionIndex,
};
use sp_std::{borrow::ToOwned, convert::TryInto, marker::PhantomData, vec, vec::Vec};

use darwinia_phragmen::{PhragmenStakedAssignment, Power, Votes};
use darwinia_support::{
	LockIdentifier, LockableCurrency, NormalLock, OnDepositRedeem, OnUnbalancedKton, StakingLock, WithdrawLock,
	WithdrawReason, WithdrawReasons,
};
use types::*;

const DEFAULT_MINIMUM_VALIDATOR_COUNT: u32 = 4;
const MONTH_IN_MINUTES: Moment = 30 * 24 * 60;
const MONTH_IN_MILLISECONDS: Moment = MONTH_IN_MINUTES * 60 * 1000;
const MAX_NOMINATIONS: usize = 16;
const MAX_UNLOCKING_CHUNKS: usize = 32;
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
	fn add_points_to_index(&mut self, index: u32, points: u32) {
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

/// To unify *Ring* and *Kton* balances.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum StakingBalance<RingBalance, KtonBalance>
where
	RingBalance: HasCompact,
	KtonBalance: HasCompact,
{
	All,
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
}

impl<AccountId, RingBalance, KtonBalance, BlockNumber, Timestamp>
	StakingLedger<AccountId, RingBalance, KtonBalance, BlockNumber, Timestamp>
where
	RingBalance: SimpleArithmetic + Saturating + Copy,
	KtonBalance: SimpleArithmetic + Saturating + Copy,
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
			let slash_from_active_ring = (*slash_ring).min(*active_ring);
			let slash_from_active_kton = (*slash_kton).min(*active_kton);

			if !slash_from_active_ring.is_zero() {
				let normal_ring = *active_ring - *active_deposit_ring;
				if normal_ring < *slash_ring {
					let mut slash_deposit_ring = *slash_ring - (*active_ring - *active_deposit_ring);
					*active_deposit_ring -= slash_deposit_ring;

					deposit_item.drain_filter(|item| {
						if ts >= item.expire_time {
							true
						} else {
							if slash_deposit_ring.is_zero() {
								false
							} else {
								if slash_deposit_ring > item.value {
									slash_deposit_ring -= item.value;
									true
								} else {
									item.value -= sp_std::mem::replace(&mut slash_deposit_ring, Zero::zero());
									false
								}
							}
						}
					});
				}
				*active_ring -= slash_from_active_ring;
				*slash_ring -= slash_from_active_ring;
			}

			if !slash_from_active_kton.is_zero() {
				*active_kton -= slash_from_active_kton;
				*slash_kton -= slash_from_active_kton;
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

/// A record of the nominations made by a specific account.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Nominations<AccountId> {
	/// The targets of nomination.
	pub targets: Vec<AccountId>,
	/// The era the nominations were submitted.
	pub submitted_in: EraIndex,
	/// Whether the nominations have been suppressed.
	pub suppressed: bool,
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
	#[codec(compact)]
	pub total_ring_balance: RingBalance,
	#[codec(compact)]
	pub total_kton_balance: KtonBalance,
	pub total_power: Power,
	/// The portions of nominators stashes that are exposed.
	pub others: Vec<IndividualExposure<AccountId, RingBalance, KtonBalance>>,
}

/// A typed conversion from stash account ID to the current exposure of nominators
/// on that account.
pub struct ExposureOf<T>(PhantomData<T>);

impl<T: Trait> Convert<T::AccountId, Option<Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>>> for ExposureOf<T> {
	fn convert(validator: T::AccountId) -> Option<Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>> {
		Some(<Module<T>>::stakers(&validator))
	}
}

// FIXME: RingBalance: HasCompact
// TODO: doc
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ValidatorReward<AccountId, RingBalance> {
	who: AccountId,
	#[codec(compact)]
	amount: RingBalance,
	nominators_reward: Vec<NominatorReward<AccountId, RingBalance>>,
}

// TODO: doc
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct NominatorReward<AccountId, RingBalance> {
	who: AccountId,
	#[codec(compact)]
	amount: RingBalance,
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

/// Means for interacting with a specialized version of the `session` trait.
///
/// This is needed because `Staking` sets the `ValidatorIdOf` of the `pallet_session::Trait`
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
	T: pallet_session::Trait<ValidatorId = <T as system::Trait>::AccountId>,
	T: pallet_session::historical::Trait<
		FullIdentification = Exposure<<T as system::Trait>::AccountId, RingBalance<T>, KtonBalance<T>>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<<T as system::Trait>::AccountId>,
	T::OnSessionEnding: pallet_session::OnSessionEnding<<T as system::Trait>::AccountId>,
	T::SelectInitialValidators: pallet_session::SelectInitialValidators<<T as system::Trait>::AccountId>,
	T::ValidatorIdOf: Convert<<T as system::Trait>::AccountId, Option<<T as system::Trait>::AccountId>>,
{
	fn disable_validator(validator: &<T as system::Trait>::AccountId) -> Result<bool, ()> {
		<pallet_session::Module<T>>::disable(validator)
	}

	fn validators() -> Vec<<T as system::Trait>::AccountId> {
		<pallet_session::Module<T>>::validators()
	}

	fn prune_historical_up_to(up_to: SessionIndex) {
		<pallet_session::historical::Module<T>>::prune_up_to(up_to);
	}
}

pub trait Trait: system::Trait {
	/// Time used for computing era duration.
	type Time: Time;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

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

	/// The *RING* balance.
	type RingCurrency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	/// Tokens have been minted and are unused for validator-reward.
	type RingRewardRemainder: OnUnbalanced<RingNegativeImbalance<Self>>;
	/// Handler for the unbalanced *RING* reduction when slashing a staker.
	type RingSlash: OnUnbalanced<RingNegativeImbalance<Self>>;
	/// Handler for the unbalanced *RING* increment when rewarding a staker.
	type RingReward: OnUnbalanced<RingPositiveImbalance<Self>>;

	/// The *KTON* balance
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

	// TODO: doc
	type GenesisTime: Get<MomentT<Self>>;
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
		pub Ledger get(fn ledger): map T::AccountId => Option<StakingLedgerT<T>>;

		/// Where the reward payment should be made. Keyed by stash.
		pub Payee get(fn payee): map T::AccountId => RewardDestination;

		/// The map from (wannabe) validator stash key to the preferences of that validator.
		pub Validators get(fn validators): linked_map T::AccountId => ValidatorPrefs;

		/// The map from nominator stash key to the set of stash keys of all validators to nominate.
		///
		/// NOTE: is private so that we can ensure upgraded before all typical accesses.
		/// Direct storage APIs can still bypass this protection.
		Nominators get(fn nominators): linked_map T::AccountId => Option<Nominations<T::AccountId>>;

		/// Nominators for a particular account that is in action right now. You can't iterate
		/// through validators here, but you can find them in the Session module.
		///
		/// This is keyed by the stash account.
		pub Stakers get(fn stakers): map T::AccountId => Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>;

		/// The currently elected validator set keyed by stash account ID.
		pub CurrentElected get(fn current_elected): Vec<T::AccountId>;

		/// The current era index.
		pub CurrentEra get(fn current_era) config(): EraIndex;

		/// The start of the current era.
		pub CurrentEraStart get(fn current_era_start): MomentT<T>;

		/// The session index at which the current era started.
		pub CurrentEraStartSessionIndex get(fn current_era_start_session_index): SessionIndex;

		/// Rewards for the current era. Using indices of current elected set.
		CurrentEraPointsEarned get(fn current_era_reward): EraPoints;

		/// The amount of balance actively at stake for each validator slot, currently.
		///
		/// This is used to derive rewards and punishments.
		pub SlotStake get(fn slot_stake) build(|config: &GenesisConfig<T>| {
			config
				.stakers
				.iter()
				.map(|&(_, _, r, _)| <Module<T>>::currency_to_power::<_>(r, <Module<T>>::ring_pool()))
				.min()
				.unwrap_or_default()
		}): Power;

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
		pub UnappliedSlashes: map EraIndex => Vec<UnappliedSlash<T::AccountId, RingBalance<T>, KtonBalance<T>>>;

		/// Total *Ring* in pool.
		pub RingPool get(fn ring_pool): RingBalance<T>;
		/// Total *Kton* in pool.
		pub KtonPool get(fn kton_pool): KtonBalance<T>;

		/// The percentage of the total payout that is distributed to validators and nominators
		///
		/// The reset might go to Treasury or something else.
		pub PayoutFraction get(fn payout_fraction) config(): Perbill;

		/// A mapping from still-bonded eras to the first session index of that era.
		BondedEras: Vec<(EraIndex, SessionIndex)>;

		/// All slashing events on validators, mapped by era to the highest slash proportion
		/// and slash value of the era.
		ValidatorSlashInEra:
		double_map EraIndex, twox_128(T::AccountId) => Option<(Perbill, slashing::RKT<T>)>;

		/// All slashing events on nominators, mapped by era to the highest slash value of the era.
		NominatorSlashInEra: double_map EraIndex, twox_128(T::AccountId) => Option<slashing::RKT<T>>;

		/// Slashing spans for stash accounts.
		SlashingSpans: map T::AccountId => Option<slashing::SlashingSpans>;

		/// Records information about the maximum slash of a stash within a slashing span,
		/// as well as how much reward has been paid out.
		SpanSlash: map (T::AccountId, slashing::SpanIndex) => slashing::SpanRecord<RingBalance<T>, KtonBalance<T>>;

		/// The earliest era for which we have a pending, unapplied slash.
		EarliestUnappliedSlash: Option<EraIndex>;

		/// The version of storage for upgrade.
		StorageVersion: u32;
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

			StorageVersion::put(migration::CURRENT_VERSION);
		});
	}
}

decl_event!(
	pub enum Event<T>
	where
		<T as system::Trait>::AccountId,
		<T as system::Trait>::BlockNumber,
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

		/// All validators have been rewarded by the first balance; the second is the remainder
		/// from the maximum amount of reward; the third is validator and nominators' reward.
		Reward(RingBalance, RingBalance, Vec<ValidatorReward<AccountId, RingBalance>>),

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
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Number of sessions per era.
		const SessionsPerEra: SessionIndex = T::SessionsPerEra::get();

		/// Number of eras that staked funds must remain bonded for.
		const BondingDurationInEra: EraIndex = T::BondingDurationInEra::get();
		/// Number of eras that staked funds must remain bonded for.
		const BondingDurationInBlockNumber: T::BlockNumber = T::BondingDurationInBlockNumber::get();

		// TODO: doc
		const Cap: RingBalance<T> = T::Cap::get();

		// TODO: doc
		const TotalPower: Power = T::TotalPower::get();

		// TODO: doc
		const GenesisTime: MomentT<T> = T::GenesisTime::get();

		type Error = Error<T>;

		fn deposit_event() = default;

		fn on_initialize() {
			Self::ensure_storage_upgraded();
		}

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
			value: StakingBalanceT<T>,
			payee: RewardDestination,
			promise_month: Moment
		) {
			let stash = ensure_signed(origin)?;
			ensure!(!<Bonded<T>>::exists(&stash), <Error<T>>::AlreadyBonded);

			let controller = T::Lookup::lookup(controller)?;
			ensure!(!<Ledger<T>>::exists(&controller), <Error<T>>::AlreadyPaired);

			let ledger = StakingLedger {
				stash: stash.clone(),
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

					let stash_balance = T::RingCurrency::free_balance(&stash);
					let value = r.min(stash_balance);
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

					let stash_balance = T::KtonCurrency::free_balance(&stash);
					let value = k.min(stash_balance);

					Self::bond_kton(&controller, value, ledger);

					<KtonPool<T>>::mutate(|k| *k += value);
					Self::deposit_event(RawEvent::BondKton(value));
				},
				_ => (),
			}

			// You're auto-bonded forever, here. We might improve this by only bonding when
			// you actually validate/nominate and remove once you unbond __everything__.
			<Bonded<T>>::insert(&stash, &controller);
			<Payee<T>>::insert(&stash, payee);
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
					let stash_balance = T::RingCurrency::free_balance(&stash);
					if let Some(extra) = stash_balance.checked_sub(&ledger.active_ring) {
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
					}
				},
				StakingBalance::KtonBalance(k) => {
					let stash_balance = T::KtonCurrency::free_balance(&stash);
					if let Some(extra) = stash_balance.checked_sub(&ledger.active_kton) {
						let extra = extra.min(k);

						Self::bond_kton(&controller, extra, ledger);

						<KtonPool<T>>::mutate(|k| *k += extra);
						Self::deposit_event(RawEvent::BondKton(extra));
					}
				},
				_ => (),
			}
		}

		// TODO: doc
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
		#[weight = SimpleDispatchInfo::FixedNormal(400_000)]
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
			let now = <system::Module<T>>::block_number();

			ring_staking_lock.shrink(now);
			kton_staking_lock.shrink(now);

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

			match value {
				StakingBalance::RingBalance(r) => {
					// Only active normal ring can be unbond:
					// `active_ring = active_normal_ring + active_deposit_ring`
					let active_normal_ring = *active_ring - *active_deposit_ring;
					let available_unbond_ring = r.min(active_normal_ring);

					if !available_unbond_ring.is_zero() {
						*active_ring -= available_unbond_ring;
						ring_staking_lock.unbondings.push(NormalLock {
							amount: available_unbond_ring,
							until: now + T::BondingDurationInBlockNumber::get(),
						});

						Self::update_ledger(&controller, &mut ledger, value);

						<RingPool<T>>::mutate(|r| *r -= available_unbond_ring);
						Self::deposit_event(RawEvent::UnbondRing(available_unbond_ring, now));
					}
				},
				StakingBalance::KtonBalance(k) => {
					let unbond_kton = k.min(*active_kton);

					if !unbond_kton.is_zero() {
						*active_kton -= unbond_kton;
						kton_staking_lock.unbondings.push(NormalLock {
							amount: unbond_kton,
							until: now + T::BondingDurationInBlockNumber::get(),
						});

						Self::update_ledger(&controller, &mut ledger, value);

						<KtonPool<T>>::mutate(|k| *k -= unbond_kton);
						Self::deposit_event(RawEvent::UnbondKton(unbond_kton, now));
					}
				},
				_ => (),
			}

			let StakingLedger {
				active_ring,
				active_kton,
				stash,
				..
			} = ledger;

			// all bonded rings and ktons is withdrawing, then remove Ledger to save storage
			if active_ring.is_zero() && active_kton.is_zero() {
				// TODO:
				// These locks are still in the system, and should be removed after 14 days
				//
				// There two situations should be considered after the 14 days
				// - the user never bond again, so the locks should be released.
				// - the user is bonded again in the 14 days, so the after 14 days
				//   the lock should not be removed
				//
				// If the locks are not deleted, this lock will wast the storage in the future
				// blocks.
				//
				// T::Ring::remove_lock(STAKING_ID, &stash);
				// T::Kton::remove_lock(STAKING_ID, &stash);
				Self::kill_stash(&stash);
			}
		}

		// TODO: doc
		fn claim_mature_deposits(origin) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::clear_mature_deposits(Self::ledger(&controller).ok_or(<Error<T>>::NotController)?);

			<Ledger<T>>::insert(controller, ledger);
		}

		// TODO: doc
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

					let (imbalance, _) = T::KtonCurrency::slash(stash, kton_slash);
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
			Self::ensure_storage_upgraded();

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
			Self::ensure_storage_upgraded();

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
				submitted_in: Self::current_era(),
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

			ensure!(!<Ledger<T>>::exists(&controller), <Error<T>>::AlreadyPaired);

			if controller != old_controller {
				<Bonded<T>>::insert(&stash, &controller);
				if let Some(l) = <Ledger<T>>::take(&old_controller) {
					<Ledger<T>>::insert(&controller, l);
				}
			}
		}

		// ----- Root calls.

		/// The ideal number of validators.
		#[weight = SimpleDispatchInfo::FreeOperational]
		fn set_validator_count(origin, #[compact] new: u32) {
			ensure_root(origin)?;
			ValidatorCount::put(new);
		}

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
			ForceEra::put(Forcing::ForceNew);
		}

		/// Set the validators who cannot be slashed (if any).
		#[weight = SimpleDispatchInfo::FreeOperational]
		fn set_invulnerables(origin, validators: Vec<T::AccountId>) {
			ensure_root(origin)?;
			<Invulnerables<T>>::put(validators);
		}

		/// Force a current staker to become completely unstaked, immediately.
		#[weight = SimpleDispatchInfo::FreeOperational]
		fn force_unstake(origin, stash: T::AccountId) {
			ensure_root(origin)?;

			// remove the lock.
			T::RingCurrency::remove_lock(STAKING_ID, &stash);
			T::KtonCurrency::remove_lock(STAKING_ID, &stash);

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

		/// Cancel enactment of a deferred slash. Can be called by either the root origin or
		/// the `T::SlashCancelOrigin`.
		/// passing the era and indices of the slashes for that era to kill.
		///
		/// # <weight>
		/// - One storage write.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FreeOperational]
		fn cancel_deferred_slash(origin, era: EraIndex, slash_indices: Vec<u32>) {
			T::SlashCancelOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(ensure_root)?;

			let mut slash_indices = slash_indices;
			slash_indices.sort_unstable();
			let mut unapplied = <Self as Store>::UnappliedSlashes::get(&era);

			for (removed, index) in slash_indices.into_iter().enumerate() {
				let index = index as usize;

				// if `index` is not duplicate, `removed` must be <= index.
				ensure!(removed <= index, <Error<T>>::DuplicateIndex);

				// all prior removals were from before this index, since the
				// list is sorted.
				let index = index - removed;
				ensure!(index < unapplied.len(), <Error<T>>::InvalidSlashIndex);

				unapplied.remove(index);
			}

			<Self as Store>::UnappliedSlashes::insert(&era, &unapplied);
		}
	}
}

impl<T: Trait> Module<T> {
	// PUBLIC IMMUTABLES

	// power is a mixture of ring and kton
	// power = ring_ratio * POWER_COUNT / 2 + kton_ratio * POWER_COUNT / 2
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

		Self::update_ledger(&controller, &mut ledger, StakingBalance::RingBalance(value));

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

		Self::update_ledger(&controller, &mut ledger, StakingBalance::RingBalance(value));

		(start_time, expire_time)
	}

	// Update the ledger while bonding controller with kton.
	fn bond_kton(controller: &T::AccountId, value: KtonBalance<T>, mut ledger: StakingLedgerT<T>) {
		ledger.active_kton += value;
		Self::update_ledger(&controller, &mut ledger, StakingBalance::KtonBalance(value));
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

	/// Update the ledger for a controller. This will also update the stash lock. The lock will
	/// will lock the entire funds except paying for further transactions.
	fn update_ledger(controller: &T::AccountId, ledger: &mut StakingLedgerT<T>, staking_balance: StakingBalanceT<T>) {
		match staking_balance {
			StakingBalance::RingBalance(_) => {
				ledger.ring_staking_lock.staking_amount = ledger.active_ring;

				T::RingCurrency::set_lock(
					STAKING_ID,
					&ledger.stash,
					WithdrawLock::WithStaking(ledger.ring_staking_lock.clone()),
					WithdrawReasons::all(),
				);
			}
			StakingBalance::KtonBalance(_) => {
				ledger.kton_staking_lock.staking_amount = ledger.active_kton;

				T::KtonCurrency::set_lock(
					STAKING_ID,
					&ledger.stash,
					WithdrawLock::WithStaking(ledger.kton_staking_lock.clone()),
					WithdrawReasons::all(),
				);
			}
			_ => {
				ledger.ring_staking_lock.staking_amount = ledger.active_ring;
				ledger.kton_staking_lock.staking_amount = ledger.active_kton;

				T::RingCurrency::set_lock(
					STAKING_ID,
					&ledger.stash,
					WithdrawLock::WithStaking(ledger.ring_staking_lock.clone()),
					WithdrawReasons::all(),
				);
				T::KtonCurrency::set_lock(
					STAKING_ID,
					&ledger.stash,
					WithdrawLock::WithStaking(ledger.kton_staking_lock.clone()),
					WithdrawReasons::all(),
				);
			}
		}

		<Ledger<T>>::insert(controller, ledger);
	}

	/// Chill a stash account.
	fn chill_stash(stash: &T::AccountId) {
		<Validators<T>>::remove(stash);
		<Nominators<T>>::remove(stash);
	}

	/// Ensures storage is upgraded to most recent necessary state.
	fn ensure_storage_upgraded() {
		migration::perform_migrations::<T>();
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
					Self::update_ledger(&c, &mut l, StakingBalance::RingBalance(amount));
					r
				}),
		}
	}

	/// Reward a given validator by a specific amount. Add the reward to the validator's, and its
	/// nominators' balance, pro-rata based on their exposure, after having removed the validator's
	/// pre-payout cut.
	fn reward_validator(stash: &T::AccountId, reward: RingBalance<T>) -> (RingPositiveImbalance<T>, Rewards<T>) {
		let off_the_table = Self::validators(stash).commission * reward;
		let reward = reward.saturating_sub(off_the_table);
		let mut imbalance = <RingPositiveImbalance<T>>::zero();
		let mut nominators_reward = vec![];
		let validator_cut = if reward.is_zero() {
			Zero::zero()
		} else {
			let exposure = Self::stakers(stash);
			let total = exposure.total_power.max(One::one());

			for i in &exposure.others {
				let per_u64 = Perbill::from_rational_approximation(i.power, total);
				let nominator_reward = per_u64 * reward;

				imbalance.maybe_subsume(Self::make_payout(&i.who, nominator_reward));
				nominators_reward.push(NominatorReward {
					who: i.who.to_owned(),
					amount: nominator_reward,
				});
			}

			let per_u64 = Perbill::from_rational_approximation(exposure.own_power, total);
			per_u64 * reward
		};
		let validator_reward = validator_cut + off_the_table;

		imbalance.maybe_subsume(Self::make_payout(stash, validator_reward));

		(imbalance, (validator_reward, nominators_reward))
	}

	/// Session has just ended. Provide the validator set for the next session if it's an era-end, along
	/// with the exposure of the prior validator set.
	fn new_session(
		session_index: SessionIndex,
	) -> Option<(
		Vec<T::AccountId>,
		Vec<(T::AccountId, Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>)>,
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
		let previous_era_start = <CurrentEraStart<T>>::mutate(|v| sp_std::mem::replace(v, now));
		let era_duration = now - previous_era_start;
		if !era_duration.is_zero() {
			let validators = Self::current_elected();
			let (total_payout, max_payout) = inflation::compute_total_payout::<T>(
				era_duration,
				T::Time::now() - T::GenesisTime::get(),
				T::Cap::get() - T::RingCurrency::total_issuance(),
				PayoutFraction::get(),
			);
			let mut total_imbalance = <RingPositiveImbalance<T>>::zero();
			let mut validators_reward = vec![];

			for (v, p) in validators.iter().zip(points.individual.into_iter()) {
				if p != 0 {
					let reward = Perbill::from_rational_approximation(p, points.total) * total_payout;
					let (imbalance, (validator_reward, nominators_reward)) = Self::reward_validator(v, reward);

					total_imbalance.subsume(imbalance);
					validators_reward.push(ValidatorReward {
						who: v.to_owned(),
						amount: validator_reward,
						nominators_reward,
					});
				}
			}

			// assert!(total_imbalance.peek() == total_payout)
			let total_payout = total_imbalance.peek();
			let rest = max_payout.saturating_sub(total_payout);

			Self::deposit_event(RawEvent::Reward(total_payout, rest, validators_reward));

			T::RingReward::on_unbalanced(total_imbalance);
			T::RingRewardRemainder::on_unbalanced(T::RingCurrency::issue(rest));
		}

		// Increment current era.
		let current_era = CurrentEra::mutate(|s| {
			*s += 1;
			*s
		});

		CurrentEraStartSessionIndex::mutate(|v| {
			*v = start_session_index;
		});
		let bonding_duration_in_era = T::BondingDurationInEra::get();

		BondedEras::mutate(|bonded| {
			bonded.push((current_era, start_session_index));

			if current_era > bonding_duration_in_era {
				let first_kept = current_era - bonding_duration_in_era;

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

		// Reassign all Stakers.
		let (_slot_stake, maybe_new_validators) = Self::select_validators();
		Self::apply_unapplied_slashes(current_era);

		maybe_new_validators
	}

	/// Apply previously-unapplied slashes on the beginning of a new era, after a delay.
	fn apply_unapplied_slashes(current_era: EraIndex) {
		let slash_defer_duration = T::SlashDeferDuration::get();
		<Self as Store>::EarliestUnappliedSlash::mutate(|earliest| {
			if let Some(ref mut earliest) = earliest {
				let keep_from = current_era.saturating_sub(slash_defer_duration);
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

	/// Select a new validator set from the assembled stakers and their role preferences.
	///
	/// Returns the new `SlotStake` value and a set of newly selected _stash_ IDs.
	///
	/// Assumes storage is coherent with the declaration.
	fn select_validators() -> (Power, Option<Vec<T::AccountId>>) {
		let mut all_nominators: Vec<(T::AccountId, Vec<T::AccountId>)> = vec![];
		let all_validator_candidates_iter = <Validators<T>>::enumerate();
		let all_validators = all_validator_candidates_iter
			.map(|(who, _pref)| {
				let self_vote = (who.clone(), vec![who.clone()]);
				all_nominators.push(self_vote);
				who
			})
			.collect::<Vec<T::AccountId>>();
		let nominator_votes = <Nominators<T>>::enumerate().map(|(nominator, nominations)| {
			let Nominations {
				submitted_in,
				mut targets,
				suppressed: _,
			} = nominations;

			// Filter out nomination targets which were nominated before the most recent
			// slashing span.
			targets.retain(|stash| {
				<Self as Store>::SlashingSpans::get(&stash).map_or(true, |spans| submitted_in >= spans.last_start())
			});

			(nominator, targets)
		});

		all_nominators.extend(nominator_votes);

		let maybe_phragmen_result = darwinia_phragmen::elect::<_, _>(
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

			let to_votes = |p: Power| p as Votes;
			let to_power = |v: Votes| v as Power;

			let mut supports = darwinia_phragmen::build_support_map::<_, _, _, _, _>(
				&elected_stashes,
				&assignments,
				Self::power_of,
				Self::stake_of,
			);

			if cfg!(feature = "equalize") {
				let mut staked_assignments: Vec<(
					T::AccountId,
					Vec<PhragmenStakedAssignment<T::AccountId, RingBalance<T>, KtonBalance<T>>>,
				)> = Vec::with_capacity(assignments.len());
				for (n, assignment) in assignments.iter() {
					let mut staked_assignment: Vec<
						PhragmenStakedAssignment<T::AccountId, RingBalance<T>, KtonBalance<T>>,
					> = Vec::with_capacity(assignment.len());

					// If this is a self vote, then we don't need to equalise it at all. While the
					// staking system does not allow nomination and validation at the same time,
					// this must always be 100% support.
					if assignment.len() == 1 && assignment[0].0 == *n {
						continue;
					}
					for (c, per_thing) in assignment.iter() {
						let nominator_stake = to_votes(Self::power_of(n));
						let (ring_balance, kton_balance) = {
							let (r, k) = Self::stake_of(n);
							(*per_thing * r, *per_thing * k)
						};
						let other_stake = *per_thing * nominator_stake;
						staked_assignment.push(PhragmenStakedAssignment {
							account_id: c.clone(),
							votes: other_stake,
							ring_balance,
							kton_balance,
						});
					}
					staked_assignments.push((n.clone(), staked_assignment));
				}

				let tolerance: Votes = 0;
				let iterations = 2_usize;
				darwinia_phragmen::equalize::<_, _, _, _>(
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
			let mut slot_stake = T::TotalPower::get();
			for (c, s) in supports.into_iter() {
				// build `struct exposure` from `support`
				let exposure = Exposure {
					own_ring_balance: s.own_ring_balance,
					own_kton_balance: s.own_kton_balance,
					own_power: to_power(s.own_votes),
					total_ring_balance: s.total_ring_balance,
					total_kton_balance: s.total_kton_balance,
					total_power: to_power(s.total_votes),
					others: s
						.others
						.into_iter()
						.map(|assignment| IndividualExposure {
							who: assignment.account_id,
							ring_balance: assignment.ring_balance,
							kton_balance: assignment.kton_balance,
							power: to_power(assignment.votes),
						})
						.collect::<Vec<IndividualExposure<_, _, _>>>(),
				};
				if exposure.total_power < slot_stake {
					slot_stake = exposure.total_power;
				}

				<Stakers<T>>::insert(&c, exposure.clone());
			}

			// Update slot stake.
			SlotStake::put(&slot_stake);
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
	/// Assumes storage is upgraded before calling.
	///
	/// This is called :
	/// - Immediately when an account's balance falls below existential deposit.
	/// - after a `withdraw_unbond()` call that frees all of a stash's bonded balance.
	fn kill_stash(stash: &T::AccountId) {
		if let Some(controller) = <Bonded<T>>::take(stash) {
			<Ledger<T>>::remove(&controller);
		}

		<Payee<T>>::remove(stash);
		<Validators<T>>::remove(stash);
		<Nominators<T>>::remove(stash);

		slashing::clear_stash_metadata::<T>(stash);
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
		CurrentEraPointsEarned::mutate(|rewards| {
			let current_elected = Self::current_elected();
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
	pub fn reward_by_indices(validators_points: impl IntoIterator<Item = (u32, u32)>) {
		// TODO: This can be optimised once #3302 is implemented.
		let current_elected_len = Self::current_elected().len() as u32;

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

impl<T: Trait> pallet_session::OnSessionEnding<T::AccountId> for Module<T> {
	fn on_session_ending(_ending: SessionIndex, start_session: SessionIndex) -> Option<Vec<T::AccountId>> {
		Self::ensure_storage_upgraded();
		Self::new_session(start_session - 1).map(|(new, _old)| new)
	}
}

impl<T: Trait> OnSessionEnding<T::AccountId, Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>> for Module<T> {
	fn on_session_ending(
		_ending: SessionIndex,
		start_session: SessionIndex,
	) -> Option<(
		Vec<T::AccountId>,
		Vec<(T::AccountId, Exposure<T::AccountId, RingBalance<T>, KtonBalance<T>>)>,
	)> {
		Self::ensure_storage_upgraded();
		Self::new_session(start_session - 1)
	}
}

impl<T: Trait> OnFreeBalanceZero<T::AccountId> for Module<T> {
	fn on_free_balance_zero(stash: &T::AccountId) {
		Self::ensure_storage_upgraded();
		Self::kill_stash(stash);
	}
}

/// Add reward points to block authors:
/// * 20 points to the block producer for producing a (non-uncle) block in the relay chain,
/// * 2 points to the block producer for each reference to a previously unreferenced uncle, and
/// * 1 point to the producer of each referenced uncle block.
impl<T: Trait + pallet_authorship::Trait> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Module<T> {
	fn note_author(author: T::AccountId) {
		Self::reward_by_ids(vec![(author, 20)]);
	}
	fn note_uncle(author: T::AccountId, _age: T::BlockNumber) {
		Self::reward_by_ids(vec![(<pallet_authorship::Module<T>>::author(), 2), (author, 1)])
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

impl<T: Trait> SelectInitialValidators<T::AccountId> for Module<T> {
	fn select_initial_validators() -> Option<Vec<T::AccountId>> {
		Self::select_validators().1
	}
}

/// This is intended to be used with `FilterHistoricalOffences`.
impl<T: Trait> OnOffenceHandler<T::AccountId, pallet_session::historical::IdentificationTuple<T>> for Module<T>
where
	T: pallet_session::Trait<ValidatorId = <T as system::Trait>::AccountId>,
	T: pallet_session::historical::Trait<
		FullIdentification = Exposure<<T as system::Trait>::AccountId, RingBalance<T>, KtonBalance<T>>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<<T as system::Trait>::AccountId>,
	T::OnSessionEnding: pallet_session::OnSessionEnding<<T as system::Trait>::AccountId>,
	T::SelectInitialValidators: pallet_session::SelectInitialValidators<<T as system::Trait>::AccountId>,
	T::ValidatorIdOf: Convert<<T as system::Trait>::AccountId, Option<<T as system::Trait>::AccountId>>,
{
	fn on_offence(
		offenders: &[OffenceDetails<T::AccountId, pallet_session::historical::IdentificationTuple<T>>],
		slash_fraction: &[Perbill],
		slash_session: SessionIndex,
	) {
		Self::ensure_storage_upgraded();

		let reward_proportion = SlashRewardFraction::get();
		let era_now = Self::current_era();
		let window_start = era_now.saturating_sub(T::BondingDurationInEra::get());
		let current_era_start_session = CurrentEraStartSessionIndex::get();
		// fast path for current-era report - most likely.
		let slash_era = if slash_session >= current_era_start_session {
			era_now
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
				*earliest = Some(era_now)
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
				now: era_now,
				reward_proportion,
			});

			if let Some(mut unapplied) = unapplied {
				unapplied.reporters = details.reporters.clone();
				if slash_defer_duration == 0 {
					// apply right away.
					slashing::apply_slash::<T>(unapplied);
				} else {
					// defer to end of some `slash_defer_duration` from now.
					<Self as Store>::UnappliedSlashes::mutate(era_now, move |for_later| for_later.push(unapplied));
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
	fn report_offence(reporters: Vec<Reporter>, offence: O) {
		<Module<T>>::ensure_storage_upgraded();

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
			<Module<T>>::deposit_event(RawEvent::OldSlashingReportDiscarded(offence_session))
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

		// TODO: Issue #169, checking the timestamp unit difference between Ethereum and Darwinia
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
