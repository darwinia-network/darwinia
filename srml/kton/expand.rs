#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use ::std::prelude::v1::*;
#[macro_use]
extern crate std as std;
mod imbalance {
    use super::{Imbalance, Saturating, StorageValue, Trait, Zero};
    use rstd::mem;
    pub struct PositiveImbalance<T: Trait>(T::Balance);
    impl<T: Trait> PositiveImbalance<T> {
        pub fn new(amount: T::Balance) -> Self {
            PositiveImbalance(amount)
        }
    }
    impl<T: Trait> Drop for PositiveImbalance<T> {
        /// Basic drop handler will just square up the total issuance.
        fn drop(&mut self) {
            <super::TotalIssuance<T>>::mutate(|v| *v = v.saturating_add(self.0));
        }
    }
    impl<T: Trait> Imbalance<T::Balance> for PositiveImbalance<T> {
        type Opposite = NegativeImbalance<T>;
        fn zero() -> Self {
            Self(Zero::zero())
        }
        fn drop_zero(self) -> Result<(), Self> {
            if self.0.is_zero() {
                Ok(())
            } else {
                Err(self)
            }
        }
        fn split(self, amount: T::Balance) -> (Self, Self) {
            let first = self.0.min(amount);
            let second = self.0 - first;
            mem::forget(self);
            (Self(first), Self(second))
        }
        fn merge(mut self, other: Self) -> Self {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);
            self
        }
        fn subsume(&mut self, other: Self) {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);
        }
        fn offset(self, other: Self::Opposite) -> Result<Self, Self::Opposite> {
            let (a, b) = (self.0, other.0);
            mem::forget((self, other));
            if a >= b {
                Ok(Self(a - b))
            } else {
                Err(NegativeImbalance::new(b - a))
            }
        }
        fn peek(&self) -> T::Balance {
            self.0
        }
    }
    pub struct NegativeImbalance<T: Trait>(T::Balance);
    impl<T: Trait> NegativeImbalance<T> {
        pub fn new(amount: T::Balance) -> Self {
            NegativeImbalance(amount)
        }
    }
    impl<T: Trait> Drop for NegativeImbalance<T> {
        /// Basic drop handler will just square up the total issuance.
        fn drop(&mut self) {
            <super::TotalIssuance<T>>::mutate(|v| *v = v.saturating_sub(self.0));
        }
    }
    impl<T: Trait> Imbalance<T::Balance> for NegativeImbalance<T> {
        type Opposite = PositiveImbalance<T>;
        fn zero() -> Self {
            Self(Zero::zero())
        }
        fn drop_zero(self) -> Result<(), Self> {
            if self.0.is_zero() {
                Ok(())
            } else {
                Err(self)
            }
        }
        fn split(self, amount: T::Balance) -> (Self, Self) {
            let first = self.0.min(amount);
            let second = self.0 - first;
            mem::forget(self);
            (Self(first), Self(second))
        }
        fn merge(mut self, other: Self) -> Self {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);
            self
        }
        fn subsume(&mut self, other: Self) {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);
        }
        fn offset(self, other: Self::Opposite) -> Result<Self, Self::Opposite> {
            let (a, b) = (self.0, other.0);
            mem::forget((self, other));
            if a >= b {
                Ok(Self(a - b))
            } else {
                Err(PositiveImbalance::new(b - a))
            }
        }
        fn peek(&self) -> T::Balance {
            self.0
        }
    }
}
use imbalance::{NegativeImbalance, PositiveImbalance};
use parity_codec::{Codec, Decode, Encode};
use primitives::traits::{
    Bounded, CheckedAdd, CheckedSub, MaybeSerializeDebug, Member, Saturating, SimpleArithmetic,
    StaticLookup, Zero,
};
#[cfg(feature = "std")]
use rstd::borrow::ToOwned;
use rstd::vec::Vec;
use srml_support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result as SResult,
    traits::{
        Currency, ExistenceRequirement, Imbalance, LockIdentifier, LockableCurrency, OnUnbalanced,
        SignedImbalance, UpdateBalanceOutcome, WithdrawReason, WithdrawReasons,
    },
    Parameter, StorageMap, StorageValue,
};
use system::ensure_signed;
/// Struct to encode the vesting schedule of an individual account.
#[structural_match]
#[rustc_copy_clone_marker]
pub struct VestingSchedule<Balance> {
    /// Locked amount at genesis.
    pub offset: Balance,
    /// Amount that gets unlocked every block from genesis.
    pub per_block: Balance,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::clone::Clone> ::std::clone::Clone for VestingSchedule<Balance> {
    #[inline]
    fn clone(&self) -> VestingSchedule<Balance> {
        match *self {
            VestingSchedule {
                offset: ref __self_0_0,
                per_block: ref __self_0_1,
            } => VestingSchedule {
                offset: ::std::clone::Clone::clone(&(*__self_0_0)),
                per_block: ::std::clone::Clone::clone(&(*__self_0_1)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::marker::Copy> ::std::marker::Copy for VestingSchedule<Balance> {}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::cmp::PartialEq> ::std::cmp::PartialEq for VestingSchedule<Balance> {
    #[inline]
    fn eq(&self, other: &VestingSchedule<Balance>) -> bool {
        match *other {
            VestingSchedule {
                offset: ref __self_1_0,
                per_block: ref __self_1_1,
            } => match *self {
                VestingSchedule {
                    offset: ref __self_0_0,
                    per_block: ref __self_0_1,
                } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
            },
        }
    }
    #[inline]
    fn ne(&self, other: &VestingSchedule<Balance>) -> bool {
        match *other {
            VestingSchedule {
                offset: ref __self_1_0,
                per_block: ref __self_1_1,
            } => match *self {
                VestingSchedule {
                    offset: ref __self_0_0,
                    per_block: ref __self_0_1,
                } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::cmp::Eq> ::std::cmp::Eq for VestingSchedule<Balance> {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<Balance>;
            let _: ::std::cmp::AssertParamIsEq<Balance>;
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_VestingSchedule: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<Balance> _parity_codec::Encode for VestingSchedule<Balance>
    where
        Balance: _parity_codec::Encode,
        Balance: _parity_codec::Encode,
        Balance: _parity_codec::Encode,
        Balance: _parity_codec::Encode,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            dest.push(&self.offset);
            dest.push(&self.per_block);
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_VestingSchedule: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<Balance> _parity_codec::Decode for VestingSchedule<Balance>
    where
        Balance: _parity_codec::Decode,
        Balance: _parity_codec::Decode,
        Balance: _parity_codec::Decode,
        Balance: _parity_codec::Decode,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            Some(VestingSchedule {
                offset: _parity_codec::Decode::decode(input)?,
                per_block: _parity_codec::Decode::decode(input)?,
            })
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::fmt::Debug> ::std::fmt::Debug for VestingSchedule<Balance> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            VestingSchedule {
                offset: ref __self_0_0,
                per_block: ref __self_0_1,
            } => {
                let mut debug_trait_builder = f.debug_struct("VestingSchedule");
                let _ = debug_trait_builder.field("offset", &&(*__self_0_0));
                let _ = debug_trait_builder.field("per_block", &&(*__self_0_1));
                debug_trait_builder.finish()
            }
        }
    }
}
impl<Balance: Copy + SimpleArithmetic> VestingSchedule<Balance> {
    /// Amount locked at block `n`.
    pub fn locked_at<BlockNumber>(&self, n: BlockNumber) -> Balance
    where
        Balance: From<BlockNumber>,
    {
        if let Some(x) = Balance::from(n).checked_mul(&self.per_block) {
            self.offset.max(x) - x
        } else {
            Balance::zero()
        }
    }
}
#[structural_match]
pub struct BalanceLock<Balance, BlockNumber> {
    pub id: LockIdentifier,
    pub amount: Balance,
    pub until: BlockNumber,
    pub reasons: WithdrawReasons,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::clone::Clone, BlockNumber: ::std::clone::Clone> ::std::clone::Clone
    for BalanceLock<Balance, BlockNumber>
{
    #[inline]
    fn clone(&self) -> BalanceLock<Balance, BlockNumber> {
        match *self {
            BalanceLock {
                id: ref __self_0_0,
                amount: ref __self_0_1,
                until: ref __self_0_2,
                reasons: ref __self_0_3,
            } => BalanceLock {
                id: ::std::clone::Clone::clone(&(*__self_0_0)),
                amount: ::std::clone::Clone::clone(&(*__self_0_1)),
                until: ::std::clone::Clone::clone(&(*__self_0_2)),
                reasons: ::std::clone::Clone::clone(&(*__self_0_3)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::cmp::PartialEq, BlockNumber: ::std::cmp::PartialEq> ::std::cmp::PartialEq
    for BalanceLock<Balance, BlockNumber>
{
    #[inline]
    fn eq(&self, other: &BalanceLock<Balance, BlockNumber>) -> bool {
        match *other {
            BalanceLock {
                id: ref __self_1_0,
                amount: ref __self_1_1,
                until: ref __self_1_2,
                reasons: ref __self_1_3,
            } => match *self {
                BalanceLock {
                    id: ref __self_0_0,
                    amount: ref __self_0_1,
                    until: ref __self_0_2,
                    reasons: ref __self_0_3,
                } => {
                    (*__self_0_0) == (*__self_1_0)
                        && (*__self_0_1) == (*__self_1_1)
                        && (*__self_0_2) == (*__self_1_2)
                        && (*__self_0_3) == (*__self_1_3)
                }
            },
        }
    }
    #[inline]
    fn ne(&self, other: &BalanceLock<Balance, BlockNumber>) -> bool {
        match *other {
            BalanceLock {
                id: ref __self_1_0,
                amount: ref __self_1_1,
                until: ref __self_1_2,
                reasons: ref __self_1_3,
            } => match *self {
                BalanceLock {
                    id: ref __self_0_0,
                    amount: ref __self_0_1,
                    until: ref __self_0_2,
                    reasons: ref __self_0_3,
                } => {
                    (*__self_0_0) != (*__self_1_0)
                        || (*__self_0_1) != (*__self_1_1)
                        || (*__self_0_2) != (*__self_1_2)
                        || (*__self_0_3) != (*__self_1_3)
                }
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::cmp::Eq, BlockNumber: ::std::cmp::Eq> ::std::cmp::Eq
    for BalanceLock<Balance, BlockNumber>
{
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<LockIdentifier>;
            let _: ::std::cmp::AssertParamIsEq<Balance>;
            let _: ::std::cmp::AssertParamIsEq<BlockNumber>;
            let _: ::std::cmp::AssertParamIsEq<WithdrawReasons>;
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_BalanceLock: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<Balance, BlockNumber> _parity_codec::Encode for BalanceLock<Balance, BlockNumber>
    where
        Balance: _parity_codec::Encode,
        Balance: _parity_codec::Encode,
        BlockNumber: _parity_codec::Encode,
        BlockNumber: _parity_codec::Encode,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            dest.push(&self.id);
            dest.push(&self.amount);
            dest.push(&self.until);
            dest.push(&self.reasons);
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DECODE_FOR_BalanceLock: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<Balance, BlockNumber> _parity_codec::Decode for BalanceLock<Balance, BlockNumber>
    where
        Balance: _parity_codec::Decode,
        Balance: _parity_codec::Decode,
        BlockNumber: _parity_codec::Decode,
        BlockNumber: _parity_codec::Decode,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            Some(BalanceLock {
                id: _parity_codec::Decode::decode(input)?,
                amount: _parity_codec::Decode::decode(input)?,
                until: _parity_codec::Decode::decode(input)?,
                reasons: _parity_codec::Decode::decode(input)?,
            })
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl<Balance: ::std::fmt::Debug, BlockNumber: ::std::fmt::Debug> ::std::fmt::Debug
    for BalanceLock<Balance, BlockNumber>
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            BalanceLock {
                id: ref __self_0_0,
                amount: ref __self_0_1,
                until: ref __self_0_2,
                reasons: ref __self_0_3,
            } => {
                let mut debug_trait_builder = f.debug_struct("BalanceLock");
                let _ = debug_trait_builder.field("id", &&(*__self_0_0));
                let _ = debug_trait_builder.field("amount", &&(*__self_0_1));
                let _ = debug_trait_builder.field("until", &&(*__self_0_2));
                let _ = debug_trait_builder.field("reasons", &&(*__self_0_3));
                debug_trait_builder.finish()
            }
        }
    }
}
pub trait Trait: timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Balance: Codec
        + Copy
        + Default
        + From<Self::BlockNumber>
        + MaybeSerializeDebug
        + Member
        + Parameter
        + SimpleArithmetic;
    type OnMinted: OnUnbalanced<PositiveImbalance<Self>>;
    type OnRemoval: OnUnbalanced<NegativeImbalance<Self>>;
}
/// [`RawEvent`] specialized for the configuration [`Trait`]
///
/// [`RawEvent`]: enum.RawEvent.html
/// [`Trait`]: trait.Trait.html
pub type Event<T> = RawEvent<<T as system::Trait>::AccountId, <T as Trait>::Balance>;
/// Events for this module.
///
#[structural_match]
pub enum RawEvent<AccountId, Balance> {
    /// Transfer succeeded (from, to, value, fees).
    TokenTransfer(AccountId, AccountId, Balance),
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::clone::Clone, Balance: ::std::clone::Clone> ::std::clone::Clone
    for RawEvent<AccountId, Balance>
{
    #[inline]
    fn clone(&self) -> RawEvent<AccountId, Balance> {
        match (&*self,) {
            (&RawEvent::TokenTransfer(ref __self_0, ref __self_1, ref __self_2),) => {
                RawEvent::TokenTransfer(
                    ::std::clone::Clone::clone(&(*__self_0)),
                    ::std::clone::Clone::clone(&(*__self_1)),
                    ::std::clone::Clone::clone(&(*__self_2)),
                )
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::cmp::PartialEq, Balance: ::std::cmp::PartialEq> ::std::cmp::PartialEq
    for RawEvent<AccountId, Balance>
{
    #[inline]
    fn eq(&self, other: &RawEvent<AccountId, Balance>) -> bool {
        match (&*self, &*other) {
            (
                &RawEvent::TokenTransfer(ref __self_0, ref __self_1, ref __self_2),
                &RawEvent::TokenTransfer(ref __arg_1_0, ref __arg_1_1, ref __arg_1_2),
            ) => {
                (*__self_0) == (*__arg_1_0)
                    && (*__self_1) == (*__arg_1_1)
                    && (*__self_2) == (*__arg_1_2)
            }
        }
    }
    #[inline]
    fn ne(&self, other: &RawEvent<AccountId, Balance>) -> bool {
        match (&*self, &*other) {
            (
                &RawEvent::TokenTransfer(ref __self_0, ref __self_1, ref __self_2),
                &RawEvent::TokenTransfer(ref __arg_1_0, ref __arg_1_1, ref __arg_1_2),
            ) => {
                (*__self_0) != (*__arg_1_0)
                    || (*__self_1) != (*__arg_1_1)
                    || (*__self_2) != (*__arg_1_2)
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::cmp::Eq, Balance: ::std::cmp::Eq> ::std::cmp::Eq
    for RawEvent<AccountId, Balance>
{
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<AccountId>;
            let _: ::std::cmp::AssertParamIsEq<AccountId>;
            let _: ::std::cmp::AssertParamIsEq<Balance>;
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_ENCODE_FOR_RawEvent: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate parity_codec as _parity_codec;
    impl<AccountId, Balance> _parity_codec::Encode for RawEvent<AccountId, Balance>
    where
        AccountId: _parity_codec::Encode,
        AccountId: _parity_codec::Encode,
        AccountId: _parity_codec::Encode,
        AccountId: _parity_codec::Encode,
        Balance: _parity_codec::Encode,
        Balance: _parity_codec::Encode,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            match *self {
                RawEvent::TokenTransfer(ref aa, ref ba, ref ca) => {
                    dest.push_byte(0usize as u8);
                    dest.push(aa);
                    dest.push(ba);
                    dest.push(ca);
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
    impl<AccountId, Balance> _parity_codec::Decode for RawEvent<AccountId, Balance>
    where
        AccountId: _parity_codec::Decode,
        AccountId: _parity_codec::Decode,
        AccountId: _parity_codec::Decode,
        AccountId: _parity_codec::Decode,
        Balance: _parity_codec::Decode,
        Balance: _parity_codec::Decode,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            match input.read_byte()? {
                x if x == 0usize as u8 => Some(RawEvent::TokenTransfer(
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                    _parity_codec::Decode::decode(input)?,
                )),
                _ => None,
            }
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl<AccountId: ::std::fmt::Debug, Balance: ::std::fmt::Debug> ::std::fmt::Debug
    for RawEvent<AccountId, Balance>
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match (&*self,) {
            (&RawEvent::TokenTransfer(ref __self_0, ref __self_1, ref __self_2),) => {
                let mut debug_trait_builder = f.debug_tuple("TokenTransfer");
                let _ = debug_trait_builder.field(&&(*__self_0));
                let _ = debug_trait_builder.field(&&(*__self_1));
                let _ = debug_trait_builder.field(&&(*__self_2));
                debug_trait_builder.finish()
            }
        }
    }
}
impl<AccountId, Balance> From<RawEvent<AccountId, Balance>> for () {
    fn from(_: RawEvent<AccountId, Balance>) -> () {
        ()
    }
}
impl<AccountId, Balance> RawEvent<AccountId, Balance> {
    #[allow(dead_code)]
    pub fn metadata() -> &'static [::srml_support::event::EventMetadata] {
        &[::srml_support::event::EventMetadata {
            name: ::srml_support::event::DecodeDifferent::Encode("TokenTransfer"),
            arguments: ::srml_support::event::DecodeDifferent::Encode(&[
                "AccountId",
                "AccountId",
                "Balance",
            ]),
            documentation: ::srml_support::event::DecodeDifferent::Encode(&[
                r" Transfer succeeded (from, to, value, fees).",
            ]),
        }]
    }
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
    const PREFIX_FOR_MinimumBalance: &'static str;
    const PREFIX_FOR_TotalIssuance: &'static str;
    const PREFIX_FOR_FreeBalance: &'static str;
    const PREFIX_FOR_ReservedBalance: &'static str;
    const PREFIX_FOR_Locks: &'static str;
    const PREFIX_FOR_TotalLock: &'static str;
    const PREFIX_FOR_Vesting: &'static str;
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
    const PREFIX_FOR_MinimumBalance: &'static str = "Kton MinimumBalance";
    const PREFIX_FOR_TotalIssuance: &'static str = "Kton TotalIssuance";
    const PREFIX_FOR_FreeBalance: &'static str = "Kton FreeBalance";
    const PREFIX_FOR_ReservedBalance: &'static str = "Kton ReservedBalance";
    const PREFIX_FOR_Locks: &'static str = "Kton Locks";
    const PREFIX_FOR_TotalLock: &'static str = "Kton TotalLock";
    const PREFIX_FOR_Vesting: &'static str = "Kton Vesting";
}
/// For Currency and LockableCurrency Trait
/// The total `units issued in the system.
pub struct MinimumBalance<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > for MinimumBalance < T > { type Query = T :: Balance ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Kton MinimumBalance" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: key ( ) ) . unwrap_or_else ( | | 0 . into ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: key ( ) ) . unwrap_or_else ( | | 0 . into ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: put ( & val , storage ) ; ret } }
pub struct TotalIssuance<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > for TotalIssuance < T > { type Query = T :: Balance ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Kton TotalIssuance" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: put ( & val , storage ) ; ret } }
pub struct FreeBalance<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > for FreeBalance < T > { type Query = T :: Balance ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Kton FreeBalance" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( x : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( x , & mut key ) ; key } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: key_for ( key ) ; storage . get ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: key_for ( key ) ; storage . take ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: get ( key , storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: insert ( key , & val , storage ) ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: AppendableStorageMap < T :: AccountId , T :: Balance > for FreeBalance < T > { }
pub struct ReservedBalance<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > for ReservedBalance < T > { type Query = T :: Balance ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Kton ReservedBalance" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( x : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( x , & mut key ) ; key } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: key_for ( key ) ; storage . get ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: key_for ( key ) ; storage . take ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: get ( key , storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: insert ( key , & val , storage ) ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: AppendableStorageMap < T :: AccountId , T :: Balance > for ReservedBalance < T > { }
pub struct Locks<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < BalanceLock < T :: Balance , T :: BlockNumber > > > for Locks < T > { type Query = Vec < BalanceLock < T :: Balance , T :: BlockNumber > > ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Kton Locks" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( x : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < BalanceLock < T :: Balance , T :: BlockNumber > > > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( x , & mut key ) ; key } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < BalanceLock < T :: Balance , T :: BlockNumber > > > > :: key_for ( key ) ; storage . get ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < BalanceLock < T :: Balance , T :: BlockNumber > > > > :: key_for ( key ) ; storage . take ( & key [ .. ] ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < BalanceLock < T :: Balance , T :: BlockNumber > > > > :: get ( key , storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < BalanceLock < T :: Balance , T :: BlockNumber > > > > :: insert ( key , & val , storage ) ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: AppendableStorageMap < T :: AccountId , Vec < BalanceLock < T :: Balance , T :: BlockNumber > > > for Locks < T > { }
pub struct TotalLock<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > for TotalLock < T > { type Query = T :: Balance ; # [ doc = r" Get the storage key." ] fn key ( ) -> & 'static [ u8 ] { "Kton TotalLock" . as_bytes ( ) } # [ doc = r" Load the value from the provided storage instance." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & S ) -> Self :: Query { storage . get ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Take a value from storage, removing it afterwards." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > > ( storage : & mut S ) -> Self :: Query { storage . take ( < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: key ( ) ) . unwrap_or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key." ] fn mutate < R , F , S > ( f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Twox128 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: get ( storage ) ; let ret = f ( & mut val ) ; < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: put ( & val , storage ) ; ret } }
pub struct Vesting<T: Trait>(
    self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T,)>,
);
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , VestingSchedule < T :: Balance > > for Vesting < T > { type Query = Option < VestingSchedule < T :: Balance > > ; type Hasher = self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 ; # [ doc = r" Get the prefix key in storage." ] fn prefix ( ) -> & 'static [ u8 ] { "Kton Vesting" . as_bytes ( ) } # [ doc = r" Get the storage key used to fetch a value corresponding to a specific key." ] fn key_for ( x : & T :: AccountId ) -> self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: vec :: Vec < u8 > { let mut key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , VestingSchedule < T :: Balance > > > :: prefix ( ) . to_vec ( ) ; self :: sr_api_hidden_includes_decl_storage :: hidden_include :: codec :: Encode :: encode_to ( x , & mut key ) ; key } # [ doc = r" Load the value associated with the given key from the map." ] fn get < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , VestingSchedule < T :: Balance > > > :: key_for ( key ) ; storage . get ( & key [ .. ] ) . or_else ( | | Default :: default ( ) ) } # [ doc = r" Take the value, reading and removing it." ] fn take < S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > > ( key : & T :: AccountId , storage : & mut S ) -> Self :: Query { let key = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , VestingSchedule < T :: Balance > > > :: key_for ( key ) ; storage . take ( & key [ .. ] ) . or_else ( | | Default :: default ( ) ) } # [ doc = r" Mutate the value under a key" ] fn mutate < R , F , S > ( key : & T :: AccountId , f : F , storage : & mut S ) -> R where F : FnOnce ( & mut Self :: Query ) -> R , S : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: HashedStorage < self :: sr_api_hidden_includes_decl_storage :: hidden_include :: Blake2_256 > { let mut val = < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , VestingSchedule < T :: Balance > > > :: get ( key , storage ) ; let ret = f ( & mut val ) ; match val { Some ( ref val ) => < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , VestingSchedule < T :: Balance > > > :: insert ( key , & val , storage ) , None => < Self as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , VestingSchedule < T :: Balance > > > :: remove ( key , storage ) , } ; ret } }
impl < T : Trait > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: AppendableStorageMap < T :: AccountId , VestingSchedule < T :: Balance > > for Vesting < T > { }
trait Store {
    type MinimumBalance;
    type TotalIssuance;
    type FreeBalance;
    type ReservedBalance;
    type Locks;
    type TotalLock;
    type Vesting;
}
#[doc(hidden)]
pub struct __GetByteStructMinimumBalance<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_MinimumBalance:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructMinimumBalance<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_MinimumBalance
            .get_or_init(|| {
                let def_val: T::Balance = 0.into();
                <T::Balance as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructTotalIssuance<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_TotalIssuance:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructTotalIssuance<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_TotalIssuance
            .get_or_init(|| {
                let def_val: T::Balance = Default::default();
                <T::Balance as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructFreeBalance<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_FreeBalance:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructFreeBalance<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_FreeBalance
            .get_or_init(|| {
                let def_val: T::Balance = Default::default();
                <T::Balance as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructReservedBalance<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_ReservedBalance:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructReservedBalance<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_ReservedBalance
            .get_or_init(|| {
                let def_val: T::Balance = Default::default();
                <T::Balance as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructLocks<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_Locks:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructLocks<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_Locks
            .get_or_init(|| {
                let def_val: Vec<BalanceLock<T::Balance, T::BlockNumber>> = Default::default();
                <Vec<BalanceLock<T::Balance, T::BlockNumber>> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructTotalLock<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_TotalLock:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructTotalLock<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_TotalLock
            .get_or_init(|| {
                let def_val: T::Balance = Default::default();
                <T::Balance as Encode>::encode(&def_val)
            })
            .clone()
    }
}
#[doc(hidden)]
pub struct __GetByteStructVesting<T>(
    pub self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::marker::PhantomData<(T)>,
);
#[cfg(feature = "std")]
#[allow(non_upper_case_globals)]
static __CACHE_GET_BYTE_STRUCT_Vesting:
    self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell<
        self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8>,
    > = self::sr_api_hidden_includes_decl_storage::hidden_include::once_cell::sync::OnceCell::INIT;
#[cfg(feature = "std")]
impl<T: Trait> self::sr_api_hidden_includes_decl_storage::hidden_include::metadata::DefaultByte
    for __GetByteStructVesting<T>
{
    fn default_byte(
        &self,
    ) -> self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::vec::Vec<u8> {
        use self::sr_api_hidden_includes_decl_storage::hidden_include::codec::Encode;
        __CACHE_GET_BYTE_STRUCT_Vesting
            .get_or_init(|| {
                let def_val: Option<VestingSchedule<T::Balance>> = Default::default();
                <Option<VestingSchedule<T::Balance>> as Encode>::encode(&def_val)
            })
            .clone()
    }
}
impl<T: Trait> Store for Module<T> {
    type MinimumBalance = MinimumBalance<T>;
    type TotalIssuance = TotalIssuance<T>;
    type FreeBalance = FreeBalance<T>;
    type ReservedBalance = ReservedBalance<T>;
    type Locks = Locks<T>;
    type TotalLock = TotalLock<T>;
    type Vesting = Vesting<T>;
}
impl<T: 'static + Trait> Module<T> {
    /// For Currency and LockableCurrency Trait
    /// The total `units issued in the system.
    pub fn minimum_balance() -> T::Balance {
        < MinimumBalance < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn total_issuance() -> T::Balance {
        < TotalIssuance < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn free_balance<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> T::Balance {
        < FreeBalance < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn reserved_balance<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> T::Balance {
        < ReservedBalance < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn locks<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> Vec<BalanceLock<T::Balance, T::BlockNumber>> {
        < Locks < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , Vec < BalanceLock < T :: Balance , T :: BlockNumber > > > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn total_lock() -> T::Balance {
        < TotalLock < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: get ( & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    pub fn vesting<
        K: self::sr_api_hidden_includes_decl_storage::hidden_include::rstd::borrow::Borrow<
            T::AccountId,
        >,
    >(
        key: K,
    ) -> Option<VestingSchedule<T::Balance>> {
        < Vesting < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , VestingSchedule < T :: Balance > > > :: get ( key . borrow ( ) , & self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: RuntimeStorage )
    }
    #[doc(hidden)]pub fn store_metadata_functions ( ) -> & 'static [ self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata ]{
        {
            & [ self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "MinimumBalance" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::Balance" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructMinimumBalance :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ " For Currency and LockableCurrency Trait" , " The total `units issued in the system." ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "TotalIssuance" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::Balance" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructTotalIssuance :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "FreeBalance" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::Balance" ) , is_linked : false , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructFreeBalance :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "ReservedBalance" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::Balance" ) , is_linked : false , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructReservedBalance :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Locks" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Vec<BalanceLock<T::Balance, T::BlockNumber>>" ) , is_linked : false , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructLocks :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "TotalLock" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Default , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Plain ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::Balance" ) ) , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructTotalLock :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } , self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryMetadata { name : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "Vesting" ) , modifier : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryModifier :: Optional , ty : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageEntryType :: Map { hasher : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: StorageHasher :: Blake2_256 , key : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "T::AccountId" ) , value : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( "VestingSchedule<T::Balance>" ) , is_linked : false , } , default : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DefaultByteGetter ( & __GetByteStructVesting :: < T > ( self :: sr_api_hidden_includes_decl_storage :: hidden_include :: rstd :: marker :: PhantomData ) ) ) , documentation : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: metadata :: DecodeDifferent :: Encode ( & [ ] ) , } ]
        }
    }
    #[doc(hidden)]
    pub fn store_metadata_name() -> &'static str {
        "Kton"
    }
}
#[cfg(feature = "std")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(bound(
    serialize = "Vec < ( T :: AccountId , T :: Balance ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::Serialize, Vec < ( T :: AccountId , T :: BlockNumber , T :: BlockNumber ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::Serialize, "
))]
#[serde(bound(
    deserialize = "Vec < ( T :: AccountId , T :: Balance ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::de::DeserializeOwned, Vec < ( T :: AccountId , T :: BlockNumber , T :: BlockNumber ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include::serde::de::DeserializeOwned, "
))]
pub struct GenesisConfig<T: Trait> {
    pub balances: Vec<(T::AccountId, T::Balance)>,
    pub vesting: Vec<(T::AccountId, T::BlockNumber, T::BlockNumber)>,
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_SERIALIZE_FOR_GenesisConfig: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<T: Trait> _serde::Serialize for GenesisConfig<T>
    where
        Vec<(T::AccountId, T::Balance)>:
            self::sr_api_hidden_includes_decl_storage::hidden_include::serde::Serialize,
        Vec<(T::AccountId, T::BlockNumber, T::BlockNumber)>:
            self::sr_api_hidden_includes_decl_storage::hidden_include::serde::Serialize,
    {
        fn serialize<__S>(&self, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = match _serde::Serializer::serialize_struct(
                __serializer,
                "GenesisConfig",
                false as usize + 1 + 1,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "balances",
                &self.balances,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "vesting",
                &self.vesting,
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
        Vec<(T::AccountId, T::Balance)>:
            self::sr_api_hidden_includes_decl_storage::hidden_include::serde::de::DeserializeOwned,
        Vec<(T::AccountId, T::BlockNumber, T::BlockNumber)>:
            self::sr_api_hidden_includes_decl_storage::hidden_include::serde::de::DeserializeOwned,
    {
        fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
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
                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                            _serde::de::Unexpected::Unsigned(__value),
                            &"field index 0 <= i < 2",
                        )),
                    }
                }
                fn visit_str<__E>(self, __value: &str) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "balances" => _serde::export::Ok(__Field::__field0),
                        "vesting" => _serde::export::Ok(__Field::__field1),
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
                        b"balances" => _serde::export::Ok(__Field::__field0),
                        b"vesting" => _serde::export::Ok(__Field::__field1),
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
            struct __Visitor < 'de , T : Trait > where Vec < ( T :: AccountId , T :: Balance ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , Vec < ( T :: AccountId , T :: BlockNumber , T :: BlockNumber ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned { marker : _serde :: export :: PhantomData < GenesisConfig < T > > , lifetime : _serde :: export :: PhantomData < & 'de ( ) > , }
            impl < 'de , T : Trait > _serde :: de :: Visitor < 'de > for __Visitor < 'de , T > where Vec < ( T :: AccountId , T :: Balance ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned , Vec < ( T :: AccountId , T :: BlockNumber , T :: BlockNumber ) > : self :: sr_api_hidden_includes_decl_storage :: hidden_include :: serde :: de :: DeserializeOwned { type Value = GenesisConfig < T > ; fn expecting ( & self , __formatter : & mut _serde :: export :: Formatter ) -> _serde :: export :: fmt :: Result { _serde :: export :: Formatter :: write_str ( __formatter , "struct GenesisConfig" ) } # [ inline ] fn visit_seq < __A > ( self , mut __seq : __A ) -> _serde :: export :: Result < Self :: Value , __A :: Error > where __A : _serde :: de :: SeqAccess < 'de > { let __field0 = match match _serde :: de :: SeqAccess :: next_element :: < Vec < ( T :: AccountId , T :: Balance ) > > ( & mut __seq ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { _serde :: export :: Some ( __value ) => __value , _serde :: export :: None => { return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 0usize , & "struct GenesisConfig with 2 elements" ) ) ; } } ; let __field1 = match match _serde :: de :: SeqAccess :: next_element :: < Vec < ( T :: AccountId , T :: BlockNumber , T :: BlockNumber ) > > ( & mut __seq ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { _serde :: export :: Some ( __value ) => __value , _serde :: export :: None => { return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 1usize , & "struct GenesisConfig with 2 elements" ) ) ; } } ; _serde :: export :: Ok ( GenesisConfig { balances : __field0 , vesting : __field1 , } ) } # [ inline ] fn visit_map < __A > ( self , mut __map : __A ) -> _serde :: export :: Result < Self :: Value , __A :: Error > where __A : _serde :: de :: MapAccess < 'de > { let mut __field0 : _serde :: export :: Option < Vec < ( T :: AccountId , T :: Balance ) > > = _serde :: export :: None ; let mut __field1 : _serde :: export :: Option < Vec < ( T :: AccountId , T :: BlockNumber , T :: BlockNumber ) > > = _serde :: export :: None ; while let _serde :: export :: Some ( __key ) = match _serde :: de :: MapAccess :: next_key :: < __Field > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } { match __key { __Field :: __field0 => { if _serde :: export :: Option :: is_some ( & __field0 ) { return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "balances" ) ) ; } __field0 = _serde :: export :: Some ( match _serde :: de :: MapAccess :: next_value :: < Vec < ( T :: AccountId , T :: Balance ) > > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } ) ; } __Field :: __field1 => { if _serde :: export :: Option :: is_some ( & __field1 ) { return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "vesting" ) ) ; } __field1 = _serde :: export :: Some ( match _serde :: de :: MapAccess :: next_value :: < Vec < ( T :: AccountId , T :: BlockNumber , T :: BlockNumber ) > > ( & mut __map ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } ) ; } } } let __field0 = match __field0 { _serde :: export :: Some ( __field0 ) => __field0 , _serde :: export :: None => match _serde :: private :: de :: missing_field ( "balances" ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } , } ; let __field1 = match __field1 { _serde :: export :: Some ( __field1 ) => __field1 , _serde :: export :: None => match _serde :: private :: de :: missing_field ( "vesting" ) { _serde :: export :: Ok ( __val ) => __val , _serde :: export :: Err ( __err ) => { return _serde :: export :: Err ( __err ) ; } } , } ; _serde :: export :: Ok ( GenesisConfig { balances : __field0 , vesting : __field1 , } ) } }
            const FIELDS: &'static [&'static str] = &["balances", "vesting"];
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
            balances: Default::default(),
            vesting: Default::default(),
        }
    }
}
#[cfg(feature = "std")]
impl<T: Trait> GenesisConfig<T> {
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
            let v = (|config: &GenesisConfig<T>| {
                config
                    .balances
                    .iter()
                    .fold(0.into(), |acc, &(_, n)| acc + n)
            })(&self);
            < TotalIssuance < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageValue < T :: Balance > > :: put ( & v , storage ) ;
        }
        {
            let data = (|config: &GenesisConfig<T>| config.balances.to_owned())(&self);
            data . into_iter ( ) . for_each ( | ( k , v ) | { < FreeBalance < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , T :: Balance > > :: insert ( & k , & v , storage ) ; } ) ;
        }
        {
            let data = (|config: &GenesisConfig<T>| {
                config
                    .vesting
                    .iter()
                    .filter_map(|&(ref who, begin, length)| {
                        let begin = <T::Balance as From<T::BlockNumber>>::from(begin);
                        let length = <T::Balance as From<T::BlockNumber>>::from(length);
                        config
                            .balances
                            .iter()
                            .find(|(id, _)| id == who)
                            .map(|&(_, balance)| {
                                let per_block = balance / length.max(1.into());
                                let offset = begin * per_block + balance;
                                (who.to_owned(), VestingSchedule { offset, per_block })
                            })
                    })
                    .collect::<Vec<_>>()
            })(&self);
            data . into_iter ( ) . for_each ( | ( k , v ) | { < Vesting < T > as self :: sr_api_hidden_includes_decl_storage :: hidden_include :: storage :: hashed :: generator :: StorageMap < T :: AccountId , VestingSchedule < T :: Balance > > > :: insert ( & k , & v , storage ) ; } ) ;
        }
        (|_, _, _| {})(storage, c, &self);
        Ok(())
    }
}
#[cfg(feature = "std")]
impl < T : Trait , __GeneratedInstance : __GeneratedInstantiable > self :: sr_api_hidden_includes_decl_storage :: hidden_include :: runtime_primitives :: BuildModuleGenesisStorage < T , __GeneratedInstance > for GenesisConfig < T > { fn build_module_genesis_storage ( self , r : & mut self :: sr_api_hidden_includes_decl_storage :: hidden_include :: runtime_primitives :: StorageOverlay , c : & mut self :: sr_api_hidden_includes_decl_storage :: hidden_include :: runtime_primitives :: ChildrenStorageOverlay ) -> std :: result :: Result < ( ) , String > { self . assimilate_storage :: < > ( r , c ) } }
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
    pub fn transfer(
        origin: T::Origin,
        dest: <T::Lookup as StaticLookup>::Source,
        value: T::Balance,
    ) -> ::srml_support::dispatch::Result {
        {
            let transactor = ensure_signed(origin)?;
            let dest = T::Lookup::lookup(dest)?;
            <Self as Currency<_>>::transfer(&transactor, &dest, value)?;
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
    transfer(
        <T::Lookup as StaticLookup>::Source,
        #[codec(compact)] T::Balance,
    ),
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
        T::Balance: _parity_codec::HasCompact,
    {
        fn encode_to<EncOut: _parity_codec::Output>(&self, dest: &mut EncOut) {
            match *self {
                Call::transfer(ref aa, ref ba) => {
                    dest.push_byte(0usize as u8);
                    dest.push(aa);
                    {
                        dest . push ( & < < T :: Balance as _parity_codec :: HasCompact > :: Type as _parity_codec :: EncodeAsRef < '_ , T :: Balance > > :: from ( ba ) ) ;
                    }
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
        T::Balance: _parity_codec::HasCompact,
    {
        fn decode<DecIn: _parity_codec::Input>(input: &mut DecIn) -> Option<Self> {
            match input . read_byte ( ) ? { x if x == 0usize as u8 => { Some ( Call :: transfer ( _parity_codec :: Decode :: decode ( input ) ? , < < T :: Balance as _parity_codec :: HasCompact > :: Type as _parity_codec :: Decode > :: decode ( input ) ? . into ( ) ) ) } _ => None , }
        }
    }
};
impl<T: Trait> ::srml_support::dispatch::Weighable for Call<T> {
    fn weight(&self, _len: usize) -> ::srml_support::dispatch::Weight {
        match self {
            Call::transfer(..) => ::srml_support::dispatch::Weighable::weight(
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
                &("srml/kton/src/lib.rs", 169u32, 1u32),
            ),
        }
    }
}
impl<T: Trait> ::srml_support::dispatch::Clone for Call<T> {
    fn clone(&self) -> Self {
        match *self {
            Call::transfer(ref dest, ref value) => {
                Call::transfer((*dest).clone(), (*value).clone())
            }
            _ => ::std::rt::begin_panic(
                "internal error: entered unreachable code",
                &("srml/kton/src/lib.rs", 169u32, 1u32),
            ),
        }
    }
}
impl<T: Trait> ::srml_support::dispatch::PartialEq for Call<T> {
    fn eq(&self, _other: &Self) -> bool {
        match *self {
            Call::transfer(ref dest, ref value) => {
                let self_params = (dest, value);
                if let Call::transfer(ref dest, ref value) = *_other {
                    self_params == (dest, value)
                } else {
                    match *_other {
                        Call::__PhantomItem(_, _) => ::std::rt::begin_panic(
                            "internal error: entered unreachable code",
                            &("srml/kton/src/lib.rs", 169u32, 1u32),
                        ),
                        _ => false,
                    }
                }
            }
            _ => ::std::rt::begin_panic(
                "internal error: entered unreachable code",
                &("srml/kton/src/lib.rs", 169u32, 1u32),
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
            Call::transfer(ref dest, ref value) => _f.write_fmt(::std::fmt::Arguments::new_v1(
                &["", ""],
                &match (&"transfer", &(dest.clone(), value.clone())) {
                    (arg0, arg1) => [
                        ::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt),
                        ::std::fmt::ArgumentV1::new(arg1, ::std::fmt::Debug::fmt),
                    ],
                },
            )),
            _ => ::std::rt::begin_panic(
                "internal error: entered unreachable code",
                &("srml/kton/src/lib.rs", 169u32, 1u32),
            ),
        }
    }
}
impl<T: Trait> ::srml_support::dispatch::Dispatchable for Call<T> {
    type Trait = T;
    type Origin = T::Origin;
    fn dispatch(self, _origin: Self::Origin) -> ::srml_support::dispatch::Result {
        match self {
            Call::transfer(dest, value) => <Module<T>>::transfer(_origin, dest, value),
            Call::__PhantomItem(_, _) => ::std::rt::begin_panic_fmt(
                &::std::fmt::Arguments::new_v1(
                    &["internal error: entered unreachable code: "],
                    &match (&"__PhantomItem should never be used.",) {
                        (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
                    },
                ),
                &("srml/kton/src/lib.rs", 169u32, 1u32),
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
        &[::srml_support::dispatch::FunctionMetadata {
            name: ::srml_support::dispatch::DecodeDifferent::Encode("transfer"),
            arguments: ::srml_support::dispatch::DecodeDifferent::Encode(&[
                ::srml_support::dispatch::FunctionArgumentMetadata {
                    name: ::srml_support::dispatch::DecodeDifferent::Encode("dest"),
                    ty: ::srml_support::dispatch::DecodeDifferent::Encode(
                        "<T::Lookup as StaticLookup>::Source",
                    ),
                },
                ::srml_support::dispatch::FunctionArgumentMetadata {
                    name: ::srml_support::dispatch::DecodeDifferent::Encode("value"),
                    ty: ::srml_support::dispatch::DecodeDifferent::Encode("Compact<T::Balance>"),
                },
            ]),
            documentation: ::srml_support::dispatch::DecodeDifferent::Encode(&[]),
        }]
    }
}
impl<T: 'static + Trait> Module<T> {
    #[doc(hidden)]
    pub fn module_constants_metadata() -> &'static [::srml_support::dispatch::ModuleConstantMetadata]
    {
        &[]
    }
}
impl<T: Trait> Module<T> {
    pub fn vesting_balance(who: &T::AccountId) -> T::Balance {
        if let Some(v) = Self::vesting(who) {
            Self::free_balance(who)
                .min(v.locked_at::<T::BlockNumber>(<system::Module<T>>::block_number()))
        } else {
            0.into()
        }
    }
    fn set_free_balance(who: &T::AccountId, balance: T::Balance) -> UpdateBalanceOutcome {
        <FreeBalance<T>>::insert(who, balance);
        UpdateBalanceOutcome::Updated
    }
    fn set_reserved_balance(who: &T::AccountId, balance: T::Balance) -> UpdateBalanceOutcome {
        <ReservedBalance<T>>::insert(who, balance);
        UpdateBalanceOutcome::Updated
    }
}
impl<T: Trait> Currency<T::AccountId> for Module<T> {
    type Balance = T::Balance;
    type PositiveImbalance = PositiveImbalance<T>;
    type NegativeImbalance = NegativeImbalance<T>;
    fn total_balance(who: &T::AccountId) -> Self::Balance {
        Self::free_balance(who) + Self::reserved_balance(who)
    }
    fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
        Self::free_balance(who) >= value
    }
    fn total_issuance() -> Self::Balance {
        Self::total_issuance()
    }
    fn minimum_balance() -> Self::Balance {
        Self::minimum_balance()
    }
    fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
        <TotalIssuance<T>>::mutate(|issued| {
            issued.checked_sub(&amount).unwrap_or_else(|| {
                amount = *issued;
                0.into()
            })
        });
        PositiveImbalance::new(amount)
    }
    fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
        <TotalIssuance<T>>::mutate(|issued| {
            *issued = issued.checked_add(&amount).unwrap_or_else(|| {
                amount = Self::Balance::max_value() - *issued;
                Self::Balance::max_value()
            })
        });
        NegativeImbalance::new(amount)
    }
    fn free_balance(who: &T::AccountId) -> Self::Balance {
        <FreeBalance<T>>::get(who)
    }
    fn ensure_can_withdraw(
        who: &T::AccountId,
        _amount: T::Balance,
        reason: WithdrawReason,
        new_balance: T::Balance,
    ) -> SResult {
        match reason {
            WithdrawReason::Reserve | WithdrawReason::Transfer => {
                if Self::vesting_balance(who) > new_balance {
                    return Err("vesting balance too high to send value");
                }
            }
            _ => (),
        }
        let locks = Self::locks(who);
        if !locks.is_empty() {
            let now = <system::Module<T>>::block_number();
            if locks
                .into_iter()
                .all(|l| now < l.until && new_balance < l.amount && l.reasons.contains(reason))
            {
                return Err("account liquidity restrictions prevent withdrawal");
            }
        }
        Ok(())
    }
    fn transfer(transactor: &T::AccountId, dest: &T::AccountId, value: Self::Balance) -> SResult {
        let new_from_balance = Self::free_balance(transactor)
            .checked_sub(&value)
            .ok_or("balance too low to send value")?;
        Self::ensure_can_withdraw(
            transactor,
            value,
            WithdrawReason::Transfer,
            new_from_balance,
        )?;
        let new_to_balance = Self::free_balance(dest)
            .checked_add(&value)
            .ok_or("destination balance too high to receive value")?;
        if transactor != dest {
            Self::set_free_balance(transactor, new_from_balance);
            Self::set_free_balance(dest, new_to_balance);
        }
        Self::deposit_event(RawEvent::TokenTransfer(
            transactor.clone(),
            dest.clone(),
            value,
        ));
        Ok(())
    }
    fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
        let free_balance = Self::free_balance(who);
        let free_slash = free_balance.min(value);
        Self::set_free_balance(who, free_balance - free_slash);
        let remaining_slash = value - free_slash;
        if remaining_slash.is_zero() {
            (NegativeImbalance::new(value), 0.into())
        } else {
            let reserved_balance = Self::reserved_balance(who);
            let reserved_slash = reserved_balance.min(remaining_slash);
            Self::set_reserved_balance(who, reserved_balance - reserved_slash);
            (
                NegativeImbalance::new(free_slash + reserved_slash),
                remaining_slash - reserved_slash,
            )
        }
    }
    fn deposit_into_existing(
        who: &T::AccountId,
        value: Self::Balance,
    ) -> Result<Self::PositiveImbalance, &'static str> {
        if Self::total_balance(who).is_zero() {
            return Err("beneficiary account must pre-exist");
        }
        let balance = Self::free_balance(who);
        let new_balance = balance + value;
        Self::set_free_balance(who, new_balance);
        Ok(PositiveImbalance::new(value))
    }
    fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
        let (imbalance, _) = Self::make_free_balance_be(who, Self::free_balance(who) + value);
        if let SignedImbalance::Positive(p) = imbalance {
            p
        } else {
            Self::PositiveImbalance::zero()
        }
    }
    fn withdraw(
        who: &T::AccountId,
        value: Self::Balance,
        reason: WithdrawReason,
        liveness: ExistenceRequirement,
    ) -> Result<Self::NegativeImbalance, &'static str> {
        if let Some(new_balance) = Self::free_balance(who).checked_sub(&value) {
            if (liveness == ExistenceRequirement::KeepAlive)
                && (new_balance < Self::minimum_balance())
            {
                return Err("payment would kill account");
            }
            Self::ensure_can_withdraw(who, value, reason, new_balance)?;
            Self::set_free_balance(who, new_balance);
            Ok(NegativeImbalance::new(value))
        } else {
            Err("too few free funds in account")
        }
    }
    fn make_free_balance_be(
        who: &T::AccountId,
        balance: Self::Balance,
    ) -> (
        SignedImbalance<Self::Balance, Self::PositiveImbalance>,
        UpdateBalanceOutcome,
    ) {
        let original = Self::free_balance(who);
        let imbalance = if original <= balance {
            SignedImbalance::Positive(PositiveImbalance::new(balance - original))
        } else {
            SignedImbalance::Negative(NegativeImbalance::new(original - balance))
        };
        Self::set_free_balance(who, balance);
        (imbalance, UpdateBalanceOutcome::Updated)
    }
}
impl<T: Trait> LockableCurrency<T::AccountId> for Module<T>
where
    T::Balance: MaybeSerializeDebug,
{
    type Moment = T::BlockNumber;
    fn set_lock(
        id: LockIdentifier,
        who: &T::AccountId,
        amount: T::Balance,
        until: T::BlockNumber,
        reasons: WithdrawReasons,
    ) {
        let now = <system::Module<T>>::block_number();
        let mut new_lock = Some(BalanceLock {
            id,
            amount,
            until,
            reasons,
        });
        let mut locks: Vec<_> = Self::locks(who)
            .into_iter()
            .filter_map(|lock| {
                if lock.id == id {
                    new_lock.take()
                } else if lock.until > now {
                    Some(lock)
                } else {
                    None
                }
            })
            .collect();
        if let Some(lock) = new_lock {
            locks.push(lock);
        }
        <Locks<T>>::insert(who, locks);
    }
    fn extend_lock(
        id: LockIdentifier,
        who: &T::AccountId,
        amount: T::Balance,
        until: T::BlockNumber,
        reasons: WithdrawReasons,
    ) {
        let now = <system::Module<T>>::block_number();
        let mut new_lock = Some(BalanceLock {
            id,
            amount,
            until,
            reasons,
        });
        let mut locks: Vec<_> = Self::locks(who)
            .into_iter()
            .filter_map(|lock| {
                if lock.id == id {
                    new_lock.take().map(|old_lock| BalanceLock {
                        id,
                        amount: lock.amount.max(old_lock.amount),
                        until: lock.until.max(old_lock.until),
                        reasons: lock.reasons | old_lock.reasons,
                    })
                } else if lock.until > now {
                    Some(lock)
                } else {
                    None
                }
            })
            .collect();
        if let Some(lock) = new_lock {
            locks.push(lock);
        }
        <Locks<T>>::insert(who, locks);
    }
    fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
        let now = <system::Module<T>>::block_number();
        <Locks<T>>::mutate(who, |locks| {
            locks.retain(|lock| (lock.until > now) && (lock.id != id));
        });
    }
}
