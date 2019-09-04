#![feature(prelude_import)]
#![no_std]
#![feature(drain_filter)]
#![recursion_limit = "128"]
#[prelude_import]
use ::std::prelude::v1::*;
#[macro_use]
extern crate std as std;
mod phragmen {
    //! Rust implementation of the Phragmén election algorithm.
    use crate::{ExpoMap, IndividualExpo, RawAssignment, Trait, ValidatorPrefs};
    use primitives::{traits::Zero, PerU128};
    use rstd::{collections::btree_map::BTreeMap, vec::Vec};
    type Fraction = PerU128;
    /// Wrapper around the type used as the _safe_ wrapper around a `balance`.
    pub type ExtendedBalance = u128;
    const SCALE_FACTOR: ExtendedBalance = u32::max_value() as ExtendedBalance + 1;
    /// These are used to expose a fixed accuracy to the caller function. The bigger they are,
    /// the more accurate we get, but the more likely it is for us to overflow. The case of overflow
    /// is handled but accuracy will be lost. 32 or 16 are reasonable values.
    pub const ACCURACY: ExtendedBalance = u32::max_value() as ExtendedBalance + 1;
    /// Wrapper around validation candidates some metadata.
    pub struct Candidate<AccountId> {
        /// The validator's account
        pub who: AccountId,
        /// Intermediary value used to sort candidates.
        pub score: Fraction,
        /// Accumulator of the stake of this candidate based on received votes.
        approval_stake: ExtendedBalance,
        /// Flag for being elected.
        elected: bool,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<AccountId: ::std::clone::Clone> ::std::clone::Clone for Candidate<AccountId> {
        #[inline]
        fn clone(&self) -> Candidate<AccountId> {
            match *self {
                Candidate {
                    who: ref __self_0_0,
                    score: ref __self_0_1,
                    approval_stake: ref __self_0_2,
                    elected: ref __self_0_3,
                } => Candidate {
                    who: ::std::clone::Clone::clone(&(*__self_0_0)),
                    score: ::std::clone::Clone::clone(&(*__self_0_1)),
                    approval_stake: ::std::clone::Clone::clone(&(*__self_0_2)),
                    elected: ::std::clone::Clone::clone(&(*__self_0_3)),
                },
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<AccountId: ::std::default::Default> ::std::default::Default for Candidate<AccountId> {
        #[inline]
        fn default() -> Candidate<AccountId> {
            Candidate {
                who: ::std::default::Default::default(),
                score: ::std::default::Default::default(),
                approval_stake: ::std::default::Default::default(),
                elected: ::std::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<AccountId: ::std::fmt::Debug> ::std::fmt::Debug for Candidate<AccountId> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                Candidate {
                    who: ref __self_0_0,
                    score: ref __self_0_1,
                    approval_stake: ref __self_0_2,
                    elected: ref __self_0_3,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Candidate");
                    let _ = debug_trait_builder.field("who", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("score", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("approval_stake", &&(*__self_0_2));
                    let _ = debug_trait_builder.field("elected", &&(*__self_0_3));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    /// Wrapper around the nomination info of a single nominator for a group of validators.
    pub struct Nominator<AccountId> {
        /// The nominator's account.
        who: AccountId,
        /// List of validators proposed by this nominator.
        edges: Vec<Edge<AccountId>>,
        /// the stake amount proposed by the nominator as a part of the vote.
        budget: ExtendedBalance,
        /// Incremented each time a nominee that this nominator voted for has been elected.
        load: Fraction,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<AccountId: ::std::clone::Clone> ::std::clone::Clone for Nominator<AccountId> {
        #[inline]
        fn clone(&self) -> Nominator<AccountId> {
            match *self {
                Nominator {
                    who: ref __self_0_0,
                    edges: ref __self_0_1,
                    budget: ref __self_0_2,
                    load: ref __self_0_3,
                } => Nominator {
                    who: ::std::clone::Clone::clone(&(*__self_0_0)),
                    edges: ::std::clone::Clone::clone(&(*__self_0_1)),
                    budget: ::std::clone::Clone::clone(&(*__self_0_2)),
                    load: ::std::clone::Clone::clone(&(*__self_0_3)),
                },
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<AccountId: ::std::default::Default> ::std::default::Default for Nominator<AccountId> {
        #[inline]
        fn default() -> Nominator<AccountId> {
            Nominator {
                who: ::std::default::Default::default(),
                edges: ::std::default::Default::default(),
                budget: ::std::default::Default::default(),
                load: ::std::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<AccountId: ::std::fmt::Debug> ::std::fmt::Debug for Nominator<AccountId> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                Nominator {
                    who: ref __self_0_0,
                    edges: ref __self_0_1,
                    budget: ref __self_0_2,
                    load: ref __self_0_3,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Nominator");
                    let _ = debug_trait_builder.field("who", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("edges", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("budget", &&(*__self_0_2));
                    let _ = debug_trait_builder.field("load", &&(*__self_0_3));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    /// Wrapper around a nominator vote and the load of that vote.
    pub struct Edge<AccountId> {
        /// Account being voted for
        who: AccountId,
        /// Load of this vote.
        load: Fraction,
        /// Equal to `edge.load / nom.load`. Stored only to be used with post-processing.
        ratio: ExtendedBalance,
        /// Index of the candidate stored in the 'candidates' vector.
        candidate_index: usize,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<AccountId: ::std::clone::Clone> ::std::clone::Clone for Edge<AccountId> {
        #[inline]
        fn clone(&self) -> Edge<AccountId> {
            match *self {
                Edge {
                    who: ref __self_0_0,
                    load: ref __self_0_1,
                    ratio: ref __self_0_2,
                    candidate_index: ref __self_0_3,
                } => Edge {
                    who: ::std::clone::Clone::clone(&(*__self_0_0)),
                    load: ::std::clone::Clone::clone(&(*__self_0_1)),
                    ratio: ::std::clone::Clone::clone(&(*__self_0_2)),
                    candidate_index: ::std::clone::Clone::clone(&(*__self_0_3)),
                },
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<AccountId: ::std::default::Default> ::std::default::Default for Edge<AccountId> {
        #[inline]
        fn default() -> Edge<AccountId> {
            Edge {
                who: ::std::default::Default::default(),
                load: ::std::default::Default::default(),
                ratio: ::std::default::Default::default(),
                candidate_index: ::std::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<AccountId: ::std::fmt::Debug> ::std::fmt::Debug for Edge<AccountId> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                Edge {
                    who: ref __self_0_0,
                    load: ref __self_0_1,
                    ratio: ref __self_0_2,
                    candidate_index: ref __self_0_3,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Edge");
                    let _ = debug_trait_builder.field("who", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("load", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("ratio", &&(*__self_0_2));
                    let _ = debug_trait_builder.field("candidate_index", &&(*__self_0_3));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    /// Perform election based on Phragmén algorithm.
    ///
    /// Reference implementation: https://github.com/w3f/consensus
    ///
    /// Returns an Option of elected candidates, if election is performed.
    /// Returns None if not enough candidates exist.
    ///
    /// The returned Option is a tuple consisting of:
    ///   - The list of elected candidates.
    ///   - The list of nominators and their associated vote weights.
    pub fn elect<T: Trait + 'static, FV, FN, FS>(
        validator_count: usize,
        minimum_validator_count: usize,
        validator_iter: FV,
        nominator_iter: FN,
        stash_of: FS,
    ) -> Option<(
        Vec<T::AccountId>,
        Vec<(T::AccountId, Vec<RawAssignment<T>>)>,
    )>
    where
        FV: Iterator<Item = (T::AccountId, ValidatorPrefs)>,
        FN: Iterator<Item = (T::AccountId, Vec<T::AccountId>)>,
        for<'r> FS: Fn(&'r T::AccountId) -> ExtendedBalance,
    {
        let mut elected_candidates: Vec<T::AccountId>;
        let mut assigned: Vec<(T::AccountId, Vec<RawAssignment<T>>)>;
        let mut c_idx_cache = BTreeMap::<T::AccountId, usize>::new();
        let mut nominators =
            Vec::with_capacity(validator_iter.size_hint().0 + nominator_iter.size_hint().0);
        let mut candidates: Vec<_> = validator_iter
            .map(|(who, _)| {
                let stash_balance = stash_of(&who);
                (
                    Candidate {
                        who,
                        ..Default::default()
                    },
                    stash_balance,
                )
            })
            .filter_map(|(mut c, s)| {
                c.approval_stake += s;
                if c.approval_stake.is_zero() {
                    None
                } else {
                    Some((c, s))
                }
            })
            .enumerate()
            .map(|(idx, (c, s))| {
                nominators.push(Nominator {
                    who: c.who.clone(),
                    edges: <[_]>::into_vec(box [Edge {
                        who: c.who.to_owned(),
                        candidate_index: idx,
                        ..Default::default()
                    }]),
                    budget: s,
                    load: Fraction::zero(),
                });
                c_idx_cache.insert(c.who.to_owned(), idx);
                c
            })
            .collect();
        nominators.extend(nominator_iter.map(|(who, nominees)| {
            let nominator_stake = stash_of(&who);
            let mut edges: Vec<Edge<T::AccountId>> = Vec::with_capacity(nominees.len());
            for n in &nominees {
                if let Some(idx) = c_idx_cache.get(n) {
                    candidates[*idx].approval_stake = candidates[*idx]
                        .approval_stake
                        .saturating_add(nominator_stake);
                    edges.push(Edge {
                        who: n.to_owned(),
                        candidate_index: *idx,
                        ..Default::default()
                    });
                }
            }
            Nominator {
                who,
                edges,
                budget: nominator_stake,
                load: Fraction::zero(),
            }
        }));
        if candidates.len() >= minimum_validator_count {
            let validator_count = validator_count.min(candidates.len());
            elected_candidates = Vec::with_capacity(validator_count);
            assigned = Vec::with_capacity(validator_count);
            for _round in 0..validator_count {
                for c in &mut candidates {
                    if !c.elected {
                        c.score = Fraction::from_xth(c.approval_stake);
                    }
                }
                for n in &nominators {
                    for e in &n.edges {
                        let c = &mut candidates[e.candidate_index];
                        if !c.elected && !c.approval_stake.is_zero() {
                            let temp = n.budget.saturating_mul(SCALE_FACTOR) / c.approval_stake
                                * (*n.load / SCALE_FACTOR);
                            c.score = Fraction::from_parts((*c.score).saturating_add(temp));
                        }
                    }
                }
                if let Some(winner) = candidates
                    .iter_mut()
                    .filter(|c| !c.elected)
                    .min_by_key(|c| *c.score)
                {
                    winner.elected = true;
                    for n in &mut nominators {
                        for e in &mut n.edges {
                            if e.who == winner.who {
                                e.load = Fraction::from_parts(*winner.score - *n.load);
                                n.load = winner.score;
                            }
                        }
                    }
                    elected_candidates.push(winner.who.to_owned());
                } else {
                    break;
                }
            }
            for n in &mut nominators {
                let mut assignment = (n.who.to_owned(), <[_]>::into_vec(box []));
                for e in &mut n.edges {
                    if let Some(c) = elected_candidates.iter().find(|c| **c == e.who) {
                        if *c != n.who {
                            let ratio = {
                                if n.load == e.load {
                                    ACCURACY
                                } else {
                                    if let Some(r) = ACCURACY.checked_mul(*e.load) {
                                        r / n.load.max(1)
                                    } else {
                                        *e.load / (n.load.max(1) / ACCURACY)
                                    }
                                }
                            };
                            e.ratio = ratio;
                            assignment.1.push((e.who.clone(), ratio));
                        }
                    }
                }
                if assignment.1.len() > 0 {
                    let vote_count = assignment.1.len() as ExtendedBalance;
                    let l = assignment.1.len();
                    let sum = assignment.1.iter().map(|a| a.1).sum();
                    let diff = ACCURACY.checked_sub(sum).unwrap_or(0);
                    let diff_per_vote = diff / vote_count;
                    if diff_per_vote > 0 {
                        for i in 0..l {
                            assignment.1[i % l].1 =
                                assignment.1[i % l].1.saturating_add(diff_per_vote);
                        }
                    }
                    let remainder = diff - diff_per_vote * vote_count;
                    for i in 0..remainder as usize {
                        assignment.1[i % l].1 = assignment.1[i % l].1.saturating_add(1);
                    }
                    assigned.push(assignment);
                }
            }
        } else {
            return None;
        }
        Some((elected_candidates, assigned))
    }
    /// Performs equalize post-processing to the output of the election algorithm
    /// This function mutates the input parameters, most noticeably it updates the exposure of
    /// the elected candidates.
    ///
    /// No value is returned from the function and the `expo_map` parameter is updated.
    pub fn equalize<T: Trait + 'static>(
        assignments: &mut Vec<(
            T::AccountId,
            ExtendedBalance,
            Vec<(T::AccountId, ExtendedBalance, ExtendedBalance)>,
        )>,
        expo_map: &mut ExpoMap<T>,
        tolerance: ExtendedBalance,
        iterations: usize,
    ) {
        for _i in 0..iterations {
            let mut max_diff = 0;
            assignments.iter_mut().for_each(|(n, budget, assignment)| {
                let diff = do_equalize::<T>(&n, *budget, assignment, expo_map, tolerance);
                if diff > max_diff {
                    max_diff = diff;
                }
            });
            if max_diff < tolerance {
                break;
            }
        }
    }
    fn do_equalize<T: Trait + 'static>(
        nominator: &T::AccountId,
        budget_balance: ExtendedBalance,
        elected_edges: &mut Vec<(T::AccountId, ExtendedBalance, ExtendedBalance)>,
        expo_map: &mut ExpoMap<T>,
        tolerance: ExtendedBalance,
    ) -> ExtendedBalance {
        let budget = budget_balance;
        if elected_edges.is_empty() {
            return 0;
        }
        let stake_used = elected_edges
            .iter()
            .fold(0, |s: ExtendedBalance, e| s.saturating_add(e.2));
        let backed_stakes_iter = elected_edges
            .iter()
            .filter_map(|e| expo_map.get(&e.0))
            .map(|e| e.total);
        let backing_backed_stake: Vec<_> = elected_edges
            .iter()
            .filter(|e| e.2 > 0)
            .filter_map(|e| expo_map.get(&e.0))
            .map(|e| e.total)
            .collect();
        let mut difference;
        if backing_backed_stake.len() > 0 {
            let max_stake = backing_backed_stake
                .iter()
                .max()
                .expect("vector with positive length will have a max; qed");
            let min_stake = backed_stakes_iter
                .min()
                .expect("iterator with positive length will have a min; qed");
            difference = max_stake.saturating_sub(min_stake);
            difference = difference.saturating_add(budget.saturating_sub(stake_used));
            if difference < tolerance {
                return difference;
            }
        } else {
            difference = budget;
        }
        elected_edges.iter_mut().for_each(|e| {
            if let Some(expo) = expo_map.get_mut(&e.0) {
                expo.total = expo.total.saturating_sub(e.2);
                expo.others.retain(|i_expo| i_expo.who != *nominator);
            }
            e.2 = 0;
        });
        elected_edges.sort_unstable_by_key(|e| {
            if let Some(e) = expo_map.get(&e.0) {
                e.total
            } else {
                0
            }
        });
        let mut cumulative_stake = 0;
        let mut last_index = elected_edges.len() - 1;
        elected_edges.iter_mut().enumerate().for_each(|(idx, e)| {
            if let Some(expo) = expo_map.get_mut(&e.0) {
                let stake = expo.total;
                let stake_mul = stake.saturating_mul(idx as _);
                let stake_sub = stake_mul.saturating_sub(cumulative_stake);
                if stake_sub > budget {
                    last_index = idx.checked_sub(1).unwrap_or(0);
                    return;
                }
                cumulative_stake = cumulative_stake.saturating_add(stake);
            }
        });
        let last_stake = elected_edges[last_index].2;
        let split_ways = last_index + 1;
        let excess = budget
            .saturating_add(cumulative_stake)
            .saturating_sub(last_stake.saturating_mul(split_ways as _));
        elected_edges.iter_mut().take(split_ways).for_each(|e| {
            if let Some(expo) = expo_map.get_mut(&e.0) {
                e.2 = (excess / split_ways as ExtendedBalance)
                    .saturating_add(last_stake)
                    .saturating_sub(expo.total);
                expo.total = expo.total.saturating_add(e.2);
                expo.others.push(IndividualExpo {
                    who: nominator.clone(),
                    value: e.2,
                });
            }
        });
        difference
    }
}
mod utils {
    use crate::{EraIndex, KtonBalanceOf, Module, RingBalanceOf, Trait};
    use primitives::traits::{IntegerSquareRoot, SaturatedConversion};
    /// utility in staking
    use rstd::convert::TryInto;
    use srml_support::traits::{Currency, Get};
    use substrate_primitives::U256;
    pub fn compute_current_era_reward<T: Trait>() -> RingBalanceOf<T> {
        let eras_per_epoch: RingBalanceOf<T> = <T::ErasPerEpoch as Get<EraIndex>>::get().into();
        let current_epoch = <Module<T>>::epoch_index();
        let total_left = (T::Cap::get() - T::Ring::total_issuance()).saturated_into::<u128>();
        let surplus = total_left
            - total_left * 99_u128.pow(current_epoch.integer_sqrt())
                / 100_u128.pow(current_epoch.integer_sqrt());
        let surplus: RingBalanceOf<T> = <RingBalanceOf<T>>::saturated_from::<u128>(surplus);
        (surplus / eras_per_epoch)
    }
    pub fn compute_kton_return<T: Trait>(value: RingBalanceOf<T>, months: u32) -> KtonBalanceOf<T> {
        let value = value.saturated_into::<u64>();
        let no = U256::from(67).pow(U256::from(months));
        let de = U256::from(66).pow(U256::from(months));
        let quotient = no / de;
        let remainder = no % de;
        let res = U256::from(value)
            * (U256::from(1000) * (quotient - 1) + U256::from(1000) * remainder / de)
            / U256::from(1970000);
        res.as_u128().try_into().unwrap_or_default()
    }
}
use parity_codec::{CompactAs, Decode, Encode, HasCompact};
use phragmen::{elect, equalize, ExtendedBalance, ACCURACY};
use primitives::{
    traits::{Bounded, CheckedSub, Convert, SaturatedConversion, Saturating, StaticLookup, Zero},
    Perbill,
};
#[cfg(feature = "std")]
use primitives::{Deserialize, Serialize};
use rstd::{collections::btree_map::BTreeMap, vec::Vec};
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
const RECENT_OFFLINE_COUNT: usize = 32;
const MAX_NOMINATIONS: usize = 16;
const MAX_UNLOCKING_CHUNKS: usize = 32;
const DEFAULT_MINIMUM_VALIDATOR_COUNT: u32 = 4;
const MAX_UNSTAKE_THRESHOLD: u32 = 10;
const MONTH_IN_SECONDS: u32 = 2592000;
const STAKING_ID: LockIdentifier = *b"staking ";
/// Counter for the number of eras that have passed.
pub type EraIndex = u32;
pub type ErasNums = u32;
pub enum StakerStatus<AccountId> {
    /// Chilling.
    Idle,
    /// Declared desire in validating or already participating in it.
    Validator,
    /// Nominating for a group of other stakers.
    Nominator(Vec<AccountId>),
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::fmt::Debug> ::std::fmt::Debug for StakerStatus<AccountId> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match (&*self,) {
            (&StakerStatus::Idle,) => {
                let mut debug_trait_builder = f.debug_tuple("Idle");
                debug_trait_builder.finish()
            }
            (&StakerStatus::Validator,) => {
                let mut debug_trait_builder = f.debug_tuple("Validator");
                debug_trait_builder.finish()
            }
            (&StakerStatus::Nominator(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("Nominator");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            }
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_SERIALIZE_FOR_StakerStatus: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<AccountId> _serde::Serialize for StakerStatus<AccountId>
    where
        AccountId: _serde::Serialize,
    {
        fn serialize<__S>(&self, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            match *self {
                StakerStatus::Idle => _serde::Serializer::serialize_unit_variant(
                    __serializer,
                    "StakerStatus",
                    0u32,
                    "Idle",
                ),
                StakerStatus::Validator => _serde::Serializer::serialize_unit_variant(
                    __serializer,
                    "StakerStatus",
                    1u32,
                    "Validator",
                ),
                StakerStatus::Nominator(ref __field0) => {
                    _serde::Serializer::serialize_newtype_variant(
                        __serializer,
                        "StakerStatus",
                        2u32,
                        "Nominator",
                        __field0,
                    )
                }
            }
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_StakerStatus: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de, AccountId> _serde::Deserialize<'de> for StakerStatus<AccountId>
    where
        AccountId: _serde::Deserialize<'de>,
    {
        fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
                __field2,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "variant identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::export::Ok(__Field::__field0),
                        1u64 => _serde::export::Ok(__Field::__field1),
                        2u64 => _serde::export::Ok(__Field::__field2),
                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                            _serde::de::Unexpected::Unsigned(__value),
                            &"variant index 0 <= i < 3",
                        )),
                    }
                }
                fn visit_str<__E>(self, __value: &str) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "Idle" => _serde::export::Ok(__Field::__field0),
                        "Validator" => _serde::export::Ok(__Field::__field1),
                        "Nominator" => _serde::export::Ok(__Field::__field2),
                        _ => _serde::export::Err(_serde::de::Error::unknown_variant(
                            __value, VARIANTS,
                        )),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"Idle" => _serde::export::Ok(__Field::__field0),
                        b"Validator" => _serde::export::Ok(__Field::__field1),
                        b"Nominator" => _serde::export::Ok(__Field::__field2),
                        _ => {
                            let __value = &_serde::export::from_utf8_lossy(__value);
                            _serde::export::Err(_serde::de::Error::unknown_variant(
                                __value, VARIANTS,
                            ))
                        }
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct __Visitor<'de, AccountId>
            where
                AccountId: _serde::Deserialize<'de>,
            {
                marker: _serde::export::PhantomData<StakerStatus<AccountId>>,
                lifetime: _serde::export::PhantomData<&'de ()>,
            }
            impl<'de, AccountId> _serde::de::Visitor<'de> for __Visitor<'de, AccountId>
            where
                AccountId: _serde::Deserialize<'de>,
            {
                type Value = StakerStatus<AccountId>;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "enum StakerStatus")
                }
                fn visit_enum<__A>(
                    self,
                    __data: __A,
                ) -> _serde::export::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::EnumAccess<'de>,
                {
                    match match _serde::de::EnumAccess::variant(__data) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        (__Field::__field0, __variant) => {
                            match _serde::de::VariantAccess::unit_variant(__variant) {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            };
                            _serde::export::Ok(StakerStatus::Idle)
                        }
                        (__Field::__field1, __variant) => {
                            match _serde::de::VariantAccess::unit_variant(__variant) {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            };
                            _serde::export::Ok(StakerStatus::Validator)
                        }
                        (__Field::__field2, __variant) => _serde::export::Result::map(
                            _serde::de::VariantAccess::newtype_variant::<Vec<AccountId>>(__variant),
                            StakerStatus::Nominator,
                        ),
                    }
                }
            }
            const VARIANTS: &'static [&'static str] = &["Idle", "Validator", "Nominator"];
            _serde::Deserializer::deserialize_enum(
                __deserializer,
                "StakerStatus",
                VARIANTS,
                __Visitor {
                    marker: _serde::export::PhantomData::<StakerStatus<AccountId>>,
                    lifetime: _serde::export::PhantomData,
                },
            )
        }
    }
};
#[structural_match]
pub struct ValidatorPrefs {
    /// Validator should ensure this many more slashes than is necessary before being unstaked.
    #[codec(compact)]
    pub unstake_threshold: u32,
    /// percent of Reward that validator takes up-front; only the rest is split between themselves and
    /// nominators.
    #[codec(compact)]
    pub validator_payment_ratio: Perbill,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::clone::Clone for ValidatorPrefs {
    #[inline]
    fn clone(&self) -> ValidatorPrefs {
        match *self {
            ValidatorPrefs {
                unstake_threshold: ref __self_0_0,
                validator_payment_ratio: ref __self_0_1,
            } => ValidatorPrefs {
                unstake_threshold: ::std::clone::Clone::clone(&(*__self_0_0)),
                validator_payment_ratio: ::std::clone::Clone::clone(&(*__self_0_1)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::PartialEq for ValidatorPrefs {
    #[inline]
    fn eq(&self, other: &ValidatorPrefs) -> bool {
        match *other {
            ValidatorPrefs {
                unstake_threshold: ref __self_1_0,
                validator_payment_ratio: ref __self_1_1,
            } => match *self {
                ValidatorPrefs {
                    unstake_threshold: ref __self_0_0,
                    validator_payment_ratio: ref __self_0_1,
                } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
            },
        }
    }
    #[inline]
    fn ne(&self, other: &ValidatorPrefs) -> bool {
        match *other {
            ValidatorPrefs {
                unstake_threshold: ref __self_1_0,
                validator_payment_ratio: ref __self_1_1,
            } => match *self {
                ValidatorPrefs {
                    unstake_threshold: ref __self_0_0,
                    validator_payment_ratio: ref __self_0_1,
                } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::Eq for ValidatorPrefs {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<u32>;
            let _: ::std::cmp::AssertParamIsEq<Perbill>;
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_ValidatorPrefs: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl _parity_codec::Encode for ValidatorPrefs {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            {
                dest.push(
                    &<<u32 as _parity_codec::HasCompact>::Type as _parity_codec::EncodeAsRef<
                        '_,
                        u32,
                    >>::from(&self.unstake_threshold),
                );
            }
            {
                dest.push(
                    &<<Perbill as _parity_codec::HasCompact>::Type as _parity_codec::EncodeAsRef<
                        '_,
                        Perbill,
                    >>::from(&self.validator_payment_ratio),
                );
            }
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_ValidatorPrefs: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl _parity_codec::Decode for ValidatorPrefs {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            Some(ValidatorPrefs {
                unstake_threshold:
                    <<u32 as _parity_codec::HasCompact>::Type as _parity_codec::Decode>::decode(
                        input,
                    )?
                    .into(),
                validator_payment_ratio:
                    <<Perbill as _parity_codec::HasCompact>::Type as _parity_codec::Decode>::decode(
                        input,
                    )?
                    .into(),
            })
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for ValidatorPrefs {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            ValidatorPrefs {
                unstake_threshold: ref __self_0_0,
                validator_payment_ratio: ref __self_0_1,
            } => {
                let mut debug_trait_builder = f.debug_struct("ValidatorPrefs");
                let _ = debug_trait_builder.field("unstake_threshold", &&(*__self_0_0));
                let _ = debug_trait_builder.field("validator_payment_ratio", &&(*__self_0_1));
                debug_trait_builder.finish()
            }
        }
    }
}
impl Default for ValidatorPrefs {
    fn default() -> Self {
        ValidatorPrefs {
            unstake_threshold: 3,
            validator_payment_ratio: Default::default(),
        }
    }
}
#[structural_match]
pub enum StakingBalance<RingBalance, KtonBalance> {
    Ring(RingBalance),
    Kton(KtonBalance),
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::clone::Clone, KtonBalance: ::std::clone::Clone> ::std::clone::Clone
    for StakingBalance<RingBalance, KtonBalance>
{
    #[inline]
    fn clone(&self) -> StakingBalance<RingBalance, KtonBalance> {
        match (&*self,) {
            (&StakingBalance::Ring(ref __self_0),) => {
                StakingBalance::Ring(::std::clone::Clone::clone(&(*__self_0)))
            }
            (&StakingBalance::Kton(ref __self_0),) => {
                StakingBalance::Kton(::std::clone::Clone::clone(&(*__self_0)))
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::cmp::PartialEq, KtonBalance: ::std::cmp::PartialEq> ::std::cmp::PartialEq
    for StakingBalance<RingBalance, KtonBalance>
{
    #[inline]
    fn eq(&self, other: &StakingBalance<RingBalance, KtonBalance>) -> bool {
        {
            let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as isize;
            let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as isize;
            if true && __self_vi == __arg_1_vi {
                match (&*self, &*other) {
                    (&StakingBalance::Ring(ref __self_0), &StakingBalance::Ring(ref __arg_1_0)) => {
                        (*__self_0) == (*__arg_1_0)
                    }
                    (&StakingBalance::Kton(ref __self_0), &StakingBalance::Kton(ref __arg_1_0)) => {
                        (*__self_0) == (*__arg_1_0)
                    }
                    _ => unsafe { ::std::intrinsics::unreachable() },
                }
            } else {
                false
            }
        }
    }
    #[inline]
    fn ne(&self, other: &StakingBalance<RingBalance, KtonBalance>) -> bool {
        {
            let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as isize;
            let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as isize;
            if true && __self_vi == __arg_1_vi {
                match (&*self, &*other) {
                    (&StakingBalance::Ring(ref __self_0), &StakingBalance::Ring(ref __arg_1_0)) => {
                        (*__self_0) != (*__arg_1_0)
                    }
                    (&StakingBalance::Kton(ref __self_0), &StakingBalance::Kton(ref __arg_1_0)) => {
                        (*__self_0) != (*__arg_1_0)
                    }
                    _ => unsafe { ::std::intrinsics::unreachable() },
                }
            } else {
                true
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::cmp::Eq, KtonBalance: ::std::cmp::Eq> ::std::cmp::Eq
    for StakingBalance<RingBalance, KtonBalance>
{
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<RingBalance>;
            let _: ::std::cmp::AssertParamIsEq<KtonBalance>;
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_StakingBalance: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<RingBalance, KtonBalance> _parity_codec::Encode for StakingBalance<RingBalance, KtonBalance>
    where
        RingBalance: _parity_codec::Encode,
        RingBalance: _parity_codec::Encode,
        KtonBalance: _parity_codec::Encode,
        KtonBalance: _parity_codec::Encode,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            match *self {
                StakingBalance::Ring(ref aa) => {
                    dest.push_byte(0usize as u8);
                    dest.push(aa);
                }
                StakingBalance::Kton(ref aa) => {
                    dest.push_byte(1usize as u8);
                    dest.push(aa);
                }
                _ => (),
            }
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_StakingBalance: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<RingBalance, KtonBalance> _parity_codec::Decode for StakingBalance<RingBalance, KtonBalance>
    where
        RingBalance: _parity_codec::Decode,
        RingBalance: _parity_codec::Decode,
        KtonBalance: _parity_codec::Decode,
        KtonBalance: _parity_codec::Decode,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            match input.read_byte()? {
                x if x == 0usize as u8 => {
                    Some(StakingBalance::Ring(_parity_codec::Decode::decode(input)?))
                }
                x if x == 1usize as u8 => {
                    Some(StakingBalance::Kton(_parity_codec::Decode::decode(input)?))
                }
                _ => None,
            }
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::fmt::Debug, KtonBalance: ::std::fmt::Debug> ::std::fmt::Debug
    for StakingBalance<RingBalance, KtonBalance>
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match (&*self,) {
            (&StakingBalance::Ring(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("Ring");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            }
            (&StakingBalance::Kton(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("Kton");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            }
        }
    }
}
impl<RingBalance: Default, KtonBalance: Default> Default
    for StakingBalance<RingBalance, KtonBalance>
{
    fn default() -> Self {
        StakingBalance::Ring(Default::default())
    }
}
/// A destination account for payment.
#[structural_match]
#[rustc_copy_clone_marker]
pub enum RewardDestination {
    /// Pay into the stash account, increasing the amount at stake accordingly.
    /// for now, we dont use this.
    /// Pay into the stash account, not increasing the amount at stake.
    Stash,
    /// Pay into the controller account.
    Controller,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::marker::Copy for RewardDestination {}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::clone::Clone for RewardDestination {
    #[inline]
    fn clone(&self) -> RewardDestination {
        {
            *self
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::PartialEq for RewardDestination {
    #[inline]
    fn eq(&self, other: &RewardDestination) -> bool {
        {
            let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as isize;
            let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as isize;
            if true && __self_vi == __arg_1_vi {
                match (&*self, &*other) {
                    _ => true,
                }
            } else {
                false
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::Eq for RewardDestination {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {}
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_RewardDestination: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl _parity_codec::Encode for RewardDestination {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            match *self {
                RewardDestination::Stash => {
                    dest.push_byte(0usize as u8);
                }
                RewardDestination::Controller => {
                    dest.push_byte(1usize as u8);
                }
                _ => (),
            }
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_RewardDestination: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl _parity_codec::Decode for RewardDestination {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            match input.read_byte()? {
                x if x == 0usize as u8 => Some(RewardDestination::Stash),
                x if x == 1usize as u8 => Some(RewardDestination::Controller),
                _ => None,
            }
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for RewardDestination {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match (&*self,) {
            (&RewardDestination::Stash,) => {
                let mut debug_trait_builder = f.debug_tuple("Stash");
                debug_trait_builder.finish()
            }
            (&RewardDestination::Controller,) => {
                let mut debug_trait_builder = f.debug_tuple("Controller");
                debug_trait_builder.finish()
            }
        }
    }
}
impl Default for RewardDestination {
    fn default() -> Self {
        RewardDestination::Stash
    }
}
#[structural_match]
pub struct UnlockChunk<RingBalance, KtonBalance> {
    /// Amount of funds to be unlocked.
    value: StakingBalance<RingBalance, KtonBalance>,
    /// Era number at which point it'll be unlocked.
    #[codec(compact)]
    era: EraIndex,
    is_time_deposit: bool,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::clone::Clone, KtonBalance: ::std::clone::Clone> ::std::clone::Clone
    for UnlockChunk<RingBalance, KtonBalance>
{
    #[inline]
    fn clone(&self) -> UnlockChunk<RingBalance, KtonBalance> {
        match *self {
            UnlockChunk {
                value: ref __self_0_0,
                era: ref __self_0_1,
                is_time_deposit: ref __self_0_2,
            } => UnlockChunk {
                value: ::std::clone::Clone::clone(&(*__self_0_0)),
                era: ::std::clone::Clone::clone(&(*__self_0_1)),
                is_time_deposit: ::std::clone::Clone::clone(&(*__self_0_2)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::cmp::PartialEq, KtonBalance: ::std::cmp::PartialEq> ::std::cmp::PartialEq
    for UnlockChunk<RingBalance, KtonBalance>
{
    #[inline]
    fn eq(&self, other: &UnlockChunk<RingBalance, KtonBalance>) -> bool {
        match *other {
            UnlockChunk {
                value: ref __self_1_0,
                era: ref __self_1_1,
                is_time_deposit: ref __self_1_2,
            } => match *self {
                UnlockChunk {
                    value: ref __self_0_0,
                    era: ref __self_0_1,
                    is_time_deposit: ref __self_0_2,
                } => {
                    (*__self_0_0) == (*__self_1_0)
                        && (*__self_0_1) == (*__self_1_1)
                        && (*__self_0_2) == (*__self_1_2)
                }
            },
        }
    }
    #[inline]
    fn ne(&self, other: &UnlockChunk<RingBalance, KtonBalance>) -> bool {
        match *other {
            UnlockChunk {
                value: ref __self_1_0,
                era: ref __self_1_1,
                is_time_deposit: ref __self_1_2,
            } => match *self {
                UnlockChunk {
                    value: ref __self_0_0,
                    era: ref __self_0_1,
                    is_time_deposit: ref __self_0_2,
                } => {
                    (*__self_0_0) != (*__self_1_0)
                        || (*__self_0_1) != (*__self_1_1)
                        || (*__self_0_2) != (*__self_1_2)
                }
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::cmp::Eq, KtonBalance: ::std::cmp::Eq> ::std::cmp::Eq
    for UnlockChunk<RingBalance, KtonBalance>
{
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<StakingBalance<RingBalance, KtonBalance>>;
            let _: ::std::cmp::AssertParamIsEq<EraIndex>;
            let _: ::std::cmp::AssertParamIsEq<bool>;
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_UnlockChunk: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<RingBalance, KtonBalance> _parity_codec::Encode for UnlockChunk<RingBalance, KtonBalance>
    where
        StakingBalance<RingBalance, KtonBalance>: _parity_codec::Encode,
        StakingBalance<RingBalance, KtonBalance>: _parity_codec::Encode,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            dest.push(&self.value);
            {
                dest . push ( & < < EraIndex as _parity_codec :: HasCompact > :: Type as _parity_codec :: EncodeAsRef < '_ , EraIndex > > :: from ( & self . era ) ) ;
            }
            dest.push(&self.is_time_deposit);
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_UnlockChunk: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<RingBalance, KtonBalance> _parity_codec::Decode for UnlockChunk<RingBalance, KtonBalance>
    where
        StakingBalance<RingBalance, KtonBalance>: _parity_codec::Decode,
        StakingBalance<RingBalance, KtonBalance>: _parity_codec::Decode,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            Some ( UnlockChunk { value : _parity_codec :: Decode :: decode ( input ) ? , era : < < EraIndex as _parity_codec :: HasCompact > :: Type as _parity_codec :: Decode > :: decode ( input ) ? . into ( ) , is_time_deposit : _parity_codec :: Decode :: decode ( input ) ? , } )
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::fmt::Debug, KtonBalance: ::std::fmt::Debug> ::std::fmt::Debug
    for UnlockChunk<RingBalance, KtonBalance>
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            UnlockChunk {
                value: ref __self_0_0,
                era: ref __self_0_1,
                is_time_deposit: ref __self_0_2,
            } => {
                let mut debug_trait_builder = f.debug_struct("UnlockChunk");
                let _ = debug_trait_builder.field("value", &&(*__self_0_0));
                let _ = debug_trait_builder.field("era", &&(*__self_0_1));
                let _ = debug_trait_builder.field("is_time_deposit", &&(*__self_0_2));
                debug_trait_builder.finish()
            }
        }
    }
}
#[structural_match]
pub struct TimeDepositItem<RingBalance: HasCompact, Moment> {
    #[codec(compact)]
    value: RingBalance,
    #[codec(compact)]
    start_time: Moment,
    #[codec(compact)]
    expire_time: Moment,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::clone::Clone + HasCompact, Moment: ::std::clone::Clone> ::std::clone::Clone
    for TimeDepositItem<RingBalance, Moment>
{
    #[inline]
    fn clone(&self) -> TimeDepositItem<RingBalance, Moment> {
        match *self {
            TimeDepositItem {
                value: ref __self_0_0,
                start_time: ref __self_0_1,
                expire_time: ref __self_0_2,
            } => TimeDepositItem {
                value: ::std::clone::Clone::clone(&(*__self_0_0)),
                start_time: ::std::clone::Clone::clone(&(*__self_0_1)),
                expire_time: ::std::clone::Clone::clone(&(*__self_0_2)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::cmp::PartialEq + HasCompact, Moment: ::std::cmp::PartialEq>
    ::std::cmp::PartialEq for TimeDepositItem<RingBalance, Moment>
{
    #[inline]
    fn eq(&self, other: &TimeDepositItem<RingBalance, Moment>) -> bool {
        match *other {
            TimeDepositItem {
                value: ref __self_1_0,
                start_time: ref __self_1_1,
                expire_time: ref __self_1_2,
            } => match *self {
                TimeDepositItem {
                    value: ref __self_0_0,
                    start_time: ref __self_0_1,
                    expire_time: ref __self_0_2,
                } => {
                    (*__self_0_0) == (*__self_1_0)
                        && (*__self_0_1) == (*__self_1_1)
                        && (*__self_0_2) == (*__self_1_2)
                }
            },
        }
    }
    #[inline]
    fn ne(&self, other: &TimeDepositItem<RingBalance, Moment>) -> bool {
        match *other {
            TimeDepositItem {
                value: ref __self_1_0,
                start_time: ref __self_1_1,
                expire_time: ref __self_1_2,
            } => match *self {
                TimeDepositItem {
                    value: ref __self_0_0,
                    start_time: ref __self_0_1,
                    expire_time: ref __self_0_2,
                } => {
                    (*__self_0_0) != (*__self_1_0)
                        || (*__self_0_1) != (*__self_1_1)
                        || (*__self_0_2) != (*__self_1_2)
                }
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::cmp::Eq + HasCompact, Moment: ::std::cmp::Eq> ::std::cmp::Eq
    for TimeDepositItem<RingBalance, Moment>
{
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<RingBalance>;
            let _: ::std::cmp::AssertParamIsEq<Moment>;
            let _: ::std::cmp::AssertParamIsEq<Moment>;
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_TimeDepositItem: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<RingBalance: HasCompact, Moment> _parity_codec::Encode for TimeDepositItem<RingBalance, Moment>
    where
        RingBalance: _parity_codec::HasCompact,
        Moment: _parity_codec::HasCompact,
        Moment: _parity_codec::HasCompact,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            {
                dest . push ( & < < RingBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: EncodeAsRef < '_ , RingBalance > > :: from ( & self . value ) ) ;
            }
            {
                dest.push(
                    &<<Moment as _parity_codec::HasCompact>::Type as _parity_codec::EncodeAsRef<
                        '_,
                        Moment,
                    >>::from(&self.start_time),
                );
            }
            {
                dest.push(
                    &<<Moment as _parity_codec::HasCompact>::Type as _parity_codec::EncodeAsRef<
                        '_,
                        Moment,
                    >>::from(&self.expire_time),
                );
            }
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_TimeDepositItem: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<RingBalance: HasCompact, Moment> _parity_codec::Decode for TimeDepositItem<RingBalance, Moment>
    where
        RingBalance: _parity_codec::HasCompact,
        Moment: _parity_codec::HasCompact,
        Moment: _parity_codec::HasCompact,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            Some ( TimeDepositItem { value : < < RingBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: Decode > :: decode ( input ) ? . into ( ) , start_time : < < Moment as _parity_codec :: HasCompact > :: Type as _parity_codec :: Decode > :: decode ( input ) ? . into ( ) , expire_time : < < Moment as _parity_codec :: HasCompact > :: Type as _parity_codec :: Decode > :: decode ( input ) ? . into ( ) , } )
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl<RingBalance: ::std::fmt::Debug + HasCompact, Moment: ::std::fmt::Debug> ::std::fmt::Debug
    for TimeDepositItem<RingBalance, Moment>
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            TimeDepositItem {
                value: ref __self_0_0,
                start_time: ref __self_0_1,
                expire_time: ref __self_0_2,
            } => {
                let mut debug_trait_builder = f.debug_struct("TimeDepositItem");
                let _ = debug_trait_builder.field("value", &&(*__self_0_0));
                let _ = debug_trait_builder.field("start_time", &&(*__self_0_1));
                let _ = debug_trait_builder.field("expire_time", &&(*__self_0_2));
                debug_trait_builder.finish()
            }
        }
    }
}
#[structural_match]
pub struct StakingLedger<AccountId, Moment, RingBalance: HasCompact, KtonBalance: HasCompact> {
    pub stash: AccountId,
    /// total_ring = normal_ring + time_deposit_ring
    #[codec(compact)]
    pub total_ring: RingBalance,
    #[codec(compact)]
    pub active_ring: RingBalance,
    #[codec(compact)]
    pub total_deposit_ring: RingBalance,
    #[codec(compact)]
    pub active_deposit_ring: RingBalance,
    #[codec(compact)]
    pub total_kton: KtonBalance,
    #[codec(compact)]
    pub active_kton: KtonBalance,
    pub deposit_items: Vec<TimeDepositItem<RingBalance, Moment>>,
    pub unlocking: Vec<UnlockChunk<RingBalance, KtonBalance>>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<
        AccountId: ::std::clone::Clone,
        Moment: ::std::clone::Clone,
        RingBalance: ::std::clone::Clone + HasCompact,
        KtonBalance: ::std::clone::Clone + HasCompact,
    > ::std::clone::Clone for StakingLedger<AccountId, Moment, RingBalance, KtonBalance>
{
    #[inline]
    fn clone(&self) -> StakingLedger<AccountId, Moment, RingBalance, KtonBalance> {
        match *self {
            StakingLedger {
                stash: ref __self_0_0,
                total_ring: ref __self_0_1,
                active_ring: ref __self_0_2,
                total_deposit_ring: ref __self_0_3,
                active_deposit_ring: ref __self_0_4,
                total_kton: ref __self_0_5,
                active_kton: ref __self_0_6,
                deposit_items: ref __self_0_7,
                unlocking: ref __self_0_8,
            } => StakingLedger {
                stash: ::std::clone::Clone::clone(&(*__self_0_0)),
                total_ring: ::std::clone::Clone::clone(&(*__self_0_1)),
                active_ring: ::std::clone::Clone::clone(&(*__self_0_2)),
                total_deposit_ring: ::std::clone::Clone::clone(&(*__self_0_3)),
                active_deposit_ring: ::std::clone::Clone::clone(&(*__self_0_4)),
                total_kton: ::std::clone::Clone::clone(&(*__self_0_5)),
                active_kton: ::std::clone::Clone::clone(&(*__self_0_6)),
                deposit_items: ::std::clone::Clone::clone(&(*__self_0_7)),
                unlocking: ::std::clone::Clone::clone(&(*__self_0_8)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<
        AccountId: ::std::default::Default,
        Moment: ::std::default::Default,
        RingBalance: ::std::default::Default + HasCompact,
        KtonBalance: ::std::default::Default + HasCompact,
    > ::std::default::Default for StakingLedger<AccountId, Moment, RingBalance, KtonBalance>
{
    #[inline]
    fn default() -> StakingLedger<AccountId, Moment, RingBalance, KtonBalance> {
        StakingLedger {
            stash: ::std::default::Default::default(),
            total_ring: ::std::default::Default::default(),
            active_ring: ::std::default::Default::default(),
            total_deposit_ring: ::std::default::Default::default(),
            active_deposit_ring: ::std::default::Default::default(),
            total_kton: ::std::default::Default::default(),
            active_kton: ::std::default::Default::default(),
            deposit_items: ::std::default::Default::default(),
            unlocking: ::std::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<
        AccountId: ::std::cmp::PartialEq,
        Moment: ::std::cmp::PartialEq,
        RingBalance: ::std::cmp::PartialEq + HasCompact,
        KtonBalance: ::std::cmp::PartialEq + HasCompact,
    > ::std::cmp::PartialEq for StakingLedger<AccountId, Moment, RingBalance, KtonBalance>
{
    #[inline]
    fn eq(&self, other: &StakingLedger<AccountId, Moment, RingBalance, KtonBalance>) -> bool {
        match *other {
            StakingLedger {
                stash: ref __self_1_0,
                total_ring: ref __self_1_1,
                active_ring: ref __self_1_2,
                total_deposit_ring: ref __self_1_3,
                active_deposit_ring: ref __self_1_4,
                total_kton: ref __self_1_5,
                active_kton: ref __self_1_6,
                deposit_items: ref __self_1_7,
                unlocking: ref __self_1_8,
            } => match *self {
                StakingLedger {
                    stash: ref __self_0_0,
                    total_ring: ref __self_0_1,
                    active_ring: ref __self_0_2,
                    total_deposit_ring: ref __self_0_3,
                    active_deposit_ring: ref __self_0_4,
                    total_kton: ref __self_0_5,
                    active_kton: ref __self_0_6,
                    deposit_items: ref __self_0_7,
                    unlocking: ref __self_0_8,
                } => {
                    (*__self_0_0) == (*__self_1_0)
                        && (*__self_0_1) == (*__self_1_1)
                        && (*__self_0_2) == (*__self_1_2)
                        && (*__self_0_3) == (*__self_1_3)
                        && (*__self_0_4) == (*__self_1_4)
                        && (*__self_0_5) == (*__self_1_5)
                        && (*__self_0_6) == (*__self_1_6)
                        && (*__self_0_7) == (*__self_1_7)
                        && (*__self_0_8) == (*__self_1_8)
                }
            },
        }
    }
    #[inline]
    fn ne(&self, other: &StakingLedger<AccountId, Moment, RingBalance, KtonBalance>) -> bool {
        match *other {
            StakingLedger {
                stash: ref __self_1_0,
                total_ring: ref __self_1_1,
                active_ring: ref __self_1_2,
                total_deposit_ring: ref __self_1_3,
                active_deposit_ring: ref __self_1_4,
                total_kton: ref __self_1_5,
                active_kton: ref __self_1_6,
                deposit_items: ref __self_1_7,
                unlocking: ref __self_1_8,
            } => match *self {
                StakingLedger {
                    stash: ref __self_0_0,
                    total_ring: ref __self_0_1,
                    active_ring: ref __self_0_2,
                    total_deposit_ring: ref __self_0_3,
                    active_deposit_ring: ref __self_0_4,
                    total_kton: ref __self_0_5,
                    active_kton: ref __self_0_6,
                    deposit_items: ref __self_0_7,
                    unlocking: ref __self_0_8,
                } => {
                    (*__self_0_0) != (*__self_1_0)
                        || (*__self_0_1) != (*__self_1_1)
                        || (*__self_0_2) != (*__self_1_2)
                        || (*__self_0_3) != (*__self_1_3)
                        || (*__self_0_4) != (*__self_1_4)
                        || (*__self_0_5) != (*__self_1_5)
                        || (*__self_0_6) != (*__self_1_6)
                        || (*__self_0_7) != (*__self_1_7)
                        || (*__self_0_8) != (*__self_1_8)
                }
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<
        AccountId: ::std::cmp::Eq,
        Moment: ::std::cmp::Eq,
        RingBalance: ::std::cmp::Eq + HasCompact,
        KtonBalance: ::std::cmp::Eq + HasCompact,
    > ::std::cmp::Eq for StakingLedger<AccountId, Moment, RingBalance, KtonBalance>
{
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<AccountId>;
            let _: ::std::cmp::AssertParamIsEq<RingBalance>;
            let _: ::std::cmp::AssertParamIsEq<RingBalance>;
            let _: ::std::cmp::AssertParamIsEq<RingBalance>;
            let _: ::std::cmp::AssertParamIsEq<RingBalance>;
            let _: ::std::cmp::AssertParamIsEq<KtonBalance>;
            let _: ::std::cmp::AssertParamIsEq<KtonBalance>;
            let _: ::std::cmp::AssertParamIsEq<Vec<TimeDepositItem<RingBalance, Moment>>>;
            let _: ::std::cmp::AssertParamIsEq<Vec<UnlockChunk<RingBalance, KtonBalance>>>;
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_StakingLedger: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<AccountId, Moment, RingBalance: HasCompact, KtonBalance: HasCompact> _parity_codec::Encode
        for StakingLedger<AccountId, Moment, RingBalance, KtonBalance>
    where
        AccountId: _parity_codec::Encode,
        AccountId: _parity_codec::Encode,
        Vec<TimeDepositItem<RingBalance, Moment>>: _parity_codec::Encode,
        Vec<TimeDepositItem<RingBalance, Moment>>: _parity_codec::Encode,
        Vec<UnlockChunk<RingBalance, KtonBalance>>: _parity_codec::Encode,
        Vec<UnlockChunk<RingBalance, KtonBalance>>: _parity_codec::Encode,
        RingBalance: _parity_codec::HasCompact,
        RingBalance: _parity_codec::HasCompact,
        RingBalance: _parity_codec::HasCompact,
        RingBalance: _parity_codec::HasCompact,
        KtonBalance: _parity_codec::HasCompact,
        KtonBalance: _parity_codec::HasCompact,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            dest.push(&self.stash);
            {
                dest . push ( & < < RingBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: EncodeAsRef < '_ , RingBalance > > :: from ( & self . total_ring ) ) ;
            }
            {
                dest . push ( & < < RingBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: EncodeAsRef < '_ , RingBalance > > :: from ( & self . active_ring ) ) ;
            }
            {
                dest . push ( & < < RingBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: EncodeAsRef < '_ , RingBalance > > :: from ( & self . total_deposit_ring ) ) ;
            }
            {
                dest . push ( & < < RingBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: EncodeAsRef < '_ , RingBalance > > :: from ( & self . active_deposit_ring ) ) ;
            }
            {
                dest . push ( & < < KtonBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: EncodeAsRef < '_ , KtonBalance > > :: from ( & self . total_kton ) ) ;
            }
            {
                dest . push ( & < < KtonBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: EncodeAsRef < '_ , KtonBalance > > :: from ( & self . active_kton ) ) ;
            }
            dest.push(&self.deposit_items);
            dest.push(&self.unlocking);
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_StakingLedger: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<AccountId, Moment, RingBalance: HasCompact, KtonBalance: HasCompact> _parity_codec::Decode
        for StakingLedger<AccountId, Moment, RingBalance, KtonBalance>
    where
        AccountId: _parity_codec::Decode,
        AccountId: _parity_codec::Decode,
        Vec<TimeDepositItem<RingBalance, Moment>>: _parity_codec::Decode,
        Vec<TimeDepositItem<RingBalance, Moment>>: _parity_codec::Decode,
        Vec<UnlockChunk<RingBalance, KtonBalance>>: _parity_codec::Decode,
        Vec<UnlockChunk<RingBalance, KtonBalance>>: _parity_codec::Decode,
        RingBalance: _parity_codec::HasCompact,
        RingBalance: _parity_codec::HasCompact,
        RingBalance: _parity_codec::HasCompact,
        RingBalance: _parity_codec::HasCompact,
        KtonBalance: _parity_codec::HasCompact,
        KtonBalance: _parity_codec::HasCompact,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            Some ( StakingLedger { stash : _parity_codec :: Decode :: decode ( input ) ? , total_ring : < < RingBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: Decode > :: decode ( input ) ? . into ( ) , active_ring : < < RingBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: Decode > :: decode ( input ) ? . into ( ) , total_deposit_ring : < < RingBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: Decode > :: decode ( input ) ? . into ( ) , active_deposit_ring : < < RingBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: Decode > :: decode ( input ) ? . into ( ) , total_kton : < < KtonBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: Decode > :: decode ( input ) ? . into ( ) , active_kton : < < KtonBalance as _parity_codec :: HasCompact > :: Type as _parity_codec :: Decode > :: decode ( input ) ? . into ( ) , deposit_items : _parity_codec :: Decode :: decode ( input ) ? , unlocking : _parity_codec :: Decode :: decode ( input ) ? , } )
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl<
        AccountId: ::std::fmt::Debug,
        Moment: ::std::fmt::Debug,
        RingBalance: ::std::fmt::Debug + HasCompact,
        KtonBalance: ::std::fmt::Debug + HasCompact,
    > ::std::fmt::Debug for StakingLedger<AccountId, Moment, RingBalance, KtonBalance>
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            StakingLedger {
                stash: ref __self_0_0,
                total_ring: ref __self_0_1,
                active_ring: ref __self_0_2,
                total_deposit_ring: ref __self_0_3,
                active_deposit_ring: ref __self_0_4,
                total_kton: ref __self_0_5,
                active_kton: ref __self_0_6,
                deposit_items: ref __self_0_7,
                unlocking: ref __self_0_8,
            } => {
                let mut debug_trait_builder = f.debug_struct("StakingLedger");
                let _ = debug_trait_builder.field("stash", &&(*__self_0_0));
                let _ = debug_trait_builder.field("total_ring", &&(*__self_0_1));
                let _ = debug_trait_builder.field("active_ring", &&(*__self_0_2));
                let _ = debug_trait_builder.field("total_deposit_ring", &&(*__self_0_3));
                let _ = debug_trait_builder.field("active_deposit_ring", &&(*__self_0_4));
                let _ = debug_trait_builder.field("total_kton", &&(*__self_0_5));
                let _ = debug_trait_builder.field("active_kton", &&(*__self_0_6));
                let _ = debug_trait_builder.field("deposit_items", &&(*__self_0_7));
                let _ = debug_trait_builder.field("unlocking", &&(*__self_0_8));
                debug_trait_builder.finish()
            }
        }
    }
}
/// The amount of exposure (to slashing) than an individual nominator has.
#[structural_match]
pub struct IndividualExpo<AccountId, Power> {
    /// The stash account of the nominator in question.
    who: AccountId,
    /// Amount of funds exposed.
    value: Power,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::clone::Clone, Power: ::std::clone::Clone> ::std::clone::Clone
    for IndividualExpo<AccountId, Power>
{
    #[inline]
    fn clone(&self) -> IndividualExpo<AccountId, Power> {
        match *self {
            IndividualExpo {
                who: ref __self_0_0,
                value: ref __self_0_1,
            } => IndividualExpo {
                who: ::std::clone::Clone::clone(&(*__self_0_0)),
                value: ::std::clone::Clone::clone(&(*__self_0_1)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::cmp::PartialEq, Power: ::std::cmp::PartialEq> ::std::cmp::PartialEq
    for IndividualExpo<AccountId, Power>
{
    #[inline]
    fn eq(&self, other: &IndividualExpo<AccountId, Power>) -> bool {
        match *other {
            IndividualExpo {
                who: ref __self_1_0,
                value: ref __self_1_1,
            } => match *self {
                IndividualExpo {
                    who: ref __self_0_0,
                    value: ref __self_0_1,
                } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
            },
        }
    }
    #[inline]
    fn ne(&self, other: &IndividualExpo<AccountId, Power>) -> bool {
        match *other {
            IndividualExpo {
                who: ref __self_1_0,
                value: ref __self_1_1,
            } => match *self {
                IndividualExpo {
                    who: ref __self_0_0,
                    value: ref __self_0_1,
                } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::cmp::Eq, Power: ::std::cmp::Eq> ::std::cmp::Eq
    for IndividualExpo<AccountId, Power>
{
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<AccountId>;
            let _: ::std::cmp::AssertParamIsEq<Power>;
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::cmp::PartialOrd, Power: ::std::cmp::PartialOrd> ::std::cmp::PartialOrd
    for IndividualExpo<AccountId, Power>
{
    #[inline]
    fn partial_cmp(
        &self,
        other: &IndividualExpo<AccountId, Power>,
    ) -> ::std::option::Option<::std::cmp::Ordering> {
        match *other {
            IndividualExpo {
                who: ref __self_1_0,
                value: ref __self_1_1,
            } => match *self {
                IndividualExpo {
                    who: ref __self_0_0,
                    value: ref __self_0_1,
                } => match ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)) {
                    ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {
                        match ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_1), &(*__self_1_1)) {
                            ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {
                                ::std::option::Option::Some(::std::cmp::Ordering::Equal)
                            }
                            cmp => cmp,
                        }
                    }
                    cmp => cmp,
                },
            },
        }
    }
    #[inline]
    fn lt(&self, other: &IndividualExpo<AccountId, Power>) -> bool {
        match *other {
            IndividualExpo {
                who: ref __self_1_0,
                value: ref __self_1_1,
            } => match *self {
                IndividualExpo {
                    who: ref __self_0_0,
                    value: ref __self_0_1,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::option::Option::unwrap_or(
                                ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_1), &(*__self_1_1)),
                                ::std::cmp::Ordering::Greater,
                            )
                        },
                    ) == ::std::cmp::Ordering::Less
                }
            },
        }
    }
    #[inline]
    fn le(&self, other: &IndividualExpo<AccountId, Power>) -> bool {
        match *other {
            IndividualExpo {
                who: ref __self_1_0,
                value: ref __self_1_1,
            } => match *self {
                IndividualExpo {
                    who: ref __self_0_0,
                    value: ref __self_0_1,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::option::Option::unwrap_or(
                                ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_1), &(*__self_1_1)),
                                ::std::cmp::Ordering::Greater,
                            )
                        },
                    ) != ::std::cmp::Ordering::Greater
                }
            },
        }
    }
    #[inline]
    fn gt(&self, other: &IndividualExpo<AccountId, Power>) -> bool {
        match *other {
            IndividualExpo {
                who: ref __self_1_0,
                value: ref __self_1_1,
            } => match *self {
                IndividualExpo {
                    who: ref __self_0_0,
                    value: ref __self_0_1,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::option::Option::unwrap_or(
                                ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_1), &(*__self_1_1)),
                                ::std::cmp::Ordering::Less,
                            )
                        },
                    ) == ::std::cmp::Ordering::Greater
                }
            },
        }
    }
    #[inline]
    fn ge(&self, other: &IndividualExpo<AccountId, Power>) -> bool {
        match *other {
            IndividualExpo {
                who: ref __self_1_0,
                value: ref __self_1_1,
            } => match *self {
                IndividualExpo {
                    who: ref __self_0_0,
                    value: ref __self_0_1,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::option::Option::unwrap_or(
                                ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_1), &(*__self_1_1)),
                                ::std::cmp::Ordering::Less,
                            )
                        },
                    ) != ::std::cmp::Ordering::Less
                }
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::cmp::Ord, Power: ::std::cmp::Ord> ::std::cmp::Ord
    for IndividualExpo<AccountId, Power>
{
    #[inline]
    fn cmp(&self, other: &IndividualExpo<AccountId, Power>) -> ::std::cmp::Ordering {
        match *other {
            IndividualExpo {
                who: ref __self_1_0,
                value: ref __self_1_1,
            } => match *self {
                IndividualExpo {
                    who: ref __self_0_0,
                    value: ref __self_0_1,
                } => match ::std::cmp::Ord::cmp(&(*__self_0_0), &(*__self_1_0)) {
                    ::std::cmp::Ordering::Equal => {
                        match ::std::cmp::Ord::cmp(&(*__self_0_1), &(*__self_1_1)) {
                            ::std::cmp::Ordering::Equal => ::std::cmp::Ordering::Equal,
                            cmp => cmp,
                        }
                    }
                    cmp => cmp,
                },
            },
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_IndividualExpo: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<AccountId, Power> _parity_codec::Encode for IndividualExpo<AccountId, Power>
    where
        AccountId: _parity_codec::Encode,
        AccountId: _parity_codec::Encode,
        Power: _parity_codec::Encode,
        Power: _parity_codec::Encode,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            dest.push(&self.who);
            dest.push(&self.value);
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_IndividualExpo: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<AccountId, Power> _parity_codec::Decode for IndividualExpo<AccountId, Power>
    where
        AccountId: _parity_codec::Decode,
        AccountId: _parity_codec::Decode,
        Power: _parity_codec::Decode,
        Power: _parity_codec::Decode,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            Some(IndividualExpo {
                who: _parity_codec::Decode::decode(input)?,
                value: _parity_codec::Decode::decode(input)?,
            })
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::fmt::Debug, Power: ::std::fmt::Debug> ::std::fmt::Debug
    for IndividualExpo<AccountId, Power>
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            IndividualExpo {
                who: ref __self_0_0,
                value: ref __self_0_1,
            } => {
                let mut debug_trait_builder = f.debug_struct("IndividualExpo");
                let _ = debug_trait_builder.field("who", &&(*__self_0_0));
                let _ = debug_trait_builder.field("value", &&(*__self_0_1));
                debug_trait_builder.finish()
            }
        }
    }
}
/// A snapshot of the stake backing a single validator in the system.
#[structural_match]
pub struct Exposures<AccountId, Power> {
    /// The total balance backing this validator.
    pub total: Power,
    /// The validator's own stash that is exposed.
    pub own: Power,
    /// The portions of nominators stashes that are exposed.
    pub others: Vec<IndividualExpo<AccountId, Power>>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::clone::Clone, Power: ::std::clone::Clone> ::std::clone::Clone
    for Exposures<AccountId, Power>
{
    #[inline]
    fn clone(&self) -> Exposures<AccountId, Power> {
        match *self {
            Exposures {
                total: ref __self_0_0,
                own: ref __self_0_1,
                others: ref __self_0_2,
            } => Exposures {
                total: ::std::clone::Clone::clone(&(*__self_0_0)),
                own: ::std::clone::Clone::clone(&(*__self_0_1)),
                others: ::std::clone::Clone::clone(&(*__self_0_2)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::default::Default, Power: ::std::default::Default> ::std::default::Default
    for Exposures<AccountId, Power>
{
    #[inline]
    fn default() -> Exposures<AccountId, Power> {
        Exposures {
            total: ::std::default::Default::default(),
            own: ::std::default::Default::default(),
            others: ::std::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::cmp::PartialEq, Power: ::std::cmp::PartialEq> ::std::cmp::PartialEq
    for Exposures<AccountId, Power>
{
    #[inline]
    fn eq(&self, other: &Exposures<AccountId, Power>) -> bool {
        match *other {
            Exposures {
                total: ref __self_1_0,
                own: ref __self_1_1,
                others: ref __self_1_2,
            } => match *self {
                Exposures {
                    total: ref __self_0_0,
                    own: ref __self_0_1,
                    others: ref __self_0_2,
                } => {
                    (*__self_0_0) == (*__self_1_0)
                        && (*__self_0_1) == (*__self_1_1)
                        && (*__self_0_2) == (*__self_1_2)
                }
            },
        }
    }
    #[inline]
    fn ne(&self, other: &Exposures<AccountId, Power>) -> bool {
        match *other {
            Exposures {
                total: ref __self_1_0,
                own: ref __self_1_1,
                others: ref __self_1_2,
            } => match *self {
                Exposures {
                    total: ref __self_0_0,
                    own: ref __self_0_1,
                    others: ref __self_0_2,
                } => {
                    (*__self_0_0) != (*__self_1_0)
                        || (*__self_0_1) != (*__self_1_1)
                        || (*__self_0_2) != (*__self_1_2)
                }
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::cmp::Eq, Power: ::std::cmp::Eq> ::std::cmp::Eq
    for Exposures<AccountId, Power>
{
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<Power>;
            let _: ::std::cmp::AssertParamIsEq<Power>;
            let _: ::std::cmp::AssertParamIsEq<Vec<IndividualExpo<AccountId, Power>>>;
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::cmp::PartialOrd, Power: ::std::cmp::PartialOrd> ::std::cmp::PartialOrd
    for Exposures<AccountId, Power>
{
    #[inline]
    fn partial_cmp(
        &self,
        other: &Exposures<AccountId, Power>,
    ) -> ::std::option::Option<::std::cmp::Ordering> {
        match *other {
            Exposures {
                total: ref __self_1_0,
                own: ref __self_1_1,
                others: ref __self_1_2,
            } => match *self {
                Exposures {
                    total: ref __self_0_0,
                    own: ref __self_0_1,
                    others: ref __self_0_2,
                } => match ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)) {
                    ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {
                        match ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_1), &(*__self_1_1)) {
                            ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {
                                match ::std::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_2),
                                    &(*__self_1_2),
                                ) {
                                    ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {
                                        ::std::option::Option::Some(::std::cmp::Ordering::Equal)
                                    }
                                    cmp => cmp,
                                }
                            }
                            cmp => cmp,
                        }
                    }
                    cmp => cmp,
                },
            },
        }
    }
    #[inline]
    fn lt(&self, other: &Exposures<AccountId, Power>) -> bool {
        match *other {
            Exposures {
                total: ref __self_1_0,
                own: ref __self_1_1,
                others: ref __self_1_2,
            } => match *self {
                Exposures {
                    total: ref __self_0_0,
                    own: ref __self_0_1,
                    others: ref __self_0_2,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::cmp::Ordering::then_with(
                                ::std::option::Option::unwrap_or(
                                    ::std::cmp::PartialOrd::partial_cmp(
                                        &(*__self_0_1),
                                        &(*__self_1_1),
                                    ),
                                    ::std::cmp::Ordering::Equal,
                                ),
                                || {
                                    ::std::option::Option::unwrap_or(
                                        ::std::cmp::PartialOrd::partial_cmp(
                                            &(*__self_0_2),
                                            &(*__self_1_2),
                                        ),
                                        ::std::cmp::Ordering::Greater,
                                    )
                                },
                            )
                        },
                    ) == ::std::cmp::Ordering::Less
                }
            },
        }
    }
    #[inline]
    fn le(&self, other: &Exposures<AccountId, Power>) -> bool {
        match *other {
            Exposures {
                total: ref __self_1_0,
                own: ref __self_1_1,
                others: ref __self_1_2,
            } => match *self {
                Exposures {
                    total: ref __self_0_0,
                    own: ref __self_0_1,
                    others: ref __self_0_2,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::cmp::Ordering::then_with(
                                ::std::option::Option::unwrap_or(
                                    ::std::cmp::PartialOrd::partial_cmp(
                                        &(*__self_0_1),
                                        &(*__self_1_1),
                                    ),
                                    ::std::cmp::Ordering::Equal,
                                ),
                                || {
                                    ::std::option::Option::unwrap_or(
                                        ::std::cmp::PartialOrd::partial_cmp(
                                            &(*__self_0_2),
                                            &(*__self_1_2),
                                        ),
                                        ::std::cmp::Ordering::Greater,
                                    )
                                },
                            )
                        },
                    ) != ::std::cmp::Ordering::Greater
                }
            },
        }
    }
    #[inline]
    fn gt(&self, other: &Exposures<AccountId, Power>) -> bool {
        match *other {
            Exposures {
                total: ref __self_1_0,
                own: ref __self_1_1,
                others: ref __self_1_2,
            } => match *self {
                Exposures {
                    total: ref __self_0_0,
                    own: ref __self_0_1,
                    others: ref __self_0_2,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::cmp::Ordering::then_with(
                                ::std::option::Option::unwrap_or(
                                    ::std::cmp::PartialOrd::partial_cmp(
                                        &(*__self_0_1),
                                        &(*__self_1_1),
                                    ),
                                    ::std::cmp::Ordering::Equal,
                                ),
                                || {
                                    ::std::option::Option::unwrap_or(
                                        ::std::cmp::PartialOrd::partial_cmp(
                                            &(*__self_0_2),
                                            &(*__self_1_2),
                                        ),
                                        ::std::cmp::Ordering::Less,
                                    )
                                },
                            )
                        },
                    ) == ::std::cmp::Ordering::Greater
                }
            },
        }
    }
    #[inline]
    fn ge(&self, other: &Exposures<AccountId, Power>) -> bool {
        match *other {
            Exposures {
                total: ref __self_1_0,
                own: ref __self_1_1,
                others: ref __self_1_2,
            } => match *self {
                Exposures {
                    total: ref __self_0_0,
                    own: ref __self_0_1,
                    others: ref __self_0_2,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::cmp::Ordering::then_with(
                                ::std::option::Option::unwrap_or(
                                    ::std::cmp::PartialOrd::partial_cmp(
                                        &(*__self_0_1),
                                        &(*__self_1_1),
                                    ),
                                    ::std::cmp::Ordering::Equal,
                                ),
                                || {
                                    ::std::option::Option::unwrap_or(
                                        ::std::cmp::PartialOrd::partial_cmp(
                                            &(*__self_0_2),
                                            &(*__self_1_2),
                                        ),
                                        ::std::cmp::Ordering::Less,
                                    )
                                },
                            )
                        },
                    ) != ::std::cmp::Ordering::Less
                }
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::cmp::Ord, Power: ::std::cmp::Ord> ::std::cmp::Ord
    for Exposures<AccountId, Power>
{
    #[inline]
    fn cmp(&self, other: &Exposures<AccountId, Power>) -> ::std::cmp::Ordering {
        match *other {
            Exposures {
                total: ref __self_1_0,
                own: ref __self_1_1,
                others: ref __self_1_2,
            } => match *self {
                Exposures {
                    total: ref __self_0_0,
                    own: ref __self_0_1,
                    others: ref __self_0_2,
                } => match ::std::cmp::Ord::cmp(&(*__self_0_0), &(*__self_1_0)) {
                    ::std::cmp::Ordering::Equal => {
                        match ::std::cmp::Ord::cmp(&(*__self_0_1), &(*__self_1_1)) {
                            ::std::cmp::Ordering::Equal => {
                                match ::std::cmp::Ord::cmp(&(*__self_0_2), &(*__self_1_2)) {
                                    ::std::cmp::Ordering::Equal => ::std::cmp::Ordering::Equal,
                                    cmp => cmp,
                                }
                            }
                            cmp => cmp,
                        }
                    }
                    cmp => cmp,
                },
            },
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_Exposures: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<AccountId, Power> _parity_codec::Encode for Exposures<AccountId, Power>
    where
        Power: _parity_codec::Encode,
        Power: _parity_codec::Encode,
        Power: _parity_codec::Encode,
        Power: _parity_codec::Encode,
        Vec<IndividualExpo<AccountId, Power>>: _parity_codec::Encode,
        Vec<IndividualExpo<AccountId, Power>>: _parity_codec::Encode,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            dest.push(&self.total);
            dest.push(&self.own);
            dest.push(&self.others);
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_Exposures: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<AccountId, Power> _parity_codec::Decode for Exposures<AccountId, Power>
    where
        Power: _parity_codec::Decode,
        Power: _parity_codec::Decode,
        Power: _parity_codec::Decode,
        Power: _parity_codec::Decode,
        Vec<IndividualExpo<AccountId, Power>>: _parity_codec::Decode,
        Vec<IndividualExpo<AccountId, Power>>: _parity_codec::Decode,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            Some(Exposures {
                total: _parity_codec::Decode::decode(input)?,
                own: _parity_codec::Decode::decode(input)?,
                others: _parity_codec::Decode::decode(input)?,
            })
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::fmt::Debug, Power: ::std::fmt::Debug> ::std::fmt::Debug
    for Exposures<AccountId, Power>
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Exposures {
                total: ref __self_0_0,
                own: ref __self_0_1,
                others: ref __self_0_2,
            } => {
                let mut debug_trait_builder = f.debug_struct("Exposures");
                let _ = debug_trait_builder.field("total", &&(*__self_0_0));
                let _ = debug_trait_builder.field("own", &&(*__self_0_1));
                let _ = debug_trait_builder.field("others", &&(*__self_0_2));
                debug_trait_builder.finish()
            }
        }
    }
}
type RingBalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;
type KtonBalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::Balance;
type RingPositiveImbalanceOf<T> =
    <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type RingNegativeImbalanceOf<T> =
    <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
type KtonPositiveImbalanceOf<T> =
    <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type KtonNegativeImbalanceOf<T> =
    <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
type RawAssignment<T> = (<T as system::Trait>::AccountId, ExtendedBalance);
type ExpoMap<T> = BTreeMap<
    <T as system::Trait>::AccountId,
    Exposures<<T as system::Trait>::AccountId, ExtendedBalance>,
>;
pub trait Trait: timestamp::Trait + session::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Ring: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
    /// Handler for the unbalanced reduction when slashing a staker.
    type RingSlash: OnUnbalanced<RingNegativeImbalanceOf<Self>>;
    /// Handler for the unbalanced increment when rewarding a staker.
    type RingReward: OnUnbalanced<RingPositiveImbalanceOf<Self>>;
    type Kton: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
    type KtonSlash: OnUnbalanced<KtonNegativeImbalanceOf<Self>>;
    type KtonReward: OnUnbalanced<KtonPositiveImbalanceOf<Self>>;
    type CurrencyToVote: Convert<KtonBalanceOf<Self>, u64> + Convert<u128, KtonBalanceOf<Self>>;
    /// Number of sessions per era.
    type SessionsPerEra: Get<SessionIndex>;
    /// Number of eras that staked funds must remain bonded for.
    type BondingDuration: Get<EraIndex>;
    type Cap: Get<<Self::Ring as Currency<Self::AccountId>>::Balance>;
    type ErasPerEpoch: Get<EraIndex>;
}
#[doc(hidden)]
mod sr_api_hidden_includes_decl_storage {
    pub extern crate srml_support as hidden_include;
}
/// Tag a type as an instance of a module.
///
/// Defines storage prefixes, they must be unique.
#[doc(hidden)]
pub trait __GeneratedInstantiable: 'static {
    const PREFIX_FOR_ValidatorCount: &'static str;
    const PREFIX_FOR_MinimumValidatorCount: &'static str;
    const PREFIX_FOR_SessionReward: &'static str;
    const PREFIX_FOR_OfflineSlash: &'static str;
    const PREFIX_FOR_OfflineSlashGrace: &'static str;
    const PREFIX_FOR_Invulnerables: &'static str;
    const PREFIX_FOR_Bonded: &'static str;
    const PREFIX_FOR_Ledger: &'static str;
    const PREFIX_FOR_Payee: &'static str;
    const PREFIX_FOR_Validators: &'static str;
    const HEAD_KEY_FOR_Validators: &'static str;
    const PREFIX_FOR_Nominators: &'static str;
    const HEAD_KEY_FOR_Nominators: &'static str;
    const PREFIX_FOR_Stakers: &'static str;
    const PREFIX_FOR_CurrentElected: &'static str;
    const PREFIX_FOR_CurrentEra: &'static str;
    const PREFIX_FOR_SlotStake: &'static str;
    const PREFIX_FOR_SlashCount: &'static str;
    const PREFIX_FOR_RecentlyOffline: &'static str;
    const PREFIX_FOR_ForceNewEra: &'static str;
    const PREFIX_FOR_EpochIndex: &'static str;
    const PREFIX_FOR_CurrentEraTotalReward: &'static str;
    const PREFIX_FOR_NodeName: &'static str;
    const PREFIX_FOR_RingPool: &'static str;
    const PREFIX_FOR_KtonPool: &'static str;
}
#[doc(hidden)]
#[structural_match]
pub struct __InherentHiddenInstance;
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for __InherentHiddenInstance {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            __InherentHiddenInstance => {
                let mut debug_trait_builder = f.debug_tuple("__InherentHiddenInstance");
                debug_trait_builder.finish()
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::clone::Clone for __InherentHiddenInstance {
    #[inline]
    fn clone(&self) -> __InherentHiddenInstance {
        match *self {
            __InherentHiddenInstance => __InherentHiddenInstance,
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::Eq for __InherentHiddenInstance {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {}
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::PartialEq for __InherentHiddenInstance {
    #[inline]
    fn eq(&self, other: &__InherentHiddenInstance) -> bool {
        match *other {
            __InherentHiddenInstance => match *self {
                __InherentHiddenInstance => true,
            },
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR___InherentHiddenInstance: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl _parity_codec::Encode for __InherentHiddenInstance {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            drop(dest);
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR___InherentHiddenInstance: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl _parity_codec::Decode for __InherentHiddenInstance {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            drop(input);
            Some(__InherentHiddenInstance)
        }
    }
};
impl __GeneratedInstantiable for __InherentHiddenInstance {
    const PREFIX_FOR_ValidatorCount: &'static str = "Staking ValidatorCount";
    const PREFIX_FOR_MinimumValidatorCount: &'static str = "Staking MinimumValidatorCount";
    const PREFIX_FOR_SessionReward: &'static str = "Staking SessionReward";
    const PREFIX_FOR_OfflineSlash: &'static str = "Staking OfflineSlash";
    const PREFIX_FOR_OfflineSlashGrace: &'static str = "Staking OfflineSlashGrace";
    const PREFIX_FOR_Invulnerables: &'static str = "Staking Invulnerables";
    const PREFIX_FOR_Bonded: &'static str = "Staking Bonded";
    const PREFIX_FOR_Ledger: &'static str = "Staking Ledger";
    const PREFIX_FOR_Payee: &'static str = "Staking Payee";
    const PREFIX_FOR_Validators: &'static str = "Staking Validators";
    const HEAD_KEY_FOR_Validators: &'static str = "head of Staking Validators";
    const PREFIX_FOR_Nominators: &'static str = "Staking Nominators";
    const HEAD_KEY_FOR_Nominators: &'static str = "head of Staking Nominators";
    const PREFIX_FOR_Stakers: &'static str = "Staking Stakers";
    const PREFIX_FOR_CurrentElected: &'static str = "Staking CurrentElected";
    const PREFIX_FOR_CurrentEra: &'static str = "Staking CurrentEra";
    const PREFIX_FOR_SlotStake: &'static str = "Staking SlotStake";
    const PREFIX_FOR_SlashCount: &'static str = "Staking SlashCount";
    const PREFIX_FOR_RecentlyOffline: &'static str = "Staking RecentlyOffline";
    const PREFIX_FOR_ForceNewEra: &'static str = "Staking ForceNewEra";
    const PREFIX_FOR_EpochIndex: &'static str = "Staking EpochIndex";
    const PREFIX_FOR_CurrentEraTotalReward: &'static str = "Staking CurrentEraTotalReward";
    const PREFIX_FOR_NodeName: &'static str = "Staking NodeName";
    const PREFIX_FOR_RingPool: &'static str = "Staking RingPool";
    const PREFIX_FOR_KtonPool: &'static str = "Staking KtonPool";
}
pub struct ValidatorCount(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<()>,
);
impl self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > for ValidatorCount < > { type Query = u32 ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking ValidatorCount" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: put ( & val , storage ) ; ret } }
pub struct MinimumValidatorCount(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<()>,
);
impl self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > for MinimumValidatorCount < > { type Query = u32 ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking MinimumValidatorCount" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: key ( ) ) . unwrap_or_else ( | | DEFAULT_MINIMUM_VALIDATOR_COUNT ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: key ( ) ) . unwrap_or_else ( | | DEFAULT_MINIMUM_VALIDATOR_COUNT ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: put ( & val , storage ) ; ret } }
pub struct SessionReward(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<()>,
);
impl self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > for SessionReward < > { type Query = Perbill ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking SessionReward" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: key ( ) ) . unwrap_or_else ( | | Perbill :: from_percent ( 60 ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: key ( ) ) . unwrap_or_else ( | | Perbill :: from_percent ( 60 ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: put ( & val , storage ) ; ret } }
pub struct OfflineSlash(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<()>,
);
impl self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > for OfflineSlash < > { type Query = Perbill ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking OfflineSlash" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: key ( ) ) . unwrap_or_else ( | | Perbill :: from_parts ( 1000 ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: key ( ) ) . unwrap_or_else ( | | Perbill :: from_parts ( 1000 ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: put ( & val , storage ) ; ret } }
pub struct OfflineSlashGrace(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<()>,
);
impl self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > for OfflineSlashGrace < > { type Query = u32 ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking OfflineSlashGrace" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: put ( & val , storage ) ; ret } }
pub struct Invulnerables<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > for Invulnerables < T > { type Query = Vec < T :: AccountId > ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking Invulnerables" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > > :: put ( & val , storage ) ; ret } }
pub struct Bonded<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: AccountId > for Bonded < T > { type Query = Option < T :: AccountId > ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Staking Bonded" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( x : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: AccountId > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( x , & mut key ) ; key } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: AccountId > > :: key_for ( key ) ; storage . get ( & key [ .. ] ) . or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: AccountId > > :: key_for ( key ) ; storage . take ( & key [ .. ] ) . or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: AccountId > > :: get ( key , storage ) ; let ret = f ( & mut val ) ; match val { Some ( ref val ) => < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: AccountId > > :: insert ( key , & val , storage ) , None => < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: AccountId > > :: remove ( key , storage ) , } ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: AppendableStorageMap < T :: AccountId , T :: AccountId > for Bonded < T > { }
pub struct Ledger<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > for Ledger < T > { type Query = Option < StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Staking Ledger" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( x : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( x , & mut key ) ; key } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > > :: key_for ( key ) ; storage . get ( & key [ .. ] ) . or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > > :: key_for ( key ) ; storage . take ( & key [ .. ] ) . or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > > :: get ( key , storage ) ; let ret = f ( & mut val ) ; match val { Some ( ref val ) => < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > > :: insert ( key , & val , storage ) , None => < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > > :: remove ( key , storage ) , } ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: AppendableStorageMap < T :: AccountId , StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > for Ledger < T > { }
pub struct Payee<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , RewardDestination > for Payee < T > { type Query = RewardDestination ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Staking Payee" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( x : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , RewardDestination > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( x , & mut key ) ; key } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , RewardDestination > > :: key_for ( key ) ; storage . get ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , RewardDestination > > :: key_for ( key ) ; storage . take ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , RewardDestination > > :: get ( key , storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , RewardDestination > > :: insert ( key , & val , storage ) ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: AppendableStorageMap < T :: AccountId , RewardDestination > for Payee < T > { }
/// Linkage data of an element (it's successor and predecessor)
pub(crate) struct __LinkageForValidatorsDoNotUse<Key> {
    /// Previous element key in storage (None for the first element)
    pub previous: Option<Key>,
    /// Next element key in storage (None for the last element)
    pub next: Option<Key>,
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR___LinkageForValidatorsDoNotUse: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<Key> _parity_codec::Encode for __LinkageForValidatorsDoNotUse<Key>
    where
        Option<Key>: _parity_codec::Encode,
        Option<Key>: _parity_codec::Encode,
        Option<Key>: _parity_codec::Encode,
        Option<Key>: _parity_codec::Encode,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            dest.push(&self.previous);
            dest.push(&self.next);
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR___LinkageForValidatorsDoNotUse: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<Key> _parity_codec::Decode for __LinkageForValidatorsDoNotUse<Key>
    where
        Option<Key>: _parity_codec::Decode,
        Option<Key>: _parity_codec::Decode,
        Option<Key>: _parity_codec::Decode,
        Option<Key>: _parity_codec::Decode,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            Some(__LinkageForValidatorsDoNotUse {
                previous: _parity_codec::Decode::decode(input)?,
                next: _parity_codec::Decode::decode(input)?,
            })
        }
    }
};
mod __linked_map_details_for_validators_do_not_use {
    /// Re-exported version of linkage to overcome proc-macro derivation issue.
    pub(crate) use super::__LinkageForValidatorsDoNotUse as Linkage;
    use super::*;
    impl<Key> Default for Linkage<Key> {
        fn default() -> Self {
            Self {
                previous: None,
                next: None,
            }
        }
    }
    /// A key-value pair iterator for enumerable map.
    pub(crate) struct Enumerator<'a, S, K, V> {
        pub storage: &'a S,
        pub next: Option<K>,
        pub _data:
            self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<V>,
    }
    impl<
            'a,
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >,
            T: Trait,
        > Iterator for Enumerator<'a, S, T::AccountId, (ValidatorPrefs, T)>
    {
        type Item = (T::AccountId, ValidatorPrefs);
        fn next(&mut self) -> Option<Self::Item> {
            let next = self.next.take()?;
            let key_for = < super :: Validators < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: key_for ( & next ) ;
            let ( val , linkage ) : ( ValidatorPrefs , Linkage < T :: AccountId > ) = self . storage . get ( & * key_for ) . expect ( "previous/next only contain existing entires; we enumerate using next; entry exists; qed" ) ;
            self.next = linkage.next;
            Some((next, val))
        }
    }
    pub(crate) trait Utils<T: Trait> {
        /// Update linkage when this element is removed.
        ///
        /// Takes care of updating previous and next elements points
        /// as well as updates head if the element is first or last.
        fn remove_linkage<
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >,
        >(
            linkage: Linkage<T::AccountId>,
            storage: &mut S,
        );
        /// Read the contained data and it's linkage.
        fn read_with_linkage<S>(
            storage: &S,
            key: &[u8],
        ) -> Option<(ValidatorPrefs, Linkage<T::AccountId>)>
        where
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >;
        /// Generate linkage for newly inserted element.
        ///
        /// Takes care of updating head and previous head's pointer.
        fn new_head_linkage<
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >,
        >(
            storage: &mut S,
            key: &T::AccountId,
        ) -> Linkage<T::AccountId>;
        /// Read current head pointer.
        fn read_head<
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >,
        >(
            storage: &S,
        ) -> Option<T::AccountId>;
        /// Overwrite current head pointer.
        ///
        /// If `None` is given head is removed from storage.
        fn write_head<
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >,
        >(
            storage: &mut S,
            head: Option<&T::AccountId>,
        );
    }
}
pub struct Validators<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl<T: Trait> self::__linked_map_details_for_validators_do_not_use::Utils<T> for Validators<T> {
    fn remove_linkage<
        S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
            self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
        >,
    >(
        linkage: self::__linked_map_details_for_validators_do_not_use::Linkage<T::AccountId>,
        storage: &mut S,
    ) {
        use self::__linked_map_details_for_validators_do_not_use::Utils;
        let next_key = linkage . next . as_ref ( ) . map ( | x | < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: key_for ( x ) ) ;
        let prev_key = linkage . previous . as_ref ( ) . map ( | x | < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: key_for ( x ) ) ;
        if let Some(prev_key) = prev_key {
            let mut res = Self :: read_with_linkage ( storage , & * prev_key ) . expect ( "Linkage is updated in case entry is removed; it always points to existing keys; qed" ) ;
            res.1.next = linkage.next;
            storage.put(&*prev_key, &res);
        } else {
            Self::write_head(storage, linkage.next.as_ref());
        }
        if let Some(next_key) = next_key {
            let mut res = Self :: read_with_linkage ( storage , & * next_key ) . expect ( "Linkage is updated in case entry is removed; it always points to existing keys; qed" ) ;
            res.1.previous = linkage.previous;
            storage.put(&*next_key, &res);
        }
    }
    fn read_with_linkage<
        S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
            self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
        >,
    >(
        storage: &S,
        key: &[u8],
    ) -> Option<(
        ValidatorPrefs,
        self::__linked_map_details_for_validators_do_not_use::Linkage<T::AccountId>,
    )> {
        storage.get(key)
    }
    fn new_head_linkage<
        S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
            self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
        >,
    >(
        storage: &mut S,
        key: &T::AccountId,
    ) -> self::__linked_map_details_for_validators_do_not_use::Linkage<T::AccountId> {
        use self::__linked_map_details_for_validators_do_not_use::Utils;
        if let Some(head) = Self::read_head(storage) {
            {
                let head_key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: key_for ( & head ) ;
                let (data, linkage) = Self::read_with_linkage(storage, &*head_key).expect(
                    r#"
								head is set when first element is inserted and unset when last element is removed;
								if head is Some then it points to existing key; qed
							"#,
                );
                storage.put(
                    &*head_key,
                    &(
                        data,
                        self::__linked_map_details_for_validators_do_not_use::Linkage {
                            next: linkage.next.as_ref(),
                            previous: Some(key),
                        },
                    ),
                );
            }
            Self::write_head(storage, Some(key));
            let mut linkage =
                self::__linked_map_details_for_validators_do_not_use::Linkage::default();
            linkage.next = Some(head);
            linkage
        } else {
            Self::write_head(storage, Some(key));
            self::__linked_map_details_for_validators_do_not_use::Linkage::default()
        }
    }
    fn read_head<
        S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
            self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
        >,
    >(
        storage: &S,
    ) -> Option<T::AccountId> {
        storage.get("head of Staking Validators".as_bytes())
    }
    fn write_head<
        S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
            self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
        >,
    >(
        storage: &mut S,
        head: Option<&T::AccountId>,
    ) {
        match head {
            Some(head) => storage.put("head of Staking Validators".as_bytes(), head),
            None => storage.kill("head of Staking Validators".as_bytes()),
        }
    }
}
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > for Validators < T > { type Query = ValidatorPrefs ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Staking Validators" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( key : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key_for = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( & key , & mut key_for ) ; key_for } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { storage . get ( & * < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: key_for ( key ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { use self :: __linked_map_details_for_validators_do_not_use :: Utils ; let res : Option < ( ValidatorPrefs , self :: __linked_map_details_for_validators_do_not_use :: Linkage < T :: AccountId > ) > = storage . take ( & * < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: key_for ( key ) ) ; match res { Some ( ( data , linkage ) ) => { Self :: remove_linkage ( linkage , storage ) ; data } None => Default :: default ( ) , } } # [ doc = r" Remove the value under a key." ] fn remove < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) { < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: take ( key , storage ) ; } # [ doc = r" Store a value to be associated with the given key from the map." ] fn insert < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , val : & ValidatorPrefs , storage : & mut S ) { use self :: __linked_map_details_for_validators_do_not_use :: Utils ; let key_for = & * < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: key_for ( key ) ; let linkage = match Self :: read_with_linkage ( storage , key_for ) { Some ( ( _data , linkage ) ) => linkage , None => Self :: new_head_linkage ( storage , key ) , } ; storage . put ( key_for , & ( val , linkage ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { use self :: __linked_map_details_for_validators_do_not_use :: Utils ; let key_for = & * < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: key_for ( key ) ; let ( mut val , linkage ) = Self :: read_with_linkage ( storage , key_for ) . map ( | ( data , linkage ) | ( data , Some ( linkage ) ) ) . unwrap_or_else ( | | ( Default :: default ( ) , None ) ) ; let ret = f ( & mut val ) ; match linkage { Some ( linkage ) => storage . put ( key_for , & ( val , linkage ) ) , None => < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: insert ( key , & val , storage ) , } ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: EnumerableStorageMap < T :: AccountId , ValidatorPrefs > for Validators < T > where T : 'static { fn head < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( storage : & S ) -> Option < T :: AccountId > { use self :: __linked_map_details_for_validators_do_not_use :: Utils ; Self :: read_head ( storage ) } fn enumerate < 'a , S > ( storage : & 'a S ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: boxed :: Box < dyn Iterator < Item = ( T :: AccountId , ValidatorPrefs ) > + 'a > where S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > , T :: AccountId : 'a , ValidatorPrefs : 'a { use self :: __linked_map_details_for_validators_do_not_use :: { Utils , Enumerator } ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: boxed :: Box :: new ( Enumerator { next : Self :: read_head ( storage ) , storage , _data : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData :: < ( ValidatorPrefs , T ) > :: default ( ) , } ) } }
/// Linkage data of an element (it's successor and predecessor)
pub(crate) struct __LinkageForNominatorsDoNotUse<Key> {
    /// Previous element key in storage (None for the first element)
    pub previous: Option<Key>,
    /// Next element key in storage (None for the last element)
    pub next: Option<Key>,
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR___LinkageForNominatorsDoNotUse: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<Key> _parity_codec::Encode for __LinkageForNominatorsDoNotUse<Key>
    where
        Option<Key>: _parity_codec::Encode,
        Option<Key>: _parity_codec::Encode,
        Option<Key>: _parity_codec::Encode,
        Option<Key>: _parity_codec::Encode,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            dest.push(&self.previous);
            dest.push(&self.next);
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR___LinkageForNominatorsDoNotUse: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<Key> _parity_codec::Decode for __LinkageForNominatorsDoNotUse<Key>
    where
        Option<Key>: _parity_codec::Decode,
        Option<Key>: _parity_codec::Decode,
        Option<Key>: _parity_codec::Decode,
        Option<Key>: _parity_codec::Decode,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            Some(__LinkageForNominatorsDoNotUse {
                previous: _parity_codec::Decode::decode(input)?,
                next: _parity_codec::Decode::decode(input)?,
            })
        }
    }
};
mod __linked_map_details_for_nominators_do_not_use {
    /// Re-exported version of linkage to overcome proc-macro derivation issue.
    pub(crate) use super::__LinkageForNominatorsDoNotUse as Linkage;
    use super::*;
    impl<Key> Default for Linkage<Key> {
        fn default() -> Self {
            Self {
                previous: None,
                next: None,
            }
        }
    }
    /// A key-value pair iterator for enumerable map.
    pub(crate) struct Enumerator<'a, S, K, V> {
        pub storage: &'a S,
        pub next: Option<K>,
        pub _data:
            self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<V>,
    }
    impl<
            'a,
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >,
            T: Trait,
        > Iterator for Enumerator<'a, S, T::AccountId, (Vec<T::AccountId>, T)>
    {
        type Item = (T::AccountId, Vec<T::AccountId>);
        fn next(&mut self) -> Option<Self::Item> {
            let next = self.next.take()?;
            let key_for = < super :: Nominators < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: key_for ( & next ) ;
            let ( val , linkage ) : ( Vec < T :: AccountId > , Linkage < T :: AccountId > ) = self . storage . get ( & * key_for ) . expect ( "previous/next only contain existing entires; we enumerate using next; entry exists; qed" ) ;
            self.next = linkage.next;
            Some((next, val))
        }
    }
    pub(crate) trait Utils<T: Trait> {
        /// Update linkage when this element is removed.
        ///
        /// Takes care of updating previous and next elements points
        /// as well as updates head if the element is first or last.
        fn remove_linkage<
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >,
        >(
            linkage: Linkage<T::AccountId>,
            storage: &mut S,
        );
        /// Read the contained data and it's linkage.
        fn read_with_linkage<S>(
            storage: &S,
            key: &[u8],
        ) -> Option<(Vec<T::AccountId>, Linkage<T::AccountId>)>
        where
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >;
        /// Generate linkage for newly inserted element.
        ///
        /// Takes care of updating head and previous head's pointer.
        fn new_head_linkage<
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >,
        >(
            storage: &mut S,
            key: &T::AccountId,
        ) -> Linkage<T::AccountId>;
        /// Read current head pointer.
        fn read_head<
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >,
        >(
            storage: &S,
        ) -> Option<T::AccountId>;
        /// Overwrite current head pointer.
        ///
        /// If `None` is given head is removed from storage.
        fn write_head<
            S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
                self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
            >,
        >(
            storage: &mut S,
            head: Option<&T::AccountId>,
        );
    }
}
pub struct Nominators<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl<T: Trait> self::__linked_map_details_for_nominators_do_not_use::Utils<T> for Nominators<T> {
    fn remove_linkage<
        S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
            self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
        >,
    >(
        linkage: self::__linked_map_details_for_nominators_do_not_use::Linkage<T::AccountId>,
        storage: &mut S,
    ) {
        use self::__linked_map_details_for_nominators_do_not_use::Utils;
        let next_key = linkage . next . as_ref ( ) . map ( | x | < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: key_for ( x ) ) ;
        let prev_key = linkage . previous . as_ref ( ) . map ( | x | < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: key_for ( x ) ) ;
        if let Some(prev_key) = prev_key {
            let mut res = Self :: read_with_linkage ( storage , & * prev_key ) . expect ( "Linkage is updated in case entry is removed; it always points to existing keys; qed" ) ;
            res.1.next = linkage.next;
            storage.put(&*prev_key, &res);
        } else {
            Self::write_head(storage, linkage.next.as_ref());
        }
        if let Some(next_key) = next_key {
            let mut res = Self :: read_with_linkage ( storage , & * next_key ) . expect ( "Linkage is updated in case entry is removed; it always points to existing keys; qed" ) ;
            res.1.previous = linkage.previous;
            storage.put(&*next_key, &res);
        }
    }
    fn read_with_linkage<
        S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
            self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
        >,
    >(
        storage: &S,
        key: &[u8],
    ) -> Option<(
        Vec<T::AccountId>,
        self::__linked_map_details_for_nominators_do_not_use::Linkage<T::AccountId>,
    )> {
        storage.get(key)
    }
    fn new_head_linkage<
        S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
            self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
        >,
    >(
        storage: &mut S,
        key: &T::AccountId,
    ) -> self::__linked_map_details_for_nominators_do_not_use::Linkage<T::AccountId> {
        use self::__linked_map_details_for_nominators_do_not_use::Utils;
        if let Some(head) = Self::read_head(storage) {
            {
                let head_key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: key_for ( & head ) ;
                let (data, linkage) = Self::read_with_linkage(storage, &*head_key).expect(
                    r#"
								head is set when first element is inserted and unset when last element is removed;
								if head is Some then it points to existing key; qed
							"#,
                );
                storage.put(
                    &*head_key,
                    &(
                        data,
                        self::__linked_map_details_for_nominators_do_not_use::Linkage {
                            next: linkage.next.as_ref(),
                            previous: Some(key),
                        },
                    ),
                );
            }
            Self::write_head(storage, Some(key));
            let mut linkage =
                self::__linked_map_details_for_nominators_do_not_use::Linkage::default();
            linkage.next = Some(head);
            linkage
        } else {
            Self::write_head(storage, Some(key));
            self::__linked_map_details_for_nominators_do_not_use::Linkage::default()
        }
    }
    fn read_head<
        S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
            self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
        >,
    >(
        storage: &S,
    ) -> Option<T::AccountId> {
        storage.get("head of Staking Nominators".as_bytes())
    }
    fn write_head<
        S: self::sr_api_hidden_includes_decl_storage::hidden_include::HashedStorage<
            self::sr_api_hidden_includes_decl_storage::hidden_include::Blake2_256,
        >,
    >(
        storage: &mut S,
        head: Option<&T::AccountId>,
    ) {
        match head {
            Some(head) => storage.put("head of Staking Nominators".as_bytes(), head),
            None => storage.kill("head of Staking Nominators".as_bytes()),
        }
    }
}
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > for Nominators < T > { type Query = Vec < T :: AccountId > ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Staking Nominators" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( key : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key_for = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( & key , & mut key_for ) ; key_for } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { storage . get ( & * < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: key_for ( key ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { use self :: __linked_map_details_for_nominators_do_not_use :: Utils ; let res : Option < ( Vec < T :: AccountId > , self :: __linked_map_details_for_nominators_do_not_use :: Linkage < T :: AccountId > ) > = storage . take ( & * < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: key_for ( key ) ) ; match res { Some ( ( data , linkage ) ) => { Self :: remove_linkage ( linkage , storage ) ; data } None => Default :: default ( ) , } } # [ doc = r" Remove the value under a key." ] fn remove < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) { < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: take ( key , storage ) ; } # [ doc = r" Store a value to be associated with the given key from the map." ] fn insert < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , val : & Vec < T :: AccountId > , storage : & mut S ) { use self :: __linked_map_details_for_nominators_do_not_use :: Utils ; let key_for = & * < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: key_for ( key ) ; let linkage = match Self :: read_with_linkage ( storage , key_for ) { Some ( ( _data , linkage ) ) => linkage , None => Self :: new_head_linkage ( storage , key ) , } ; storage . put ( key_for , & ( val , linkage ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { use self :: __linked_map_details_for_nominators_do_not_use :: Utils ; let key_for = & * < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: key_for ( key ) ; let ( mut val , linkage ) = Self :: read_with_linkage ( storage , key_for ) . map ( | ( data , linkage ) | ( data , Some ( linkage ) ) ) . unwrap_or_else ( | | ( Default :: default ( ) , None ) ) ; let ret = f ( & mut val ) ; match linkage { Some ( linkage ) => storage . put ( key_for , & ( val , linkage ) ) , None => < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: insert ( key , & val , storage ) , } ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: EnumerableStorageMap < T :: AccountId , Vec < T :: AccountId > > for Nominators < T > where T : 'static { fn head < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( storage : & S ) -> Option < T :: AccountId > { use self :: __linked_map_details_for_nominators_do_not_use :: Utils ; Self :: read_head ( storage ) } fn enumerate < 'a , S > ( storage : & 'a S ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: boxed :: Box < dyn Iterator < Item = ( T :: AccountId , Vec < T :: AccountId > ) > + 'a > where S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > , T :: AccountId : 'a , Vec < T :: AccountId > : 'a { use self :: __linked_map_details_for_nominators_do_not_use :: { Utils , Enumerator } ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: boxed :: Box :: new ( Enumerator { next : Self :: read_head ( storage ) , storage , _data : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData :: < ( Vec < T :: AccountId > , T ) > :: default ( ) , } ) } }
pub struct Stakers<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Exposures < T :: AccountId , ExtendedBalance > > for Stakers < T > { type Query = Exposures < T :: AccountId , ExtendedBalance > ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Staking Stakers" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( x : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Exposures < T :: AccountId , ExtendedBalance > > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( x , & mut key ) ; key } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Exposures < T :: AccountId , ExtendedBalance > > > :: key_for ( key ) ; storage . get ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Exposures < T :: AccountId , ExtendedBalance > > > :: key_for ( key ) ; storage . take ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Exposures < T :: AccountId , ExtendedBalance > > > :: get ( key , storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Exposures < T :: AccountId , ExtendedBalance > > > :: insert ( key , & val , storage ) ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: AppendableStorageMap < T :: AccountId , Exposures < T :: AccountId , ExtendedBalance > > for Stakers < T > { }
pub struct CurrentElected<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > for CurrentElected < T > { type Query = Vec < T :: AccountId > ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking CurrentElected" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > > :: put ( & val , storage ) ; ret } }
pub struct CurrentEra(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<()>,
);
impl self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < EraIndex > for CurrentEra < > { type Query = EraIndex ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking CurrentEra" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < EraIndex > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < EraIndex > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < EraIndex > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < EraIndex > > :: put ( & val , storage ) ; ret } }
pub struct SlotStake(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<()>,
);
impl self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < ExtendedBalance > for SlotStake < > { type Query = ExtendedBalance ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking SlotStake" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < ExtendedBalance > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < ExtendedBalance > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < ExtendedBalance > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < ExtendedBalance > > :: put ( & val , storage ) ; ret } }
pub struct SlashCount<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , u32 > for SlashCount < T > { type Query = u32 ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Staking SlashCount" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( x : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , u32 > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( x , & mut key ) ; key } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , u32 > > :: key_for ( key ) ; storage . get ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , u32 > > :: key_for ( key ) ; storage . take ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , u32 > > :: get ( key , storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , u32 > > :: insert ( key , & val , storage ) ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: AppendableStorageMap < T :: AccountId , u32 > for SlashCount < T > { }
pub struct RecentlyOffline<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < ( T :: AccountId , T :: BlockNumber , u32 ) > > for RecentlyOffline < T > { type Query = Vec < ( T :: AccountId , T :: BlockNumber , u32 ) > ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking RecentlyOffline" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < ( T :: AccountId , T :: BlockNumber , u32 ) > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < ( T :: AccountId , T :: BlockNumber , u32 ) > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < ( T :: AccountId , T :: BlockNumber , u32 ) > > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < ( T :: AccountId , T :: BlockNumber , u32 ) > > > :: put ( & val , storage ) ; ret } }
pub struct ForceNewEra(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<()>,
);
impl self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < bool > for ForceNewEra < > { type Query = bool ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking ForceNewEra" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < bool > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < bool > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < bool > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < bool > > :: put ( & val , storage ) ; ret } }
pub struct EpochIndex(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<()>,
);
impl self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > for EpochIndex < > { type Query = u32 ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking EpochIndex" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: key ( ) ) . unwrap_or_else ( | | 0 ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: key ( ) ) . unwrap_or_else ( | | 0 ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: put ( & val , storage ) ; ret } }
/// The accumulated reward for the current era. Reset to zero at the beginning of the era
/// and increased for every successfully finished session.
pub struct CurrentEraTotalReward<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > for CurrentEraTotalReward < T > { type Query = RingBalanceOf < T > ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking CurrentEraTotalReward" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > > :: put ( & val , storage ) ; ret } }
pub struct NodeName<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < u8 > > for NodeName < T > { type Query = Vec < u8 > ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Staking NodeName" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( x : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < u8 > > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( x , & mut key ) ; key } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < u8 > > > :: key_for ( key ) ; storage . get ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < u8 > > > :: key_for ( key ) ; storage . take ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < u8 > > > :: get ( key , storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < u8 > > > :: insert ( key , & val , storage ) ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: AppendableStorageMap < T :: AccountId , Vec < u8 > > for NodeName < T > { }
pub struct RingPool<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > for RingPool < T > { type Query = RingBalanceOf < T > ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking RingPool" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > > :: put ( & val , storage ) ; ret } }
pub struct KtonPool<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < KtonBalanceOf < T > > for KtonPool < T > { type Query = KtonBalanceOf < T > ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Staking KtonPool" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < KtonBalanceOf < T > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < KtonBalanceOf < T > > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < KtonBalanceOf < T > > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < KtonBalanceOf < T > > > :: put ( & val , storage ) ; ret } }
trait Store {
    type ValidatorCount;
    type MinimumValidatorCount;
    type SessionReward;
    type OfflineSlash;
    type OfflineSlashGrace;
    type Invulnerables;
    type Bonded;
    type Ledger;
    type Payee;
    type Validators;
    type Nominators;
    type Stakers;
    type CurrentElected;
    type CurrentEra;
    type SlotStake;
    type SlashCount;
    type RecentlyOffline;
    type ForceNewEra;
    type EpochIndex;
    type CurrentEraTotalReward;
    type NodeName;
    type RingPool;
    type KtonPool;
}
#[doc(hidden)]
pub struct __GetByteStructValidatorCount<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_ValidatorCount:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructValidatorCount<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_ValidatorCount
            .get_or_init(|| {
                let def_val: u32 = Default::default();
                <u32 as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructMinimumValidatorCount<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_MinimumValidatorCount:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructMinimumValidatorCount<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_MinimumValidatorCount
            .get_or_init(|| {
                let def_val: u32 = DEFAULT_MINIMUM_VALIDATOR_COUNT;
                <u32 as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructSessionReward<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_SessionReward:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructSessionReward<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_SessionReward
            .get_or_init(|| {
                let def_val: Perbill = Perbill::from_percent(60);
                <Perbill as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructOfflineSlash<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_OfflineSlash:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructOfflineSlash<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_OfflineSlash
            .get_or_init(|| {
                let def_val: Perbill = Perbill::from_parts(1000);
                <Perbill as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructOfflineSlashGrace<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_OfflineSlashGrace:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructOfflineSlashGrace<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_OfflineSlashGrace
            .get_or_init(|| {
                let def_val: u32 = Default::default();
                <u32 as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructInvulnerables<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_Invulnerables:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructInvulnerables<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_Invulnerables
            .get_or_init(|| {
                let def_val: Vec<T::AccountId> = Default::default();
                <Vec<T::AccountId> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructBonded<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_Bonded:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructBonded<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_Bonded
            .get_or_init(|| {
                let def_val: Option<T::AccountId> = Default::default();
                <Option<T::AccountId> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructLedger<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_Ledger:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructLedger<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_Ledger . get_or_init ( | | { let def_val : Option < StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > = Default :: default ( ) ; < Option < StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > as Encode > :: encode ( & def_val ) } ) . clone ( )
    }
}
#[doc(hidden)]
pub struct __GetByteStructPayee<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_Payee:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructPayee<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_Payee
            .get_or_init(|| {
                let def_val: RewardDestination = Default::default();
                <RewardDestination as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructValidators<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_Validators:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructValidators<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_Validators
            .get_or_init(|| {
                let def_val: ValidatorPrefs = Default::default();
                <ValidatorPrefs as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructNominators<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_Nominators:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructNominators<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_Nominators
            .get_or_init(|| {
                let def_val: Vec<T::AccountId> = Default::default();
                <Vec<T::AccountId> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructStakers<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_Stakers:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructStakers<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_Stakers
            .get_or_init(|| {
                let def_val: Exposures<T::AccountId, ExtendedBalance> = Default::default();
                <Exposures<T::AccountId, ExtendedBalance> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructCurrentElected<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_CurrentElected:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructCurrentElected<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_CurrentElected
            .get_or_init(|| {
                let def_val: Vec<T::AccountId> = Default::default();
                <Vec<T::AccountId> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructCurrentEra<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_CurrentEra:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructCurrentEra<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_CurrentEra
            .get_or_init(|| {
                let def_val: EraIndex = Default::default();
                <EraIndex as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructSlotStake<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_SlotStake:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructSlotStake<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_SlotStake
            .get_or_init(|| {
                let def_val: ExtendedBalance = Default::default();
                <ExtendedBalance as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructSlashCount<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_SlashCount:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructSlashCount<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_SlashCount
            .get_or_init(|| {
                let def_val: u32 = Default::default();
                <u32 as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructRecentlyOffline<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_RecentlyOffline:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructRecentlyOffline<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_RecentlyOffline
            .get_or_init(|| {
                let def_val: Vec<(T::AccountId, T::BlockNumber, u32)> = Default::default();
                <Vec<(T::AccountId, T::BlockNumber, u32)> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructForceNewEra<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_ForceNewEra:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructForceNewEra<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_ForceNewEra
            .get_or_init(|| {
                let def_val: bool = Default::default();
                <bool as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructEpochIndex<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_EpochIndex:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructEpochIndex<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_EpochIndex
            .get_or_init(|| {
                let def_val: u32 = 0;
                <u32 as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructCurrentEraTotalReward<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_CurrentEraTotalReward:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructCurrentEraTotalReward<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_CurrentEraTotalReward
            .get_or_init(|| {
                let def_val: RingBalanceOf<T> = Default::default();
                <RingBalanceOf<T> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructNodeName<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_NodeName:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructNodeName<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_NodeName
            .get_or_init(|| {
                let def_val: Vec<u8> = Default::default();
                <Vec<u8> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructRingPool<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_RingPool:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructRingPool<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_RingPool
            .get_or_init(|| {
                let def_val: RingBalanceOf<T> = Default::default();
                <RingBalanceOf<T> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructKtonPool<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_KtonPool:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructKtonPool<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_KtonPool
            .get_or_init(|| {
                let def_val: KtonBalanceOf<T> = Default::default();
                <KtonBalanceOf<T> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
impl<T: Trait> Store for Module<T> {
    type ValidatorCount = ValidatorCount;
    type MinimumValidatorCount = MinimumValidatorCount;
    type SessionReward = SessionReward;
    type OfflineSlash = OfflineSlash;
    type OfflineSlashGrace = OfflineSlashGrace;
    type Invulnerables = Invulnerables<T>;
    type Bonded = Bonded<T>;
    type Ledger = Ledger<T>;
    type Payee = Payee<T>;
    type Validators = Validators<T>;
    type Nominators = Nominators<T>;
    type Stakers = Stakers<T>;
    type CurrentElected = CurrentElected<T>;
    type CurrentEra = CurrentEra;
    type SlotStake = SlotStake;
    type SlashCount = SlashCount<T>;
    type RecentlyOffline = RecentlyOffline<T>;
    type ForceNewEra = ForceNewEra;
    type EpochIndex = EpochIndex;
    type CurrentEraTotalReward = CurrentEraTotalReward<T>;
    type NodeName = NodeName<T>;
    type RingPool = RingPool<T>;
    type KtonPool = KtonPool<T>;
}
impl<T: 'static + Trait> Module<T> {
    pub fn validator_count() -> u32 {
        < ValidatorCount < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn minimum_validator_count() -> u32 {
        < MinimumValidatorCount < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn session_reward() -> Perbill {
        < SessionReward < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn offline_slash() -> Perbill {
        < OfflineSlash < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn offline_slash_grace() -> u32 {
        < OfflineSlashGrace < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn invulnerables() -> Vec<T::AccountId> {
        < Invulnerables < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn bonded<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> Option<T::AccountId> {
        < Bonded < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: AccountId > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn ledger<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> Option<StakingLedger<T::AccountId, T::Moment, RingBalanceOf<T>, KtonBalanceOf<T>>> {
        < Ledger < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , StakingLedger < T :: AccountId , T :: Moment , RingBalanceOf < T > , KtonBalanceOf < T > > > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn payee<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> RewardDestination {
        < Payee < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , RewardDestination > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn validator<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> ValidatorPrefs {
        < Validators < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , ValidatorPrefs > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn nominators<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> Vec<T::AccountId> {
        < Nominators < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < T :: AccountId > > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn staker<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> Exposures<T::AccountId, ExtendedBalance> {
        < Stakers < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Exposures < T :: AccountId , ExtendedBalance > > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn current_elected() -> Vec<T::AccountId> {
        < CurrentElected < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn current_era() -> EraIndex {
        < CurrentEra < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < EraIndex > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn slot_stake() -> ExtendedBalance {
        < SlotStake < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < ExtendedBalance > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn slash_count<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> u32 {
        < SlashCount < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , u32 > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn recently_offline() -> Vec<(T::AccountId, T::BlockNumber, u32)> {
        < RecentlyOffline < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < ( T :: AccountId , T :: BlockNumber , u32 ) > > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn forcing_new_era() -> bool {
        < ForceNewEra < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < bool > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn epoch_index() -> u32 {
        < EpochIndex < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    /// The accumulated reward for the current era. Reset to zero at the beginning of the era
    /// and increased for every successfully finished session.
    pub fn current_era_total_reward() -> RingBalanceOf<T> {
        < CurrentEraTotalReward < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn node_name<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> Vec<u8> {
        < NodeName < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < u8 > > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn ring_pool() -> RingBalanceOf<T> {
        < RingPool < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn kton_pool() -> KtonBalanceOf<T> {
        < KtonPool < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < KtonBalanceOf < T > > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    #[doc(hidden)]pub fn store_metadata_functions ( ) -> & 'static [ self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata ]{
        {
            & [ self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "ValidatorCount" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "u32" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructValidatorCount :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "MinimumValidatorCount" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "u32" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructMinimumValidatorCount :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "SessionReward" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Perbill" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructSessionReward :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "OfflineSlash" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Perbill" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructOfflineSlash :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "OfflineSlashGrace" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "u32" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructOfflineSlashGrace :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Invulnerables" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Vec<T::AccountId>" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructInvulnerables :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Bonded" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Optional , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , is_linked : false , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructBonded :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Ledger" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Optional , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "StakingLedger<T::AccountId, T::Moment, RingBalanceOf<T>,\nKtonBalanceOf<T>,>," ) , is_linked : false , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructLedger :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Payee" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "RewardDestination" ) , is_linked : false , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructPayee :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Validators" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "ValidatorPrefs" ) , is_linked : true , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructValidators :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Nominators" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Vec<T::AccountId>" ) , is_linked : true , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructNominators :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Stakers" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Exposures<T::AccountId, ExtendedBalance>" ) , is_linked : false , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructStakers :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "CurrentElected" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Vec<T::AccountId>" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructCurrentElected :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "CurrentEra" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "EraIndex" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructCurrentEra :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "SlotStake" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "ExtendedBalance" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructSlotStake :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "SlashCount" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "u32" ) , is_linked : false , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructSlashCount :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "RecentlyOffline" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Vec<(T::AccountId, T::BlockNumber, u32)>" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructRecentlyOffline :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "ForceNewEra" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "bool" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructForceNewEra :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "EpochIndex" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "u32" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructEpochIndex :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "CurrentEraTotalReward" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "RingBalanceOf<T>" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructCurrentEraTotalReward :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ " The accumulated reward for the current era. Reset to zero at the beginning of the era" , " and increased for every successfully finished session." ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "NodeName" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Vec<u8>" ) , is_linked : false , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructNodeName :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "RingPool" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "RingBalanceOf<T>" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructRingPool :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "KtonPool" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "KtonBalanceOf<T>" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructKtonPool :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } ]
        }
    }
    #[doc(hidden)]
    pub fn store_metadata_name() -> &'static str {
        "Staking"
    }
}
#[cfg(feature = "std")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(bound(
    serialize = "u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::Serialize, u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::Serialize, Perbill : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::Serialize, Perbill : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::Serialize, u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::Serialize, Vec < T :: AccountId > : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::Serialize, EraIndex : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::Serialize, RingBalanceOf < T > : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::Serialize, Vec < (\nT :: AccountId , T :: AccountId , RingBalanceOf < T > , StakerStatus < T ::\nAccountId > ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::Serialize, "
))]
#[serde(bound(
    deserialize = "u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::de::DeserializeOwned, u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::de::DeserializeOwned, Perbill : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::de::DeserializeOwned, Perbill : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::de::DeserializeOwned, u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::de::DeserializeOwned, Vec < T :: AccountId > : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::de::DeserializeOwned, EraIndex : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::de::DeserializeOwned, RingBalanceOf < T > : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::de::DeserializeOwned, Vec < (\nT :: AccountId , T :: AccountId , RingBalanceOf < T > , StakerStatus < T ::\nAccountId > ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::de::DeserializeOwned, "
))]
pub struct GenesisConfig<T: Trait> {
    pub validator_count: u32,
    pub minimum_validator_count: u32,
    pub session_reward: Perbill,
    pub offline_slash: Perbill,
    pub offline_slash_grace: u32,
    pub invulnerables: Vec<T::AccountId>,
    pub current_era: EraIndex,
    /// The accumulated reward for the current era. Reset to zero at the beginning of the era
    /// and increased for every successfully finished session.
    pub current_era_total_reward: RingBalanceOf<T>,
    pub stakers: Vec<(
        T::AccountId,
        T::AccountId,
        RingBalanceOf<T>,
        StakerStatus<T::AccountId>,
    )>,
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_SERIALIZE_FOR_GenesisConfig: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<T: Trait> _serde::Serialize for GenesisConfig<T>
    where
        u32: self::sr_api_hidden_includes_decl_storage::hidden_include::serde::Serialize,
        u32: self::sr_api_hidden_includes_decl_storage::hidden_include::serde::Serialize,
        Perbill: self::sr_api_hidden_includes_decl_storage::hidden_include::serde::Serialize,
        Perbill: self::sr_api_hidden_includes_decl_storage::hidden_include::serde::Serialize,
        u32: self::sr_api_hidden_includes_decl_storage::hidden_include::serde::Serialize,
        Vec<T::AccountId>:
            self::sr_api_hidden_includes_decl_storage::hidden_include::serde::Serialize,
        EraIndex: self::sr_api_hidden_includes_decl_storage::hidden_include::serde::Serialize,
        RingBalanceOf<T>:
            self::sr_api_hidden_includes_decl_storage::hidden_include::serde::Serialize,
        Vec<(
            T::AccountId,
            T::AccountId,
            RingBalanceOf<T>,
            StakerStatus<T::AccountId>,
        )>: self::sr_api_hidden_includes_decl_storage::hidden_include::serde::Serialize,
    {
        fn serialize<__S>(&self, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = match _serde::Serializer::serialize_struct(
                __serializer,
                "GenesisConfig",
                false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "validatorCount",
                &self.validator_count,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "minimumValidatorCount",
                &self.minimum_validator_count,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "sessionReward",
                &self.session_reward,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "offlineSlash",
                &self.offline_slash,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "offlineSlashGrace",
                &self.offline_slash_grace,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "invulnerables",
                &self.invulnerables,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "currentEra",
                &self.current_era,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "currentEraTotalReward",
                &self.current_era_total_reward,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "stakers",
                &self.stakers,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_GenesisConfig: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de, T: Trait> _serde::Deserialize<'de> for GenesisConfig<T>
    where
        u32: self::sr_api_hidden_includes_decl_storage::hidden_include::serde::de::DeserializeOwned,
        u32: self::sr_api_hidden_includes_decl_storage::hidden_include::serde::de::DeserializeOwned,
        Perbill:
            self::sr_api_hidden_includes_decl_storage::hidden_include::serde::de::DeserializeOwned,
        Perbill:
            self::sr_api_hidden_includes_decl_storage::hidden_include::serde::de::DeserializeOwned,
        u32: self::sr_api_hidden_includes_decl_storage::hidden_include::serde::de::DeserializeOwned,
        Vec<T::AccountId>:
            self::sr_api_hidden_includes_decl_storage::hidden_include::serde::de::DeserializeOwned,
        EraIndex:
            self::sr_api_hidden_includes_decl_storage::hidden_include::serde::de::DeserializeOwned,
        RingBalanceOf<T>:
            self::sr_api_hidden_includes_decl_storage::hidden_include::serde::de::DeserializeOwned,
        Vec<(
            T::AccountId,
            T::AccountId,
            RingBalanceOf<T>,
            StakerStatus<T::AccountId>,
        )>: self::sr_api_hidden_includes_decl_storage::hidden_include::serde::de::DeserializeOwned,
    {
        fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __field4,
                __field5,
                __field6,
                __field7,
                __field8,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::export::Ok(__Field::__field0),
                        1u64 => _serde::export::Ok(__Field::__field1),
                        2u64 => _serde::export::Ok(__Field::__field2),
                        3u64 => _serde::export::Ok(__Field::__field3),
                        4u64 => _serde::export::Ok(__Field::__field4),
                        5u64 => _serde::export::Ok(__Field::__field5),
                        6u64 => _serde::export::Ok(__Field::__field6),
                        7u64 => _serde::export::Ok(__Field::__field7),
                        8u64 => _serde::export::Ok(__Field::__field8),
                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                            _serde::de::Unexpected::Unsigned(__value),
                            &"field index 0 <= i < 9",
                        )),
                    }
                }
                fn visit_str<__E>(self, __value: &str) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "validatorCount" => _serde::export::Ok(__Field::__field0),
                        "minimumValidatorCount" => _serde::export::Ok(__Field::__field1),
                        "sessionReward" => _serde::export::Ok(__Field::__field2),
                        "offlineSlash" => _serde::export::Ok(__Field::__field3),
                        "offlineSlashGrace" => _serde::export::Ok(__Field::__field4),
                        "invulnerables" => _serde::export::Ok(__Field::__field5),
                        "currentEra" => _serde::export::Ok(__Field::__field6),
                        "currentEraTotalReward" => _serde::export::Ok(__Field::__field7),
                        "stakers" => _serde::export::Ok(__Field::__field8),
                        _ => _serde::export::Err(_serde::de::Error::unknown_field(__value, FIELDS)),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"validatorCount" => _serde::export::Ok(__Field::__field0),
                        b"minimumValidatorCount" => _serde::export::Ok(__Field::__field1),
                        b"sessionReward" => _serde::export::Ok(__Field::__field2),
                        b"offlineSlash" => _serde::export::Ok(__Field::__field3),
                        b"offlineSlashGrace" => _serde::export::Ok(__Field::__field4),
                        b"invulnerables" => _serde::export::Ok(__Field::__field5),
                        b"currentEra" => _serde::export::Ok(__Field::__field6),
                        b"currentEraTotalReward" => _serde::export::Ok(__Field::__field7),
                        b"stakers" => _serde::export::Ok(__Field::__field8),
                        _ => {
                            let __value = &_serde::export::from_utf8_lossy(__value);
                            _serde::export::Err(_serde::de::Error::unknown_field(__value, FIELDS))
                        }
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct __Visitor < 'de , T : Trait > where u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , Perbill : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , Perbill : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , Vec < T :: AccountId > : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , EraIndex : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , RingBalanceOf < T > : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , Vec < ( T :: AccountId , T :: AccountId , RingBalanceOf < T > , StakerStatus < T :: AccountId > ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned { marker : _serde :: export :: PhantomData < GenesisConfig < T > > , lifetime : _serde :: export :: PhantomData < & 'de ( ) > , }
            impl < 'de , T : Trait > _serde :: de :: Visitor < 'de > for __Visitor < 'de , T > where u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , Perbill : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , Perbill : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , u32 : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , Vec < T :: AccountId > : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , EraIndex : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , RingBalanceOf < T > : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , Vec < ( T :: AccountId , T :: AccountId , RingBalanceOf < T > , StakerStatus < T :: AccountId > ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned { type Value = GenesisConfig < T > ; fn expecting ( & self , __formatter : & mut _serde :: export :: Formatter ) -> _serde :: export :: fmt :: Result { _serde :: export :: Formatter :: write_str ( __formatter , "struct GenesisConfig" ) } # [ inline ] fn visit_seq < __A > ( self , mut __seq : __A ) -> _serde :: export :: Result < Self :: Value , __A :: Error > where __A : _serde :: de :: SeqAccess < 'de > { let __field0 = match match _serde :: de :: SeqAccess :: next_element :: < u32 > ( & mut __seq ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { _serde :: export :: Some ( __value ) => __value , _serde :: export :: None => { return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 0usize , & "struct GenesisConfig with 9 elements" ) ) ; } } ; let __field1 = match match _serde :: de :: SeqAccess :: next_element :: < u32 > ( & mut __seq ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { _serde :: export :: Some ( __value ) => __value , _serde :: export :: None => { return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 1usize , & "struct GenesisConfig with 9 elements" ) ) ; } } ; let __field2 = match match _serde :: de :: SeqAccess :: next_element :: < Perbill > ( & mut __seq ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { _serde :: export :: Some ( __value ) => __value , _serde :: export :: None => { return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 2usize , & "struct GenesisConfig with 9 elements" ) ) ; } } ; let __field3 = match match _serde :: de :: SeqAccess :: next_element :: < Perbill > ( & mut __seq ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { _serde :: export :: Some ( __value ) => __value , _serde :: export :: None => { return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 3usize , & "struct GenesisConfig with 9 elements" ) ) ; } } ; let __field4 = match match _serde :: de :: SeqAccess :: next_element :: < u32 > ( & mut __seq ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { _serde :: export :: Some ( __value ) => __value , _serde :: export :: None => { return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 4usize , & "struct GenesisConfig with 9 elements" ) ) ; } } ; let __field5 = match match _serde :: de :: SeqAccess :: next_element :: < Vec < T :: AccountId > > ( & mut __seq ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { _serde :: export :: Some ( __value ) => __value , _serde :: export :: None => { return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 5usize , & "struct GenesisConfig with 9 elements" ) ) ; } } ; let __field6 = match match _serde :: de :: SeqAccess :: next_element :: < EraIndex > ( & mut __seq ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { _serde :: export :: Some ( __value ) => __value , _serde :: export :: None => { return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 6usize , & "struct GenesisConfig with 9 elements" ) ) ; } } ; let __field7 = match match _serde :: de :: SeqAccess :: next_element :: < RingBalanceOf < T > > ( & mut __seq ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { _serde :: export :: Some ( __value ) => __value , _serde :: export :: None => { return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 7usize , & "struct GenesisConfig with 9 elements" ) ) ; } } ; let __field8 = match match _serde :: de :: SeqAccess :: next_element :: < Vec < ( T :: AccountId , T :: AccountId , RingBalanceOf < T > , StakerStatus < T :: AccountId > ) > > ( & mut __seq ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { _serde :: export :: Some ( __value ) => __value , _serde :: export :: None => { return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 8usize , & "struct GenesisConfig with 9 elements" ) ) ; } } ; _serde :: export :: Ok ( GenesisConfig { validator_count : __field0 , minimum_validator_count : __field1 , session_reward : __field2 , offline_slash : __field3 , offline_slash_grace : __field4 , invulnerables : __field5 , current_era : __field6 , current_era_total_reward : __field7 , stakers : __field8 , } ) } # [ inline ] fn visit_map < __A > ( self , mut __map : __A ) -> _serde :: export :: Result < Self :: Value , __A :: Error > where __A : _serde :: de :: MapAccess < 'de > { let mut __field0 : _serde :: export :: Option < u32 > = _serde :: export :: None ; let mut __field1 : _serde :: export :: Option < u32 > = _serde :: export :: None ; let mut __field2 : _serde :: export :: Option < Perbill > = _serde :: export :: None ; let mut __field3 : _serde :: export :: Option < Perbill > = _serde :: export :: None ; let mut __field4 : _serde :: export :: Option < u32 > = _serde :: export :: None ; let mut __field5 : _serde :: export :: Option < Vec < T :: AccountId > > = _serde :: export :: None ; let mut __field6 : _serde :: export :: Option < EraIndex > = _serde :: export :: None ; let mut __field7 : _serde :: export :: Option < RingBalanceOf < T > > = _serde :: export :: None ; let mut __field8 : _serde :: export :: Option < Vec < ( T :: AccountId , T :: AccountId , RingBalanceOf < T > , StakerStatus < T :: AccountId > ) > > = _serde :: export :: None ; while let _serde :: export :: Some ( __key ) = match _serde :: de :: MapAccess :: next_key :: < __Field > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { match __key { __Field :: __field0 => { if _serde :: export :: Option :: is_some ( & __field0 ) { return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "validatorCount" ) ) ; } __field0 = _serde :: export :: Some ( match _serde :: de :: MapAccess :: next_value :: < u32 > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } ) ; } __Field :: __field1 => { if _serde :: export :: Option :: is_some ( & __field1 ) { return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "minimumValidatorCount" ) ) ; } __field1 = _serde :: export :: Some ( match _serde :: de :: MapAccess :: next_value :: < u32 > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } ) ; } __Field :: __field2 => { if _serde :: export :: Option :: is_some ( & __field2 ) { return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "sessionReward" ) ) ; } __field2 = _serde :: export :: Some ( match _serde :: de :: MapAccess :: next_value :: < Perbill > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } ) ; } __Field :: __field3 => { if _serde :: export :: Option :: is_some ( & __field3 ) { return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "offlineSlash" ) ) ; } __field3 = _serde :: export :: Some ( match _serde :: de :: MapAccess :: next_value :: < Perbill > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } ) ; } __Field :: __field4 => { if _serde :: export :: Option :: is_some ( & __field4 ) { return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "offlineSlashGrace" ) ) ; } __field4 = _serde :: export :: Some ( match _serde :: de :: MapAccess :: next_value :: < u32 > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } ) ; } __Field :: __field5 => { if _serde :: export :: Option :: is_some ( & __field5 ) { return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "invulnerables" ) ) ; } __field5 = _serde :: export :: Some ( match _serde :: de :: MapAccess :: next_value :: < Vec < T :: AccountId > > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } ) ; } __Field :: __field6 => { if _serde :: export :: Option :: is_some ( & __field6 ) { return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "currentEra" ) ) ; } __field6 = _serde :: export :: Some ( match _serde :: de :: MapAccess :: next_value :: < EraIndex > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } ) ; } __Field :: __field7 => { if _serde :: export :: Option :: is_some ( & __field7 ) { return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "currentEraTotalReward" ) ) ; } __field7 = _serde :: export :: Some ( match _serde :: de :: MapAccess :: next_value :: < RingBalanceOf < T > > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } ) ; } __Field :: __field8 => { if _serde :: export :: Option :: is_some ( & __field8 ) { return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "stakers" ) ) ; } __field8 = _serde :: export :: Some ( match _serde :: de :: MapAccess :: next_value :: < Vec < ( T :: AccountId , T :: AccountId , RingBalanceOf < T > , StakerStatus < T :: AccountId > ) > > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } ) ; } } } let __field0 = match __field0 { _serde :: export :: Some ( __field0 ) => __field0 , _serde :: export :: None => match _serde :: private :: de :: missing_field ( "validatorCount" ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } , } ; let __field1 = match __field1 { _serde :: export :: Some ( __field1 ) => __field1 , _serde :: export :: None => match _serde :: private :: de :: missing_field ( "minimumValidatorCount" ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } , } ; let __field2 = match __field2 { _serde :: export :: Some ( __field2 ) => __field2 , _serde :: export :: None => match _serde :: private :: de :: missing_field ( "sessionReward" ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } , } ; let __field3 = match __field3 { _serde :: export :: Some ( __field3 ) => __field3 , _serde :: export :: None => match _serde :: private :: de :: missing_field ( "offlineSlash" ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } , } ; let __field4 = match __field4 { _serde :: export :: Some ( __field4 ) => __field4 , _serde :: export :: None => match _serde :: private :: de :: missing_field ( "offlineSlashGrace" ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } , } ; let __field5 = match __field5 { _serde :: export :: Some ( __field5 ) => __field5 , _serde :: export :: None => match _serde :: private :: de :: missing_field ( "invulnerables" ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } , } ; let __field6 = match __field6 { _serde :: export :: Some ( __field6 ) => __field6 , _serde :: export :: None => match _serde :: private :: de :: missing_field ( "currentEra" ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } , } ; let __field7 = match __field7 { _serde :: export :: Some ( __field7 ) => __field7 , _serde :: export :: None => match _serde :: private :: de :: missing_field ( "currentEraTotalReward" ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } , } ; let __field8 = match __field8 { _serde :: export :: Some ( __field8 ) => __field8 , _serde :: export :: None => match _serde :: private :: de :: missing_field ( "stakers" ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } , } ; _serde :: export :: Ok ( GenesisConfig { validator_count : __field0 , minimum_validator_count : __field1 , session_reward : __field2 , offline_slash : __field3 , offline_slash_grace : __field4 , invulnerables : __field5 , current_era : __field6 , current_era_total_reward : __field7 , stakers : __field8 , } ) } }
            const FIELDS: &'static [&'static str] = &[
                "validatorCount",
                "minimumValidatorCount",
                "sessionReward",
                "offlineSlash",
                "offlineSlashGrace",
                "invulnerables",
                "currentEra",
                "currentEraTotalReward",
                "stakers",
            ];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "GenesisConfig",
                FIELDS,
                __Visitor {
                    marker: _serde::export::PhantomData::<GenesisConfig<T>>,
                    lifetime: _serde::export::PhantomData,
                },
            )
        }
    }
};
#[cfg(feature = "std")]
impl<T: Trait> Default for GenesisConfig<T> {
    fn default() -> Self {
        GenesisConfig {
            validator_count: Default::default(),
            minimum_validator_count: DEFAULT_MINIMUM_VALIDATOR_COUNT,
            session_reward: Perbill::from_percent(60),
            offline_slash: Perbill::from_parts(1000),
            offline_slash_grace: Default::default(),
            invulnerables: Default::default(),
            current_era: Default::default(),
            current_era_total_reward: Default::default(),
            stakers: Default::default(),
        }
    }
}
#[cfg(feature = "std")]
impl<T: Trait> GenesisConfig<T>
where
    u32: Clone,
    u32: Clone,
    Perbill: Clone,
    Perbill: Clone,
    u32: Clone,
    Vec<T::AccountId>: Clone,
    EraIndex: Clone,
    RingBalanceOf<T>: Clone,
{
    pub fn build_storage ( self ) -> std :: result :: Result < ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: runtime_primitives :: StorageOverlay , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: runtime_primitives :: ChildrenStorageOverlay ) , String >{
        let mut storage = Default::default();
        let mut child_storage = Default::default();
        self.assimilate_storage(&mut storage, &mut child_storage)?;
        Ok((storage, child_storage))
    }
    /// Assimilate the storage for this module into pre-existing overlays.
    pub fn assimilate_storage(
        self,
        r : & mut self :: sr_api_hidden_includes_decl_storage :: hidden_include :: runtime_primitives :: StorageOverlay,
        c : & mut self :: sr_api_hidden_includes_decl_storage :: hidden_include :: runtime_primitives :: ChildrenStorageOverlay,
    ) -> std::result::Result<(), String> {
        let storage = r;
        {
            let v = (|config: &Self| config.validator_count.clone())(&self);
            < ValidatorCount < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: put ( & v , storage ) ;
        }
        {
            let v = (|config: &Self| config.minimum_validator_count.clone())(&self);
            < MinimumValidatorCount < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: put ( & v , storage ) ;
        }
        {
            let v = (|config: &Self| config.session_reward.clone())(&self);
            < SessionReward < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: put ( & v , storage ) ;
        }
        {
            let v = (|config: &Self| config.offline_slash.clone())(&self);
            < OfflineSlash < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Perbill > > :: put ( & v , storage ) ;
        }
        {
            let v = (|config: &Self| config.offline_slash_grace.clone())(&self);
            < OfflineSlashGrace < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < u32 > > :: put ( & v , storage ) ;
        }
        {
            let v = (|config: &Self| config.invulnerables.clone())(&self);
            < Invulnerables < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < Vec < T :: AccountId > > > :: put ( & v , storage ) ;
        }
        {
            let v = (|config: &Self| config.current_era.clone())(&self);
            < CurrentEra < > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < EraIndex > > :: put ( & v , storage ) ;
        }
        {
            let v = (|config: &Self| config.current_era_total_reward.clone())(&self);
            < CurrentEraTotalReward < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < RingBalanceOf < T > > > :: put ( & v , storage ) ;
        }
        (|storage: &mut primitives::StorageOverlay,
          _: &mut primitives::ChildrenStorageOverlay,
          config: &GenesisConfig<T>| {
            with_storage(storage, || {
                for &(ref stash, ref controller, balance, ref status) in &config.stakers {
                    if !(T::Ring::free_balance(&stash) >= balance) {
                        {
                            ::std::rt::begin_panic(
                                "assertion failed: T::Ring::free_balance(&stash) >= balance",
                                &("srml/staking/src/lib.rs", 409u32, 25u32),
                            )
                        }
                    };
                    let _ = <Module<T>>::bond(
                        T::Origin::from(Some(stash.to_owned()).into()),
                        T::Lookup::unlookup(controller.to_owned()),
                        StakingBalance::Ring(balance),
                        RewardDestination::Stash,
                        12,
                    );
                    let _ = match status {
                        StakerStatus::Validator => <Module<T>>::validate(
                            T::Origin::from(Some(controller.to_owned()).into()),
                            ::alloc::vec::from_elem(0, 8),
                            0,
                            3,
                        ),
                        StakerStatus::Nominator(votes) => <Module<T>>::nominate(
                            T::Origin::from(Some(controller.to_owned()).into()),
                            votes
                                .iter()
                                .map(|l| T::Lookup::unlookup(l.to_owned()))
                                .collect(),
                        ),
                        _ => Ok(()),
                    };
                }
                if let (_, Some(validators)) = <Module<T>>::select_validators() {
                    <session::Validators<T>>::put(&validators);
                }
            });
        })(storage, c, &self);
        Ok(())
    }
}
#[cfg(feature = "std")]
impl < T : Trait , __GeneratedInstance : __GeneratedInstantiable > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: runtime_primitives :: BuildModuleGenesisStorage < T , __GeneratedInstance > for GenesisConfig < T > where u32 : Clone , u32 : Clone , Perbill : Clone , Perbill : Clone , u32 : Clone , Vec < T :: AccountId > : Clone , EraIndex : Clone , RingBalanceOf < T > : Clone { fn build_module_genesis_storage ( self , r : & mut self :: sr_api_hidden_includes_decl_storage :: hidden_include :: runtime_primitives :: StorageOverlay , c : & mut self :: sr_api_hidden_includes_decl_storage :: hidden_include :: runtime_primitives :: ChildrenStorageOverlay ) -> std :: result :: Result < ( ) , String > { self . assimilate_storage :: < > ( r , c ) } }
/// [`RawEvent`] specialized for the configuration [`Trait`]
///
/// [`RawEvent`]: enum.RawEvent.html
/// [`Trait`]: trait.Trait.html
pub type Event<T> = RawEvent<RingBalanceOf<T>, <T as system::Trait>::AccountId>;
/// Events for this module.
///
#[structural_match]
pub enum RawEvent<Balance, AccountId> {
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
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::clone::Clone, AccountId: ::std::clone::Clone> ::std::clone::Clone
    for RawEvent<Balance, AccountId>
{
    #[inline]
    fn clone(&self) -> RawEvent<Balance, AccountId> {
        match (&*self,) {
            (&RawEvent::Reward(ref __self_0),) => {
                RawEvent::Reward(::std::clone::Clone::clone(&(*__self_0)))
            }
            (&RawEvent::OfflineWarning(ref __self_0, ref __self_1),) => RawEvent::OfflineWarning(
                ::std::clone::Clone::clone(&(*__self_0)),
                ::std::clone::Clone::clone(&(*__self_1)),
            ),
            (&RawEvent::OfflineSlash(ref __self_0, ref __self_1),) => RawEvent::OfflineSlash(
                ::std::clone::Clone::clone(&(*__self_0)),
                ::std::clone::Clone::clone(&(*__self_1)),
            ),
            (&RawEvent::NodeNameUpdated,) => RawEvent::NodeNameUpdated,
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::cmp::PartialEq, AccountId: ::std::cmp::PartialEq> ::std::cmp::PartialEq
    for RawEvent<Balance, AccountId>
{
    #[inline]
    fn eq(&self, other: &RawEvent<Balance, AccountId>) -> bool {
        {
            let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as isize;
            let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as isize;
            if true && __self_vi == __arg_1_vi {
                match (&*self, &*other) {
                    (&RawEvent::Reward(ref __self_0), &RawEvent::Reward(ref __arg_1_0)) => {
                        (*__self_0) == (*__arg_1_0)
                    }
                    (
                        &RawEvent::OfflineWarning(ref __self_0, ref __self_1),
                        &RawEvent::OfflineWarning(ref __arg_1_0, ref __arg_1_1),
                    ) => (*__self_0) == (*__arg_1_0) && (*__self_1) == (*__arg_1_1),
                    (
                        &RawEvent::OfflineSlash(ref __self_0, ref __self_1),
                        &RawEvent::OfflineSlash(ref __arg_1_0, ref __arg_1_1),
                    ) => (*__self_0) == (*__arg_1_0) && (*__self_1) == (*__arg_1_1),
                    _ => true,
                }
            } else {
                false
            }
        }
    }
    #[inline]
    fn ne(&self, other: &RawEvent<Balance, AccountId>) -> bool {
        {
            let __self_vi = unsafe { ::std::intrinsics::discriminant_value(&*self) } as isize;
            let __arg_1_vi = unsafe { ::std::intrinsics::discriminant_value(&*other) } as isize;
            if true && __self_vi == __arg_1_vi {
                match (&*self, &*other) {
                    (&RawEvent::Reward(ref __self_0), &RawEvent::Reward(ref __arg_1_0)) => {
                        (*__self_0) != (*__arg_1_0)
                    }
                    (
                        &RawEvent::OfflineWarning(ref __self_0, ref __self_1),
                        &RawEvent::OfflineWarning(ref __arg_1_0, ref __arg_1_1),
                    ) => (*__self_0) != (*__arg_1_0) || (*__self_1) != (*__arg_1_1),
                    (
                        &RawEvent::OfflineSlash(ref __self_0, ref __self_1),
                        &RawEvent::OfflineSlash(ref __arg_1_0, ref __arg_1_1),
                    ) => (*__self_0) != (*__arg_1_0) || (*__self_1) != (*__arg_1_1),
                    _ => false,
                }
            } else {
                true
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::cmp::Eq, AccountId: ::std::cmp::Eq> ::std::cmp::Eq
    for RawEvent<Balance, AccountId>
{
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<Balance>;
            let _: ::std::cmp::AssertParamIsEq<AccountId>;
            let _: ::std::cmp::AssertParamIsEq<u32>;
            let _: ::std::cmp::AssertParamIsEq<AccountId>;
            let _: ::std::cmp::AssertParamIsEq<u32>;
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_RawEvent: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<Balance, AccountId> _parity_codec::Encode for RawEvent<Balance, AccountId>
    where
        Balance: _parity_codec::Encode,
        Balance: _parity_codec::Encode,
        AccountId: _parity_codec::Encode,
        AccountId: _parity_codec::Encode,
        AccountId: _parity_codec::Encode,
        AccountId: _parity_codec::Encode,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            match *self {
                RawEvent::Reward(ref aa) => {
                    dest.push_byte(0usize as u8);
                    dest.push(aa);
                }
                RawEvent::OfflineWarning(ref aa, ref ba) => {
                    dest.push_byte(1usize as u8);
                    dest.push(aa);
                    dest.push(ba);
                }
                RawEvent::OfflineSlash(ref aa, ref ba) => {
                    dest.push_byte(2usize as u8);
                    dest.push(aa);
                    dest.push(ba);
                }
                RawEvent::NodeNameUpdated => {
                    dest.push_byte(3usize as u8);
                }
                _ => (),
            }
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_RawEvent: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<Balance, AccountId> _parity_codec::Decode for RawEvent<Balance, AccountId>
    where
        Balance: _parity_codec::Decode,
        Balance: _parity_codec::Decode,
        AccountId: _parity_codec::Decode,
        AccountId: _parity_codec::Decode,
        AccountId: _parity_codec::Decode,
        AccountId: _parity_codec::Decode,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            match input.read_byte()? {
                x if x == 0usize as u8 => {
                    Some(RawEvent::Reward(_parity_codec::Decode::decode(input)?))
                }
                x if x == 1usize as u8 => Some(RawEvent::OfflineWarning(
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                )),
                x if x == 2usize as u8 => Some(RawEvent::OfflineSlash(
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                )),
                x if x == 3usize as u8 => Some(RawEvent::NodeNameUpdated),
                _ => None,
            }
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::fmt::Debug, AccountId: ::std::fmt::Debug> ::std::fmt::Debug
    for RawEvent<Balance, AccountId>
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match (&*self,) {
            (&RawEvent::Reward(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("Reward");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            }
            (&RawEvent::OfflineWarning(ref __self_0, ref __self_1),) => {
                let mut debug_trait_builder = f.debug_tuple("OfflineWarning");
                let _ = debug_trait_builder.field(&&(*__self_0));
                let _ = debug_trait_builder.field(&&(*__self_1));
                debug_trait_builder.finish()
            }
            (&RawEvent::OfflineSlash(ref __self_0, ref __self_1),) => {
                let mut debug_trait_builder = f.debug_tuple("OfflineSlash");
                let _ = debug_trait_builder.field(&&(*__self_0));
                let _ = debug_trait_builder.field(&&(*__self_1));
                debug_trait_builder.finish()
            }
            (&RawEvent::NodeNameUpdated,) => {
                let mut debug_trait_builder = f.debug_tuple("NodeNameUpdated");
                debug_trait_builder.finish()
            }
        }
    }
}
impl<Balance, AccountId> From<RawEvent<Balance, AccountId>> for () {
    fn from(_: RawEvent<Balance, AccountId>) -> () {
        ()
    }
}
impl<Balance, AccountId> RawEvent<Balance, AccountId> {
    #[allow(dead_code)]
    pub fn metadata() -> &'static [::srml_support::event::EventMetadata] {
        & [ :: srml_support :: event :: EventMetadata { name : :: srml_support :: event :: DecodeDifferent :: Encode ( "Reward" ) , arguments : :: srml_support :: event :: DecodeDifferent :: Encode ( & [ "Balance" ] ) , documentation : :: srml_support :: event :: DecodeDifferent :: Encode ( & [ r" All validators have been rewarded by the given balance." ] ) , } , :: srml_support :: event :: EventMetadata { name : :: srml_support :: event :: DecodeDifferent :: Encode ( "OfflineWarning" ) , arguments : :: srml_support :: event :: DecodeDifferent :: Encode ( & [ "AccountId" , "u32" ] ) , documentation : :: srml_support :: event :: DecodeDifferent :: Encode ( & [ r" One validator (and its nominators) has been given an offline-warning (it is still" , r" within its grace). The accrued number of slashes is recorded, too." ] ) , } , :: srml_support :: event :: EventMetadata { name : :: srml_support :: event :: DecodeDifferent :: Encode ( "OfflineSlash" ) , arguments : :: srml_support :: event :: DecodeDifferent :: Encode ( & [ "AccountId" , "u32" ] ) , documentation : :: srml_support :: event :: DecodeDifferent :: Encode ( & [ r" One validator (and its nominators) has been slashed by the given ratio." ] ) , } , :: srml_support :: event :: EventMetadata { name : :: srml_support :: event :: DecodeDifferent :: Encode ( "NodeNameUpdated" ) , arguments : :: srml_support :: event :: DecodeDifferent :: Encode ( & [ ] ) , documentation : :: srml_support :: event :: DecodeDifferent :: Encode ( & [ r" NodeName changed" ] ) , } ]
    }
}
#[structural_match]
#[rustc_copy_clone_marker]
pub struct Module<T: Trait>(::srml_support::rstd::marker::PhantomData<(T)>);
#[automatically_derived]
#[allow(unused_qualifications)]
impl<T: ::std::clone::Clone + Trait> ::std::clone::Clone for Module<T> {
    #[inline]
    fn clone(&self) -> Module<T> {
        match *self {
            Module(ref __self_0_0) => Module(::std::clone::Clone::clone(&(*__self_0_0))),
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<T: ::std::marker::Copy + Trait> ::std::marker::Copy for Module<T> {}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<T: ::std::cmp::PartialEq + Trait> ::std::cmp::PartialEq for Module<T> {
    #[inline]
    fn eq(&self, other: &Module<T>) -> bool {
        match *other {
            Module(ref __self_1_0) => match *self {
                Module(ref __self_0_0) => (*__self_0_0) == (*__self_1_0),
            },
        }
    }
    #[inline]
    fn ne(&self, other: &Module<T>) -> bool {
        match *other {
            Module(ref __self_1_0) => match *self {
                Module(ref __self_0_0) => (*__self_0_0) != (*__self_1_0),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<T: ::std::cmp::Eq + Trait> ::std::cmp::Eq for Module<T> {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<::srml_support::rstd::marker::PhantomData<(T)>>;
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<T: ::std::fmt::Debug + Trait> ::std::fmt::Debug for Module<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Module(ref __self_0_0) => {
                let mut debug_trait_builder = f.debug_tuple("Module");
                let _ = debug_trait_builder.field(&&(*__self_0_0));
                debug_trait_builder.finish()
            }
        }
    }
}
impl<T: Trait> ::srml_support::runtime_primitives::traits::OnInitialize<T::BlockNumber>
    for Module<T>
{
}
impl<T: Trait> ::srml_support::runtime_primitives::traits::OnFinalize<T::BlockNumber>
    for Module<T>
{
}
impl<T: Trait> ::srml_support::runtime_primitives::traits::OffchainWorker<T::BlockNumber>
    for Module<T>
{
}
impl<T: Trait> Module<T> {
    fn deposit_event(event: Event<T>) {
        <system::Module<T>>::deposit_event(<T as Trait>::from(event).into());
    }
}
/// Can also be called using [`Call`].
///
/// [`Call`]: enum.Call.html
impl<T: Trait> Module<T> {
    fn bond(
        origin: T::Origin,
        controller: <T::Lookup as StaticLookup>::Source,
        value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
        payee: RewardDestination,
        promise_month: u32,
    ) -> ::srml_support::dispatch::Result {
        {
            let stash = ensure_signed(origin)?;
            {
                if !(promise_month <= 36) {
                    {
                        return Err("months at most is 36.");
                    };
                }
            };
            if <Bonded<T>>::exists(&stash) {
                return Err("stash already bonded");
            }
            let controller = T::Lookup::lookup(controller)?;
            if <Ledger<T>>::exists(&controller) {
                return Err("controller already paired");
            }
            <Bonded<T>>::insert(&stash, &controller);
            <Payee<T>>::insert(&stash, &payee);
            let ledger = StakingLedger {
                stash: stash.clone(),
                ..Default::default()
            };
            match value {
                StakingBalance::Ring(ring) => {
                    let stash_balance = T::Ring::free_balance(&stash);
                    let value = ring.min(stash_balance);
                    <RingPool<T>>::mutate(|ring| *ring += value);
                    Self::bond_helper_in_ring(&stash, &controller, value, promise_month, ledger);
                }
                StakingBalance::Kton(kton) => {
                    let stash_balance = T::Kton::free_balance(&stash);
                    let value = kton.min(stash_balance);
                    <KtonPool<T>>::mutate(|kton| *kton += value);
                    Self::bond_helper_in_kton(&controller, value, ledger);
                }
            }
        }
        Ok(())
    }
    fn bond_extra(
        origin: T::Origin,
        value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
        promise_month: u32,
    ) -> ::srml_support::dispatch::Result {
        {
            let stash = ensure_signed(origin)?;
            {
                if !(promise_month <= 36) {
                    {
                        return Err("months at most is 36.");
                    };
                }
            };
            let controller = Self::bonded(&stash).ok_or("not a stash")?;
            let ledger = Self::ledger(&controller).ok_or("not a controller")?;
            match value {
                StakingBalance::Ring(ring) => {
                    let stash_balance = T::Ring::free_balance(&stash);
                    if let Some(extra) = stash_balance.checked_sub(&(ledger.total_ring)) {
                        let extra = extra.min(ring);
                        <RingPool<T>>::mutate(|ring| *ring += extra);
                        Self::bond_helper_in_ring(
                            &stash,
                            &controller,
                            extra,
                            promise_month,
                            ledger,
                        );
                    }
                }
                StakingBalance::Kton(kton) => {
                    let stash_balance = T::Kton::free_balance(&stash);
                    if let Some(extra) = stash_balance.checked_sub(&(ledger.total_kton)) {
                        let extra = extra.min(kton);
                        <KtonPool<T>>::mutate(|kton| *kton += extra);
                        Self::bond_helper_in_kton(&controller, extra, ledger);
                    }
                }
            }
        }
        Ok(())
    }
    /// for normal_ring or normal_kton, follow the original substrate pattern
    /// for time_deposit_ring, transform it into normal_ring first
    /// modify time_deposit_items and time_deposit_ring amount
    fn unbond(
        origin: T::Origin,
        value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
    ) -> ::srml_support::dispatch::Result {
        {
            let controller = ensure_signed(origin)?;
            let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let StakingLedger {
                active_ring,
                active_deposit_ring,
                active_kton,
                deposit_items,
                unlocking,
                ..
            } = &mut ledger;
            {
                if !(unlocking.len() < MAX_UNLOCKING_CHUNKS) {
                    {
                        return Err("can not schedule more unlock chunks");
                    };
                }
            };
            let era = Self::current_era() + T::BondingDuration::get();
            match value {
                StakingBalance::Ring(ring) => {
                    let total_value = ring.min(*active_ring);
                    let active_normal_ring = *active_ring - *active_deposit_ring;
                    let active_normal_value = total_value.min(active_normal_ring);
                    let mut unlock_value_left = total_value - active_normal_value;
                    <RingPool<T>>::mutate(|ring| *ring -= active_normal_value);
                    if !active_normal_value.is_zero() {
                        *active_ring -= active_normal_value;
                        unlocking.push(UnlockChunk {
                            value: StakingBalance::Ring(total_value),
                            era,
                            is_time_deposit: false,
                        });
                    }
                    let is_time_deposit =
                        active_normal_value.is_zero() || !unlock_value_left.is_zero();
                    let mut total_deposit_changed = 0.into();
                    if is_time_deposit {
                        let now = <timestamp::Module<T>>::now();
                        # [ doc = " for time_deposit_ring, transform into normal one" ] deposit_items . drain_filter ( | item | { if item . expire_time > now { return false ; } if unlock_value_left . is_zero ( ) { return true ; } let value = unlock_value_left . min ( item . value ) ; unlock_value_left = unlock_value_left . saturating_sub ( value ) ; * active_deposit_ring = active_deposit_ring . saturating_sub ( value ) ; * active_ring = active_ring . saturating_sub ( value ) ; total_deposit_changed += value ; item . value -= value ; item . value . is_zero ( ) } ) ;
                        unlocking.push(UnlockChunk {
                            value: StakingBalance::Ring(total_deposit_changed),
                            era,
                            is_time_deposit: true,
                        });
                        <RingPool<T>>::mutate(|ring| *ring -= total_deposit_changed);
                    }
                }
                StakingBalance::Kton(kton) => {
                    let value = kton.min(*active_kton);
                    <KtonPool<T>>::mutate(|kton| *kton -= value);
                    *active_kton -= value;
                    unlocking.push(UnlockChunk {
                        value: StakingBalance::Kton(value),
                        era,
                        is_time_deposit: false,
                    });
                }
            }
            <Ledger<T>>::insert(&controller, &ledger);
        }
        Ok(())
    }
    fn unbond_with_punish(
        origin: T::Origin,
        value: RingBalanceOf<T>,
        expire_time: T::Moment,
    ) -> ::srml_support::dispatch::Result {
        {
            let controller = ensure_signed(origin)?;
            let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let StakingLedger {
                stash,
                active_ring,
                active_deposit_ring,
                deposit_items,
                unlocking,
                ..
            } = &mut ledger;
            let now = <timestamp::Module<T>>::now();
            {
                if !(expire_time > now) {
                    {
                        return Err("use unbond instead.");
                    };
                }
            };
            deposit_items.drain_filter(|item| {
                if item.expire_time != expire_time {
                    return false;
                }
                let value = item.value.min(value);
                let month_left = ((expire_time.clone() - now.clone()).saturated_into::<u32>()
                    / MONTH_IN_SECONDS)
                    .max(1);
                let kton_slash = utils::compute_kton_return::<T>(value, month_left) * 3.into();
                let is_slashable = T::Kton::free_balance(stash)
                    .checked_sub(&kton_slash)
                    .and_then(|new_balance| {
                        T::Kton::ensure_can_withdraw(
                            stash,
                            kton_slash,
                            WithdrawReason::Transfer,
                            new_balance,
                        )
                        .ok()
                    })
                    .is_some();
                if !is_slashable {
                    return false;
                }
                item.value -= value;
                *active_ring = active_ring.saturating_sub(value);
                *active_deposit_ring = active_deposit_ring.saturating_sub(value);
                let (imbalance, _) = T::Kton::slash(stash, kton_slash);
                T::KtonSlash::on_unbalanced(imbalance);
                unlocking.push(UnlockChunk {
                    value: StakingBalance::Ring(value),
                    era: Self::current_era() + T::BondingDuration::get(),
                    is_time_deposit: true,
                });
                item.value.is_zero()
            });
            <Ledger<T>>::insert(&controller, &ledger);
        }
        Ok(())
    }
    /// called by controller
    fn promise_extra(
        origin: T::Origin,
        value: RingBalanceOf<T>,
        promise_month: u32,
    ) -> ::srml_support::dispatch::Result {
        {
            let controller = ensure_signed(origin)?;
            {
                if !(promise_month <= 36) {
                    {
                        return Err("months at most is 36.");
                    };
                }
            };
            let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let StakingLedger {
                active_ring,
                total_deposit_ring,
                active_deposit_ring,
                deposit_items,
                stash,
                ..
            } = &mut ledger;
            let now = <timestamp::Module<T>>::now();
            deposit_items.retain(|item| {
                if item.expire_time < now {
                    *active_deposit_ring = active_deposit_ring.saturating_sub(item.value);
                    *total_deposit_ring = total_deposit_ring.saturating_sub(item.value);
                    false
                } else {
                    true
                }
            });
            let value = value.min(*active_ring - *active_deposit_ring);
            if promise_month >= 3 {
                *total_deposit_ring += value;
                *active_deposit_ring += value;
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
            <Ledger<T>>::insert(&controller, &ledger);
        }
        Ok(())
    }
    /// may both withdraw ring and kton at the same time
    fn withdraw_unbonded(origin: T::Origin) -> ::srml_support::dispatch::Result {
        {
            let controller = ensure_signed(origin)?;
            let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let StakingLedger {
                total_ring,
                total_deposit_ring,
                total_kton,
                unlocking,
                ..
            } = &mut ledger;
            let mut balance_kind = 0u8;
            let current_era = Self::current_era();
            unlocking.retain(
                |UnlockChunk {
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
                },
            );
            match balance_kind {
                0 => (),
                1 => Self::update_ledger(&controller, &ledger, StakingBalance::Ring(0.into())),
                2 => Self::update_ledger(&controller, &ledger, StakingBalance::Kton(0.into())),
                3 => {
                    Self::update_ledger(&controller, &ledger, StakingBalance::Ring(0.into()));
                    Self::update_ledger(&controller, &ledger, StakingBalance::Kton(0.into()));
                }
                _ => ::std::rt::begin_panic(
                    "internal error: entered unreachable code",
                    &("srml/staking/src/lib.rs", 856u32, 22u32),
                ),
            }
        }
        Ok(())
    }
    fn validate(
        origin: T::Origin,
        name: Vec<u8>,
        ratio: u32,
        unstake_threshold: u32,
    ) -> ::srml_support::dispatch::Result {
        {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let stash = &ledger.stash;
            {
                if !(unstake_threshold <= MAX_UNSTAKE_THRESHOLD) {
                    {
                        return Err("unstake threshold too large");
                    };
                }
            };
            let validator_payment_ratio = Perbill::from_percent(ratio.min(100));
            let prefs = ValidatorPrefs {
                unstake_threshold,
                validator_payment_ratio,
            };
            <Nominators<T>>::remove(stash);
            <Validators<T>>::insert(stash, &prefs);
            if !<NodeName<T>>::exists(&controller) {
                <NodeName<T>>::insert(&controller, &name);
                Self::deposit_event(RawEvent::NodeNameUpdated);
            }
        }
        Ok(())
    }
    fn nominate(
        origin: T::Origin,
        targets: Vec<<T::Lookup as StaticLookup>::Source>,
    ) -> ::srml_support::dispatch::Result {
        {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or("not a controller")?;
            {
                if !!targets.is_empty() {
                    {
                        return Err("targets cannot be empty");
                    };
                }
            };
            let stash = &ledger.stash;
            let targets = targets
                .into_iter()
                .take(MAX_NOMINATIONS)
                .map(T::Lookup::lookup)
                .collect::<Result<Vec<_>, _>>()?;
            <Validators<T>>::remove(stash);
            <Nominators<T>>::insert(stash, &targets);
        }
        Ok(())
    }
    fn chill(origin: T::Origin) -> ::srml_support::dispatch::Result {
        {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let stash = &ledger.stash;
            <Validators<T>>::remove(stash);
            <Nominators<T>>::remove(stash);
        }
        Ok(())
    }
    fn set_payee(origin: T::Origin, payee: RewardDestination) -> ::srml_support::dispatch::Result {
        {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let stash = &ledger.stash;
            <Payee<T>>::insert(stash, &payee);
        }
        Ok(())
    }
    fn set_controller(
        origin: T::Origin,
        controller: <T::Lookup as StaticLookup>::Source,
    ) -> ::srml_support::dispatch::Result {
        {
            let stash = ensure_signed(origin)?;
            let old_controller = Self::bonded(&stash).ok_or("not a stash")?;
            let controller = T::Lookup::lookup(controller)?;
            if <Ledger<T>>::exists(&controller) {
                return Err("controller already paired");
            }
            if controller != old_controller {
                <Bonded<T>>::insert(&stash, &controller);
                if let Some(ledger) = <Ledger<T>>::take(&old_controller) {
                    <Ledger<T>>::insert(&controller, &ledger);
                }
            }
        }
        Ok(())
    }
    /// The ideal number of validators.
    #[allow(unreachable_code)]
    fn set_validator_count(new: u32) -> ::srml_support::dispatch::Result {
        {
            ValidatorCount::put(new);
        }
        Ok(())
    }
    #[allow(unreachable_code)]
    fn force_new_era() -> ::srml_support::dispatch::Result {
        {
            Self::apply_force_new_era();
        }
        Ok(())
    }
    /// Set the offline slash grace period.
    #[allow(unreachable_code)]
    fn set_offline_slash_grace(new: u32) -> ::srml_support::dispatch::Result {
        {
            OfflineSlashGrace::put(new);
        }
        Ok(())
    }
    /// Set the validators who cannot be slashed (if any).
    #[allow(unreachable_code)]
    fn set_invulnerables(validators: Vec<T::AccountId>) -> ::srml_support::dispatch::Result {
        {
            <Invulnerables<T>>::put(validators);
        }
        Ok(())
    }
}
pub enum Call<T: Trait> {
    #[doc(hidden)]
    #[codec(skip)]
    __PhantomItem(
        ::srml_support::rstd::marker::PhantomData<(T)>,
        ::srml_support::dispatch::Never,
    ),
    #[allow(non_camel_case_types)]
    bond(
        <T::Lookup as StaticLookup>::Source,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
        RewardDestination,
        u32,
    ),
    #[allow(non_camel_case_types)]
    bond_extra(StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>, u32),
    #[allow(non_camel_case_types)]
    /// for normal_ring or normal_kton, follow the original substrate pattern
    /// for time_deposit_ring, transform it into normal_ring first
    /// modify time_deposit_items and time_deposit_ring amount
    unbond(StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>),
    #[allow(non_camel_case_types)]
    unbond_with_punish(RingBalanceOf<T>, T::Moment),
    #[allow(non_camel_case_types)]
    /// called by controller
    promise_extra(RingBalanceOf<T>, u32),
    #[allow(non_camel_case_types)]
    /// may both withdraw ring and kton at the same time
    withdraw_unbonded(),
    #[allow(non_camel_case_types)]
    validate(Vec<u8>, u32, u32),
    #[allow(non_camel_case_types)]
    nominate(Vec<<T::Lookup as StaticLookup>::Source>),
    #[allow(non_camel_case_types)]
    chill(),
    #[allow(non_camel_case_types)]
    set_payee(RewardDestination),
    #[allow(non_camel_case_types)]
    set_controller(<T::Lookup as StaticLookup>::Source),
    #[allow(non_camel_case_types)]
    /// The ideal number of validators.
    set_validator_count(#[codec(compact)] u32),
    #[allow(non_camel_case_types)]
    force_new_era(),
    #[allow(non_camel_case_types)]
    /// Set the offline slash grace period.
    set_offline_slash_grace(#[codec(compact)] u32),
    #[allow(non_camel_case_types)]
    /// Set the validators who cannot be slashed (if any).
    set_invulnerables(Vec<T::AccountId>),
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_Call: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<T: Trait> _parity_codec::Encode for Call<T>
    where
        <T::Lookup as StaticLookup>::Source: _parity_codec::Encode,
        <T::Lookup as StaticLookup>::Source: _parity_codec::Encode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Encode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Encode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Encode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Encode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Encode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Encode,
        RingBalanceOf<T>: _parity_codec::Encode,
        RingBalanceOf<T>: _parity_codec::Encode,
        T::Moment: _parity_codec::Encode,
        T::Moment: _parity_codec::Encode,
        RingBalanceOf<T>: _parity_codec::Encode,
        RingBalanceOf<T>: _parity_codec::Encode,
        Vec<<T::Lookup as StaticLookup>::Source>: _parity_codec::Encode,
        Vec<<T::Lookup as StaticLookup>::Source>: _parity_codec::Encode,
        <T::Lookup as StaticLookup>::Source: _parity_codec::Encode,
        <T::Lookup as StaticLookup>::Source: _parity_codec::Encode,
        Vec<T::AccountId>: _parity_codec::Encode,
        Vec<T::AccountId>: _parity_codec::Encode,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            match *self {
                Call::bond(ref aa, ref ba, ref ca, ref da) => {
                    dest.push_byte(0usize as u8);
                    dest.push(aa);
                    dest.push(ba);
                    dest.push(ca);
                    dest.push(da);
                }
                Call::bond_extra(ref aa, ref ba) => {
                    dest.push_byte(1usize as u8);
                    dest.push(aa);
                    dest.push(ba);
                }
                Call::unbond(ref aa) => {
                    dest.push_byte(2usize as u8);
                    dest.push(aa);
                }
                Call::unbond_with_punish(ref aa, ref ba) => {
                    dest.push_byte(3usize as u8);
                    dest.push(aa);
                    dest.push(ba);
                }
                Call::promise_extra(ref aa, ref ba) => {
                    dest.push_byte(4usize as u8);
                    dest.push(aa);
                    dest.push(ba);
                }
                Call::withdraw_unbonded() => {
                    dest.push_byte(5usize as u8);
                }
                Call::validate(ref aa, ref ba, ref ca) => {
                    dest.push_byte(6usize as u8);
                    dest.push(aa);
                    dest.push(ba);
                    dest.push(ca);
                }
                Call::nominate(ref aa) => {
                    dest.push_byte(7usize as u8);
                    dest.push(aa);
                }
                Call::chill() => {
                    dest.push_byte(8usize as u8);
                }
                Call::set_payee(ref aa) => {
                    dest.push_byte(9usize as u8);
                    dest.push(aa);
                }
                Call::set_controller(ref aa) => {
                    dest.push_byte(10usize as u8);
                    dest.push(aa);
                }
                Call::set_validator_count(ref aa) => {
                    dest.push_byte(11usize as u8);
                    {
                        dest . push ( & < < u32 as _parity_codec :: HasCompact > :: Type as _parity_codec :: EncodeAsRef < '_ , u32 > > :: from ( aa ) ) ;
                    }
                }
                Call::force_new_era() => {
                    dest.push_byte(12usize as u8);
                }
                Call::set_offline_slash_grace(ref aa) => {
                    dest.push_byte(13usize as u8);
                    {
                        dest . push ( & < < u32 as _parity_codec :: HasCompact > :: Type as _parity_codec :: EncodeAsRef < '_ , u32 > > :: from ( aa ) ) ;
                    }
                }
                Call::set_invulnerables(ref aa) => {
                    dest.push_byte(14usize as u8);
                    dest.push(aa);
                }
                _ => (),
            }
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_Call: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<T: Trait> _parity_codec::Decode for Call<T>
    where
        <T::Lookup as StaticLookup>::Source: _parity_codec::Decode,
        <T::Lookup as StaticLookup>::Source: _parity_codec::Decode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Decode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Decode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Decode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Decode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Decode,
        StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>: _parity_codec::Decode,
        RingBalanceOf<T>: _parity_codec::Decode,
        RingBalanceOf<T>: _parity_codec::Decode,
        T::Moment: _parity_codec::Decode,
        T::Moment: _parity_codec::Decode,
        RingBalanceOf<T>: _parity_codec::Decode,
        RingBalanceOf<T>: _parity_codec::Decode,
        Vec<<T::Lookup as StaticLookup>::Source>: _parity_codec::Decode,
        Vec<<T::Lookup as StaticLookup>::Source>: _parity_codec::Decode,
        <T::Lookup as StaticLookup>::Source: _parity_codec::Decode,
        <T::Lookup as StaticLookup>::Source: _parity_codec::Decode,
        Vec<T::AccountId>: _parity_codec::Decode,
        Vec<T::AccountId>: _parity_codec::Decode,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            match input.read_byte()? {
                x if x == 0usize as u8 => Some(Call::bond(
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                )),
                x if x == 1usize as u8 => Some(Call::bond_extra(
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                )),
                x if x == 2usize as u8 => Some(Call::unbond(_parity_codec::Decode::decode(input)?)),
                x if x == 3usize as u8 => Some(Call::unbond_with_punish(
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                )),
                x if x == 4usize as u8 => Some(Call::promise_extra(
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                )),
                x if x == 5usize as u8 => Some(Call::withdraw_unbonded()),
                x if x == 6usize as u8 => Some(Call::validate(
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                )),
                x if x == 7usize as u8 => {
                    Some(Call::nominate(_parity_codec::Decode::decode(input)?))
                }
                x if x == 8usize as u8 => Some(Call::chill()),
                x if x == 9usize as u8 => {
                    Some(Call::set_payee(_parity_codec::Decode::decode(input)?))
                }
                x if x == 10usize as u8 => {
                    Some(Call::set_controller(_parity_codec::Decode::decode(input)?))
                }
                x if x == 11usize as u8 => Some(Call::set_validator_count(
                    <<u32 as _parity_codec::HasCompact>::Type as _parity_codec::Decode>::decode(
                        input,
                    )?
                    .into(),
                )),
                x if x == 12usize as u8 => Some(Call::force_new_era()),
                x if x == 13usize as u8 => Some(Call::set_offline_slash_grace(
                    <<u32 as _parity_codec::HasCompact>::Type as _parity_codec::Decode>::decode(
                        input,
                    )?
                    .into(),
                )),
                x if x == 14usize as u8 => Some(Call::set_invulnerables(
                    _parity_codec::Decode::decode(input)?,
                )),
                _ => None,
            }
        }
    }
};
impl<T: Trait> ::srml_support::dispatch::Weighable for Call<T> {
    fn weight(&self, _len: usize) -> ::srml_support::dispatch::Weight {
        match self {
            Call::bond(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::bond_extra(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::unbond(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::unbond_with_punish(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::promise_extra(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::withdraw_unbonded(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::validate(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::nominate(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::chill(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::set_payee(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::set_controller(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::set_validator_count(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::force_new_era(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::set_offline_slash_grace(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::set_invulnerables(..) => ::srml_support::dispatch::Weighable::weight(
                &::srml_support::dispatch::TransactionWeight::default(),
                _len,
            ),
            Call::__PhantomItem(_, _) => ::std::rt::begin_panic_fmt(
                &::std::fmt::Arguments::new_v1(
                    &["internal error: entered unreachable code: "],
                    &match (&"__PhantomItem should never be used.",) {
                        (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                    },
                ),
                &("srml/staking/src/lib.rs", 467u32, 1u32),
            ),
        }
    }
}
impl<T: Trait> ::srml_support::dispatch::Clone for Call<T> {
    fn clone(&self) -> Self {
        match *self {
            Call::bond(ref controller, ref value, ref payee, ref promise_month) => Call::bond(
                (*controller).clone(),
                (*value).clone(),
                (*payee).clone(),
                (*promise_month).clone(),
            ),
            Call::bond_extra(ref value, ref promise_month) => {
                Call::bond_extra((*value).clone(), (*promise_month).clone())
            }
            Call::unbond(ref value) => Call::unbond((*value).clone()),
            Call::unbond_with_punish(ref value, ref expire_time) => {
                Call::unbond_with_punish((*value).clone(), (*expire_time).clone())
            }
            Call::promise_extra(ref value, ref promise_month) => {
                Call::promise_extra((*value).clone(), (*promise_month).clone())
            }
            Call::withdraw_unbonded() => Call::withdraw_unbonded(),
            Call::validate(ref name, ref ratio, ref unstake_threshold) => Call::validate(
                (*name).clone(),
                (*ratio).clone(),
                (*unstake_threshold).clone(),
            ),
            Call::nominate(ref targets) => Call::nominate((*targets).clone()),
            Call::chill() => Call::chill(),
            Call::set_payee(ref payee) => Call::set_payee((*payee).clone()),
            Call::set_controller(ref controller) => Call::set_controller((*controller).clone()),
            Call::set_validator_count(ref new) => Call::set_validator_count((*new).clone()),
            Call::force_new_era() => Call::force_new_era(),
            Call::set_offline_slash_grace(ref new) => Call::set_offline_slash_grace((*new).clone()),
            Call::set_invulnerables(ref validators) => {
                Call::set_invulnerables((*validators).clone())
            }
            _ => ::std::rt::begin_panic(
                "internal error: entered unreachable code",
                &("srml/staking/src/lib.rs", 467u32, 1u32),
            ),
        }
    }
}
impl<T: Trait> ::srml_support::dispatch::PartialEq for Call<T> {
    fn eq(&self, _other: &Self) -> bool {
        match *self {
            Call::bond(ref controller, ref value, ref payee, ref promise_month) => {
                let self_params = (controller, value, payee, promise_month);
                if let Call::bond(ref controller, ref value, ref payee, ref promise_month) = *_other
                {
                    self_params == (controller, value, payee, promise_month)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::bond_extra(ref value, ref promise_month) => {
                let self_params = (value, promise_month);
                if let Call::bond_extra(ref value, ref promise_month) = *_other {
                    self_params == (value, promise_month)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::unbond(ref value) => {
                let self_params = (value,);
                if let Call::unbond(ref value) = *_other {
                    self_params == (value,)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::unbond_with_punish(ref value, ref expire_time) => {
                let self_params = (value, expire_time);
                if let Call::unbond_with_punish(ref value, ref expire_time) = *_other {
                    self_params == (value, expire_time)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::promise_extra(ref value, ref promise_month) => {
                let self_params = (value, promise_month);
                if let Call::promise_extra(ref value, ref promise_month) = *_other {
                    self_params == (value, promise_month)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::withdraw_unbonded() => {
                let self_params = ();
                if let Call::withdraw_unbonded() = *_other {
                    self_params == ()
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::validate(ref name, ref ratio, ref unstake_threshold) => {
                let self_params = (name, ratio, unstake_threshold);
                if let Call::validate(ref name, ref ratio, ref unstake_threshold) = *_other {
                    self_params == (name, ratio, unstake_threshold)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::nominate(ref targets) => {
                let self_params = (targets,);
                if let Call::nominate(ref targets) = *_other {
                    self_params == (targets,)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::chill() => {
                let self_params = ();
                if let Call::chill() = *_other {
                    self_params == ()
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::set_payee(ref payee) => {
                let self_params = (payee,);
                if let Call::set_payee(ref payee) = *_other {
                    self_params == (payee,)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::set_controller(ref controller) => {
                let self_params = (controller,);
                if let Call::set_controller(ref controller) = *_other {
                    self_params == (controller,)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::set_validator_count(ref new) => {
                let self_params = (new,);
                if let Call::set_validator_count(ref new) = *_other {
                    self_params == (new,)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::force_new_era() => {
                let self_params = ();
                if let Call::force_new_era() = *_other {
                    self_params == ()
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::set_offline_slash_grace(ref new) => {
                let self_params = (new,);
                if let Call::set_offline_slash_grace(ref new) = *_other {
                    self_params == (new,)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            Call::set_invulnerables(ref validators) => {
                let self_params = (validators,);
                if let Call::set_invulnerables(ref validators) = *_other {
                    self_params == (validators,)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/staking/src/lib.rs", 467u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            _ => ::std::rt::begin_panic(
                "internal error: entered unreachable code",
                &("srml/staking/src/lib.rs", 467u32, 1u32),
            ),
        }
    }
}
impl<T: Trait> ::srml_support::dispatch::Eq for Call<T> {}
#[cfg(feature = "std")]
impl<T: Trait> ::srml_support::dispatch::fmt::Debug for Call<T> {
    fn fmt(
        &self,
        _f: &mut ::srml_support::dispatch::fmt::Formatter,
    ) -> ::srml_support::dispatch::result::Result<(), ::srml_support::dispatch::fmt::Error> {
        match *self {
            Call::bond(ref controller, ref value, ref payee, ref promise_month) => {
                _f.write_fmt(::std::fmt::Arguments::new_v1(
                    &["", ""],
                    &match (
                        &"bond",
                        &(
                            controller.clone(),
                            value.clone(),
                            payee.clone(),
                            promise_month.clone(),
                        ),
                    ) {
                        (arg0, arg1) => [
                            ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                            ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                        ],
                    },
                ))
            }
            Call::bond_extra(ref value, ref promise_month) => {
                _f.write_fmt(::std::fmt::Arguments::new_v1(
                    &["", ""],
                    &match (&"bond_extra", &(value.clone(), promise_month.clone())) {
                        (arg0, arg1) => [
                            ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                            ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                        ],
                    },
                ))
            }
            Call::unbond(ref value) => _f.write_fmt(::std::fmt::Arguments::new_v1(
                &["", ""],
                &match (&"unbond", &(value.clone(),)) {
                    (arg0, arg1) => [
                        ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                        ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                    ],
                },
            )),
            Call::unbond_with_punish(ref value, ref expire_time) => {
                _f.write_fmt(::std::fmt::Arguments::new_v1(
                    &["", ""],
                    &match (&"unbond_with_punish", &(value.clone(), expire_time.clone())) {
                        (arg0, arg1) => [
                            ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                            ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                        ],
                    },
                ))
            }
            Call::promise_extra(ref value, ref promise_month) => {
                _f.write_fmt(::std::fmt::Arguments::new_v1(
                    &["", ""],
                    &match (&"promise_extra", &(value.clone(), promise_month.clone())) {
                        (arg0, arg1) => [
                            ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                            ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                        ],
                    },
                ))
            }
            Call::withdraw_unbonded() => _f.write_fmt(::std::fmt::Arguments::new_v1(
                &["", ""],
                &match (&"withdraw_unbonded", &()) {
                    (arg0, arg1) => [
                        ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                        ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                    ],
                },
            )),
            Call::validate(ref name, ref ratio, ref unstake_threshold) => {
                _f.write_fmt(::std::fmt::Arguments::new_v1(
                    &["", ""],
                    &match (
                        &"validate",
                        &(name.clone(), ratio.clone(), unstake_threshold.clone()),
                    ) {
                        (arg0, arg1) => [
                            ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                            ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                        ],
                    },
                ))
            }
            Call::nominate(ref targets) => _f.write_fmt(::std::fmt::Arguments::new_v1(
                &["", ""],
                &match (&"nominate", &(targets.clone(),)) {
                    (arg0, arg1) => [
                        ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                        ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                    ],
                },
            )),
            Call::chill() => _f.write_fmt(::std::fmt::Arguments::new_v1(
                &["", ""],
                &match (&"chill", &()) {
                    (arg0, arg1) => [
                        ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                        ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                    ],
                },
            )),
            Call::set_payee(ref payee) => _f.write_fmt(::std::fmt::Arguments::new_v1(
                &["", ""],
                &match (&"set_payee", &(payee.clone(),)) {
                    (arg0, arg1) => [
                        ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                        ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                    ],
                },
            )),
            Call::set_controller(ref controller) => _f.write_fmt(::std::fmt::Arguments::new_v1(
                &["", ""],
                &match (&"set_controller", &(controller.clone(),)) {
                    (arg0, arg1) => [
                        ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                        ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                    ],
                },
            )),
            Call::set_validator_count(ref new) => _f.write_fmt(::std::fmt::Arguments::new_v1(
                &["", ""],
                &match (&"set_validator_count", &(new.clone(),)) {
                    (arg0, arg1) => [
                        ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                        ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                    ],
                },
            )),
            Call::force_new_era() => _f.write_fmt(::std::fmt::Arguments::new_v1(
                &["", ""],
                &match (&"force_new_era", &()) {
                    (arg0, arg1) => [
                        ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                        ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                    ],
                },
            )),
            Call::set_offline_slash_grace(ref new) => _f.write_fmt(::std::fmt::Arguments::new_v1(
                &["", ""],
                &match (&"set_offline_slash_grace", &(new.clone(),)) {
                    (arg0, arg1) => [
                        ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                        ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                    ],
                },
            )),
            Call::set_invulnerables(ref validators) => _f.write_fmt(::std::fmt::Arguments::new_v1(
                &["", ""],
                &match (&"set_invulnerables", &(validators.clone(),)) {
                    (arg0, arg1) => [
                        ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                        ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                    ],
                },
            )),
            _ => ::std::rt::begin_panic(
                "internal error: entered unreachable code",
                &("srml/staking/src/lib.rs", 467u32, 1u32),
            ),
        }
    }
}
impl<T: Trait> ::srml_support::dispatch::Dispatchable for Call<T> {
    type Trait = T;
    type Origin = T::Origin;
    fn dispatch(self, _origin: Self::Origin) -> ::srml_support::dispatch::Result {
        match self {
            Call::bond(controller, value, payee, promise_month) => {
                <Module<T>>::bond(_origin, controller, value, payee, promise_month)
            }
            Call::bond_extra(value, promise_month) => {
                <Module<T>>::bond_extra(_origin, value, promise_month)
            }
            Call::unbond(value) => <Module<T>>::unbond(_origin, value),
            Call::unbond_with_punish(value, expire_time) => {
                <Module<T>>::unbond_with_punish(_origin, value, expire_time)
            }
            Call::promise_extra(value, promise_month) => {
                <Module<T>>::promise_extra(_origin, value, promise_month)
            }
            Call::withdraw_unbonded() => <Module<T>>::withdraw_unbonded(_origin),
            Call::validate(name, ratio, unstake_threshold) => {
                <Module<T>>::validate(_origin, name, ratio, unstake_threshold)
            }
            Call::nominate(targets) => <Module<T>>::nominate(_origin, targets),
            Call::chill() => <Module<T>>::chill(_origin),
            Call::set_payee(payee) => <Module<T>>::set_payee(_origin, payee),
            Call::set_controller(controller) => <Module<T>>::set_controller(_origin, controller),
            Call::set_validator_count(new) => {
                system::ensure_root(_origin)?;
                <Module<T>>::set_validator_count(new)
            }
            Call::force_new_era() => {
                system::ensure_root(_origin)?;
                <Module<T>>::force_new_era()
            }
            Call::set_offline_slash_grace(new) => {
                system::ensure_root(_origin)?;
                <Module<T>>::set_offline_slash_grace(new)
            }
            Call::set_invulnerables(validators) => {
                system::ensure_root(_origin)?;
                <Module<T>>::set_invulnerables(validators)
            }
            Call::__PhantomItem(_, _) => ::std::rt::begin_panic_fmt(
                &::std::fmt::Arguments::new_v1(
                    &["internal error: entered unreachable code: "],
                    &match (&"__PhantomItem should never be used.",) {
                        (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                    },
                ),
                &("srml/staking/src/lib.rs", 467u32, 1u32),
            ),
        }
    }
}
impl<T: Trait> ::srml_support::dispatch::Callable for Module<T> {
    type Call = Call<T>;
}
impl<T: Trait> Module<T> {
    #[doc(hidden)]
    pub fn dispatch<D: ::srml_support::dispatch::Dispatchable<Trait = T>>(
        d: D,
        origin: D::Origin,
    ) -> ::srml_support::dispatch::Result {
        d.dispatch(origin)
    }
}
impl<T: Trait> Module<T> {
    #[doc(hidden)]
    pub fn call_functions() -> &'static [::srml_support::dispatch::FunctionMetadata] {
        &[
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("bond"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("controller"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode(
                            "<T::Lookup as StaticLookup>::Source",
                        ),
                    },
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("value"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode(
                            "StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>",
                        ),
                    },
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("payee"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("RewardDestination"),
                    },
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("promise_month"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("u32"),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("bond_extra"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("value"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode(
                            "StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>",
                        ),
                    },
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("promise_month"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("u32"),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("unbond"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("value"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode(
                            "StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>",
                        ),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    r" for normal_ring or normal_kton, follow the original substrate pattern",
                    r" for time_deposit_ring, transform it into normal_ring first",
                    r" modify time_deposit_items and time_deposit_ring amount",
                ]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("unbond_with_punish"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("value"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("RingBalanceOf<T>"),
                    },
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("expire_time"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("T::Moment"),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("promise_extra"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("value"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("RingBalanceOf<T>"),
                    },
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("promise_month"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("u32"),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    r" called by controller",
                ]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("withdraw_unbonded"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    r" may both withdraw ring and kton at the same time",
                ]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("validate"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("name"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("Vec<u8>"),
                    },
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("ratio"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("u32"),
                    },
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode(
                            "unstake_threshold",
                        ),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("u32"),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("nominate"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("targets"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode(
                            "Vec<<T::Lookup as StaticLookup>::Source>",
                        ),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("chill"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("set_payee"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("payee"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("RewardDestination"),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("set_controller"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("controller"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode(
                            "<T::Lookup as StaticLookup>::Source",
                        ),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("set_validator_count"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("new"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("Compact<u32>"),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    r" The ideal number of validators.",
                ]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("force_new_era"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("set_offline_slash_grace"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("new"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("Compact<u32>"),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    r" Set the offline slash grace period.",
                ]),
            },
            ::srml_support::dispatch::FunctionMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("set_invulnerables"),
                arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    ::srml_support::dispatch::FunctionArgumentMetadata {
                        name: ::srml_support::dispatch::DecodeDifferent::Encode("validators"),
                        ty: ::srml_support::dispatch::DecodeDifferent::Encode("Vec<T::AccountId>"),
                    },
                ]),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    r" Set the validators who cannot be slashed (if any).",
                ]),
            },
        ]
    }
}
impl<T: 'static + Trait> Module<T> {
    #[doc(hidden)]
    pub fn module_constants_metadata() -> &'static [::srml_support::dispatch::ModuleConstantMetadata]
    {
        #[allow(non_upper_case_types)]
        #[allow(non_camel_case_types)]
        struct SessionsPerEraDefaultByteGetter<T: Trait>(
            ::srml_support::dispatch::marker::PhantomData<(T)>,
        );
        impl<T: 'static + Trait> ::srml_support::dispatch::DefaultByte
            for SessionsPerEraDefaultByteGetter<T>
        {
            fn default_byte(&self) -> ::srml_support::dispatch::Vec<u8> {
                let value: SessionIndex = T::SessionsPerEra::get();
                ::srml_support::dispatch::Encode::encode(&value)
            }
        }
        #[allow(non_upper_case_types)]
        #[allow(non_camel_case_types)]
        struct BondingDurationDefaultByteGetter<T: Trait>(
            ::srml_support::dispatch::marker::PhantomData<(T)>,
        );
        impl<T: 'static + Trait> ::srml_support::dispatch::DefaultByte
            for BondingDurationDefaultByteGetter<T>
        {
            fn default_byte(&self) -> ::srml_support::dispatch::Vec<u8> {
                let value: EraIndex = T::BondingDuration::get();
                ::srml_support::dispatch::Encode::encode(&value)
            }
        }
        &[
            ::srml_support::dispatch::ModuleConstantMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("SessionsPerEra"),
                ty: ::srml_support::dispatch::DecodeDifferent::Encode("SessionIndex"),
                value: ::srml_support::dispatch::DecodeDifferent::Encode(
                    ::srml_support::dispatch::DefaultByteGetter(
                        &SessionsPerEraDefaultByteGetter::<T>(
                            ::srml_support::dispatch::marker::PhantomData,
                        ),
                    ),
                ),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    r" Number of sessions per era.",
                ]),
            },
            ::srml_support::dispatch::ModuleConstantMetadata {
                name: ::srml_support::dispatch::DecodeDifferent::Encode("BondingDuration"),
                ty: ::srml_support::dispatch::DecodeDifferent::Encode("EraIndex"),
                value: ::srml_support::dispatch::DecodeDifferent::Encode(
                    ::srml_support::dispatch::DefaultByteGetter(
                        &BondingDurationDefaultByteGetter::<T>(
                            ::srml_support::dispatch::marker::PhantomData,
                        ),
                    ),
                ),
                documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                    r" Number of eras that staked funds must remain bonded for.",
                ]),
            },
        ]
    }
}
impl<T: Trait> Module<T> {
    /// The total that can be slashed from a validator controller account as of
    /// right now.
    pub fn slashable_balance(who: &T::AccountId) -> ExtendedBalance {
        Self::staker(who).total
    }
    fn bond_helper_in_ring(
        stash: &T::AccountId,
        controller: &T::AccountId,
        value: RingBalanceOf<T>,
        promise_month: u32,
        mut ledger: StakingLedger<T::AccountId, T::Moment, RingBalanceOf<T>, KtonBalanceOf<T>>,
    ) {
        if promise_month >= 3 {
            let kton_return = utils::compute_kton_return::<T>(value, promise_month);
            ledger.active_deposit_ring += value;
            ledger.total_deposit_ring += value;
            let kton_positive_imbalance = T::Kton::deposit_creating(stash, kton_return);
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
        Self::update_ledger(controller, &ledger, StakingBalance::Ring(value));
    }
    fn bond_helper_in_kton(
        controller: &T::AccountId,
        value: KtonBalanceOf<T>,
        mut ledger: StakingLedger<T::AccountId, T::Moment, RingBalanceOf<T>, KtonBalanceOf<T>>,
    ) {
        ledger.total_kton += value;
        ledger.active_kton += value;
        Self::update_ledger(controller, &ledger, StakingBalance::Kton(value));
    }
    fn update_ledger(
        controller: &T::AccountId,
        ledger: &StakingLedger<T::AccountId, T::Moment, RingBalanceOf<T>, KtonBalanceOf<T>>,
        staking_balance: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
    ) {
        let stash = &ledger.stash;
        match staking_balance {
            StakingBalance::Ring(_) => T::Ring::set_lock(
                STAKING_ID,
                stash,
                ledger.total_ring,
                T::BlockNumber::max_value(),
                WithdrawReasons::all(),
            ),
            StakingBalance::Kton(_) => T::Kton::set_lock(
                STAKING_ID,
                stash,
                ledger.total_kton,
                T::BlockNumber::max_value(),
                WithdrawReasons::all(),
            ),
        }
        <Ledger<T>>::insert(controller, ledger);
    }
    fn slash_validator(stash: &T::AccountId, slash_ratio_in_u32: u32) {
        let slash_ratio = Perbill::from_parts(slash_ratio_in_u32);
        let exposures = Self::staker(stash);
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
        ledger: &mut StakingLedger<T::AccountId, T::Moment, RingBalanceOf<T>, KtonBalanceOf<T>>,
        value: StakingBalance<RingBalanceOf<T>, KtonBalanceOf<T>>,
    ) -> (RingBalanceOf<T>, KtonBalanceOf<T>) {
        match value {
            StakingBalance::Ring(ring) => {
                let StakingLedger {
                    total_ring,
                    active_ring,
                    total_deposit_ring,
                    active_deposit_ring,
                    deposit_items,
                    unlocking,
                    ..
                } = ledger;
                let total_value = ring.min(*total_ring);
                let normal_active_value = total_value.min(*active_ring - *active_deposit_ring);
                <RingPool<T>>::mutate(|ring| *ring -= normal_active_value);
                *total_ring -= normal_active_value;
                *active_ring -= normal_active_value;
                let mut value_left = total_value - normal_active_value;
                if !value_left.is_zero() {
                    deposit_items.sort_unstable_by_key(|item| {
                        u64::max_value() - item.expire_time.to_owned().saturated_into::<u64>()
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
                if !value_left.is_zero() {
                    unlocking.drain_filter(|chunk| {
                        if let StakingBalance::Ring(ring) = value {
                            if value_left.is_zero() {
                                return false;
                            }
                            let value = value_left.min(ring);
                            let new_value = ring - value;
                            chunk.value = StakingBalance::Ring(new_value);
                            value_left -= value;
                            *total_ring -= value;
                            if chunk.is_time_deposit {
                                *total_deposit_ring -= value;
                            }
                            new_value.is_zero()
                        } else {
                            false
                        }
                    });
                }
                Self::update_ledger(controller, ledger, StakingBalance::Ring(0.into()));
                (total_value, 0.into())
            }
            StakingBalance::Kton(kton) => {
                let StakingLedger {
                    total_kton,
                    active_kton,
                    unlocking,
                    ..
                } = ledger;
                let total_value = kton.min(*total_kton);
                let active_value = total_value.min(*active_kton);
                *total_kton -= active_value;
                *active_kton -= active_value;
                <KtonPool<T>>::mutate(|kton| *kton -= active_value);
                let mut value_left = total_value - active_value;
                if !value_left.is_zero() {
                    unlocking.drain_filter(|chunk| {
                        if let StakingBalance::Kton(kton) = value {
                            if value_left.is_zero() {
                                return false;
                            }
                            let value = value_left.min(kton);
                            let new_value = kton - value;
                            chunk.value = StakingBalance::Kton(new_value);
                            value_left -= value;
                            *total_kton -= value;
                            new_value.is_zero()
                        } else {
                            false
                        }
                    });
                }
                Self::update_ledger(controller, ledger, StakingBalance::Kton(0.into()));
                (0.into(), total_value)
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
            let len = validators.len() as u32;
            let len: RingBalanceOf<T> = len.max(1).into();
            let block_reward_per_validator = reward / len;
            for v in validators.iter() {
                Self::reward_validator(v, block_reward_per_validator);
            }
            Self::deposit_event(RawEvent::Reward(block_reward_per_validator));
        }
        CurrentEra::mutate(|s| *s += 1);
        if Self::current_era() % T::ErasPerEpoch::get() == 0 {
            Self::new_epoch();
        }
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
        let off_the_table = Self::validator(stash).validator_payment_ratio * reward;
        let reward = reward - off_the_table;
        let mut imbalance = <RingPositiveImbalanceOf<T>>::zero();
        let validator_cut = if reward.is_zero() {
            0.into()
        } else {
            let exposures = Self::staker(stash);
            let total = exposures.total.max(1);
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
        match Self::payee(stash) {
            RewardDestination::Controller => Self::bonded(stash)
                .and_then(|controller| T::Ring::deposit_into_existing(&controller, amount).ok()),
            RewardDestination::Stash => T::Ring::deposit_into_existing(stash, amount).ok(),
        }
    }
    fn slashable_balance_of(stash: &T::AccountId) -> ExtendedBalance {
        Self::bonded(stash)
            .and_then(Self::ledger)
            .map(|ledger| {
                ledger.active_ring.saturated_into::<ExtendedBalance>()
                    + ledger.active_kton.saturated_into::<ExtendedBalance>()
                        * Self::kton_vote_weight()
                        / ACCURACY
            })
            .unwrap_or_default()
    }
    /// Select a new validator set from the assembled stakers and their role preferences.
    ///
    /// Returns the new `SlotStake` value.
    fn select_validators() -> (ExtendedBalance, Option<Vec<T::AccountId>>) {
        let maybe_elected_set = elect::<T, _, _, _>(
            Self::validator_count() as _,
            Self::minimum_validator_count().max(1) as _,
            <Validators<T>>::enumerate(),
            <Nominators<T>>::enumerate(),
            Self::slashable_balance_of,
        );
        let (elected_stashes, assignments) = if let Some(elected_set) = maybe_elected_set {
            elected_set
        } else {
            return (Self::slot_stake(), None);
        };
        let ratio_of = |b, p| (p as ExtendedBalance).saturating_mul(b) / ACCURACY;
        let assignments_with_stakes: Vec<_> = assignments
            .into_iter()
            .map(|(n, a)| {
                (
                    n.clone(),
                    Self::slashable_balance_of(&n),
                    a.into_iter()
                        .map(|(acc, r)| (acc, r, ratio_of(Self::slashable_balance_of(&n), r)))
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
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
                exposures.insert(e.to_owned(), item);
            });
        for (n, _, assignment) in &assignments_with_stakes {
            for (c, _, s) in assignment {
                if let Some(expo) = exposures.get_mut(c) {
                    expo.total = expo.total.saturating_add(*s);
                    expo.others.push(IndividualExpo {
                        who: n.to_owned(),
                        value: *s,
                    });
                }
            }
        }
        if true {
            let tolerance = 0_u128;
            let iterations = 2_usize;
            let mut assignments_with_votes = assignments_with_stakes.clone();
            equalize::<T>(
                &mut assignments_with_votes,
                &mut exposures,
                tolerance,
                iterations,
            );
        }
        for v in Self::current_elected().iter() {
            <Stakers<T>>::remove(v);
            let slash_count = <SlashCount<T>>::take(v);
            if slash_count > 1 {
                <SlashCount<T>>::insert(v, slash_count - 1);
            }
        }
        let mut slot_stake = ExtendedBalance::max_value();
        for (c, e) in exposures.iter() {
            if e.total < slot_stake {
                slot_stake = e.total;
            }
            <Stakers<T>>::insert(c, e);
        }
        SlotStake::put(&slot_stake);
        <CurrentElected<T>>::put(&elected_stashes);
        let validators: Vec<_> = elected_stashes
            .into_iter()
            .map(|s| Self::bonded(s).unwrap_or_default())
            .collect();
        (slot_stake, Some(validators))
    }
    fn apply_force_new_era() {
        ForceNewEra::put(true);
    }
    /// Call when a validator is determined to be offline. `count` is the
    /// number of offenses the validator has committed.
    ///
    /// NOTE: This is called with the controller (not the stash) account id.
    pub fn on_offline_validator(controller: T::AccountId, count: usize) {
        let stash = if let Some(ledger) = Self::ledger(&controller) {
            ledger.stash
        } else {
            return;
        };
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
                count as _,
            );
            <RecentlyOffline<T>>::mutate(|v| {
                if v.len() >= RECENT_OFFLINE_COUNT {
                    *v.iter_mut()
                        .min_by(|x, y| x.1.cmp(&y.1))
                        .expect("v is non-empty; qed") = item;
                } else {
                    v.push(item);
                }
            });
        }
        if <Validators<T>>::exists(&stash) {
            let prefs = Self::validator(&stash);
            let unstake_threshold = prefs.unstake_threshold.min(MAX_UNSTAKE_THRESHOLD);
            let max_slashes = grace + unstake_threshold;
            if new_slash_count > max_slashes {
                let offline_slash_ratio_base = *Self::offline_slash().encode_as();
                let slash_ratio_in_u32 = offline_slash_ratio_base
                    .checked_shl(unstake_threshold)
                    .unwrap_or_default();
                Self::slash_validator(&stash, slash_ratio_in_u32);
                <Validators<T>>::remove(&stash);
                let _ = <session::Module<T>>::disable(&controller);
                Self::deposit_event(RawEvent::OfflineSlash(stash, slash_ratio_in_u32));
            } else {
                Self::deposit_event(RawEvent::OfflineWarning(stash, slash_count));
            };
        }
    }
    fn kton_vote_weight() -> ExtendedBalance {
        let total_ring = Self::ring_pool().saturated_into::<ExtendedBalance>();
        let total_kton = Self::kton_pool().saturated_into::<ExtendedBalance>().max(1);
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
