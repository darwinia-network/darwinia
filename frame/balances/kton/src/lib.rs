#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use codec::{Codec, Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	traits::{
		Currency, ExistenceRequirement, Get, Imbalance, ReservableCurrency, SignedImbalance, Time, TryDrop,
		UpdateBalanceOutcome, VestingCurrency,
	},
	weights::SimpleDispatchInfo,
	Parameter, StorageValue,
};
use frame_system::{self as system, ensure_root, ensure_signed, IsDeadAccount};
use sp_runtime::{
	traits::{
		Bounded, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, Saturating, SimpleArithmetic,
		StaticLookup, Zero,
	},
	DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::{cmp, fmt::Debug, mem, vec::Vec};

use self::imbalances::{NegativeImbalance, PositiveImbalance};
use darwinia_support::{
	BalanceLock, Fee, LockIdentifier, LockableCurrency, WithdrawLock, WithdrawReason, WithdrawReasons,
};

type MomentOf<T> = <<T as Trait>::Time as Time>::Moment;
type RingBalance<T> = <<T as Trait>::RingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Struct to encode the vesting schedule of an individual account.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct VestingSchedule<Balance, BlockNumber> {
	/// Locked amount at genesis.
	pub locked: Balance,
	/// Amount that gets unlocked every block after `starting_block`.
	pub per_block: Balance,
	/// Starting block for unlocking(vesting).
	pub starting_block: BlockNumber,
}

impl<Balance: SimpleArithmetic + Copy, BlockNumber: SimpleArithmetic + Copy> VestingSchedule<Balance, BlockNumber> {
	/// Amount locked at block `n`.
	pub fn locked_at(&self, n: BlockNumber) -> Balance
	where
		Balance: From<BlockNumber>,
	{
		// Number of blocks that count toward vesting
		// Saturating to 0 when n < starting_block
		let vested_block_count = n.saturating_sub(self.starting_block);
		// Return amount that is still locked in vesting
		if let Some(x) = Balance::from(vested_block_count).checked_mul(&self.per_block) {
			self.locked.max(x) - x
		} else {
			Zero::zero()
		}
	}
}

pub trait Trait: frame_system::Trait {
	/// The balance of an account.
	type Balance: Parameter
		+ Member
		+ SimpleArithmetic
		+ Codec
		+ Default
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ From<Self::BlockNumber>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	// TODO doc
	type RingCurrency: Currency<Self::AccountId>;
	// TODO doc
	type TransferPayment: Fee<Self::AccountId, RingBalance<Self>>;
	/// The fee required to make a transfer.
	type TransferFee: Get<RingBalance<Self>>;
	// TODO doc
	type Time: Time;
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
		RingBalance = RingBalance<T>
	{
		/// Transfer succeeded (from, to, value, fees).
		Transfer(AccountId, AccountId, Balance, RingBalance),
		/// A balance was set by root (who, free, reserved).
		BalanceSet(AccountId, Balance, Balance),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Vesting balance too high to send value
		VestingBalance,
		/// Account liquidity restrictions prevent withdrawal
		LiquidityRestrictions,
		/// Got an overflow after adding
		Overflow,
		/// Balance too low to send value
		InsufficientBalance,
		/// A vesting schedule already exists for this account
		ExistingVestingSchedule,
		/// Beneficiary account must pre-exist
		DeadAccount,
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Balances {
		/// The total units issued in the system.
		pub TotalIssuance get(fn total_issuance) build(|config: &GenesisConfig<T>| {
			config.balances.iter().fold(Zero::zero(), |acc: T::Balance, &(_, n)| acc + n)
		}): T::Balance;

		/// Information regarding the vesting of a given account.
		pub Vesting get(fn vesting) build(|config: &GenesisConfig<T>| {
			// Generate initial vesting configuration
			// * who - Account which we are generating vesting configuration for
			// * begin - Block when the account will start to vest
			// * length - Number of blocks from `begin` until fully vested
			// * liquid - Number of units which can be spent before vesting begins
			config.vesting.iter().filter_map(|&(ref who, begin, length, liquid)| {
				let length = <T::Balance as From<T::BlockNumber>>::from(length);

				config.balances.iter()
					.find(|&&(ref w, _)| w == who)
					.map(|&(_, balance)| {
						// Total genesis `balance` minus `liquid` equals funds locked for vesting
						let locked = balance.saturating_sub(liquid);
						// Number of units unlocked per block after `begin`
						let per_block = locked / length.max(One::one());

						(who.to_owned(), VestingSchedule {
							locked: locked,
							per_block: per_block,
							starting_block: begin,
						})
					})
			}).collect::<Vec<_>>()
		}): map T::AccountId => Option<VestingSchedule<T::Balance, T::BlockNumber>>;

		/// The 'free' balance of a given account.
		///
		/// This is the only balance that matters in terms of most operations on tokens. It
		/// alone is used to determine the balance when in the contract execution environment. When this
		/// balance falls below the value of `ExistentialDeposit`, then the 'current account' is
		/// deleted: specifically `FreeBalance`. Further, the `OnFreeBalanceZero` callback
		/// is invoked, giving a chance to external modules to clean up data associated with
		/// the deleted account.
		///
		/// `frame_system::AccountNonce` is also deleted if `ReservedBalance` is also zero (it also gets
		/// collapsed to zero if it ever becomes less than `ExistentialDeposit`.
		pub FreeBalance get(fn free_balance)
			build(|config: &GenesisConfig<T>| config.balances.clone()):
			map T::AccountId => T::Balance;

		/// The amount of the balance of a given account that is externally reserved; this can still get
		/// slashed, but gets slashed last of all.
		///
		/// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
		/// that are still 'owned' by the account holder, but which are suspendable.
		///
		/// When this balance falls below the value of `ExistentialDeposit`, then this 'reserve account'
		/// is deleted: specifically, `ReservedBalance`.
		///
		/// `frame_system::AccountNonce` is also deleted if `FreeBalance` is also zero (it also gets
		/// collapsed to zero if it ever becomes less than `ExistentialDeposit`.)
		pub ReservedBalance get(fn reserved_balance): map T::AccountId => T::Balance;

		/// Any liquidity locks on some account balances.
		pub Locks get(fn locks): map T::AccountId => Vec<BalanceLock<T::Balance, MomentOf<T>>>;
	}
	add_extra_genesis {
		config(balances): Vec<(T::AccountId, T::Balance)>;
		config(vesting): Vec<(T::AccountId, T::BlockNumber, T::BlockNumber, T::Balance)>;
		// ^^ begin, length, amount liquid at genesis
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		/// The fee required to make a transfer.
		const TransferFee: RingBalance<T> = T::TransferFee::get();

		fn deposit_event() = default;

		/// Transfer some liquid free balance to another account.
		///
		/// `transfer` will set the `FreeBalance` of the sender and receiver.
		/// It will decrease the total issuance of the system by the `TransferFee`.
		/// If the sender's account is below the existential deposit as a result
		/// of the transfer, the account will be reaped.
		///
		/// The dispatch origin for this call must be `Signed` by the transactor.
		///
		/// # <weight>
		/// - Dependent on arguments but not critical, given proper implementations for
		///   input config types. See related functions below.
		/// - It contains a limited number of reads and writes internally and no complex computation.
		///
		/// Related functions:
		///
		///   - `ensure_can_withdraw` is always called internally but has a bounded complexity.
		///   - Transferring balances to accounts that did not exist before will cause
		///      `T::OnNewAccount::on_new_account` to be called.
		///   - Removing enough funds from an account will trigger
		///     `T::DustRemoval::on_unbalanced` and `T::OnFreeBalanceZero::on_free_balance_zero`.
		///   - `transfer_keep_alive` works the same way as `transfer`, but has an additional
		///     check that the transfer will not kill the origin account.
		///
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
		pub fn transfer(
			origin,
			dest: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::Balance
		) {
			let transactor = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			<Self as Currency<_>>::transfer(&transactor, &dest, value, ExistenceRequirement::AllowDeath)?;
		}

		/// Set the balances of a given account.
		///
		/// This will alter `FreeBalance` and `ReservedBalance` in storage. it will
		/// also decrease the total issuance of the system (`TotalIssuance`).
		/// If the new free or reserved balance is below the existential deposit,
		/// it will reset the account nonce (`frame_system::AccountNonce`).
		///
		/// The dispatch origin for this call is `root`.
		///
		/// # <weight>
		/// - Independent of the arguments.
		/// - Contains a limited number of reads and writes.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedOperational(50_000)]
		fn set_balance(
			origin,
			who: <T::Lookup as StaticLookup>::Source,
			#[compact] new_free: T::Balance,
			#[compact] new_reserved: T::Balance
		) {
			ensure_root(origin)?;
			let who = T::Lookup::lookup(who)?;

			let current_free = <FreeBalance<T>>::get(&who);
			if new_free > current_free {
				mem::drop(PositiveImbalance::<T>::new(new_free - current_free));
			} else if new_free < current_free {
				mem::drop(NegativeImbalance::<T>::new(current_free - new_free));
			}
			Self::set_free_balance(&who, new_free);

			let current_reserved = <ReservedBalance<T>>::get(&who);
			if new_reserved > current_reserved {
				mem::drop(PositiveImbalance::<T>::new(new_reserved - current_reserved));
			} else if new_reserved < current_reserved {
				mem::drop(NegativeImbalance::<T>::new(current_reserved - new_reserved));
			}
			Self::set_reserved_balance(&who, new_reserved);

			Self::deposit_event(RawEvent::BalanceSet(who, new_free, new_reserved));
		}

		/// Exactly as `transfer`, except the origin must be root and the source account may be
		/// specified.
		#[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
		pub fn force_transfer(
			origin,
			source: <T::Lookup as StaticLookup>::Source,
			dest: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::Balance
		) {
			ensure_root(origin)?;
			let source = T::Lookup::lookup(source)?;
			let dest = T::Lookup::lookup(dest)?;
			<Self as Currency<_>>::transfer(&source, &dest, value, ExistenceRequirement::AllowDeath)?;
		}
	}
}

impl<T: Trait> Module<T> {
	// PRIVATE MUTABLES

	/// Set the reserved balance of an account to some new value. Will enforce `ExistentialDeposit`
	/// law, annulling the account as needed.
	///
	/// Doesn't do any preparatory work for creating a new account, so should only be used when it
	/// is known that the account already exists.
	///
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn set_reserved_balance(who: &T::AccountId, balance: T::Balance) -> UpdateBalanceOutcome {
		<ReservedBalance<T>>::insert(who, balance);
		UpdateBalanceOutcome::Updated
	}

	/// Set the free balance of an account to some new value. Will enforce `ExistentialDeposit`
	/// law, annulling the account as needed.
	///
	/// Doesn't do any preparatory work for creating a new account, so should only be used when it
	/// is known that the account already exists.
	///
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn set_free_balance(who: &T::AccountId, balance: T::Balance) -> UpdateBalanceOutcome {
		// Commented out for now - but consider it instructive.
		// assert!(!Self::total_balance(who).is_zero());
		// assert!(Self::free_balance(who) > T::ExistentialDeposit::get());
		<FreeBalance<T>>::insert(who, balance);
		UpdateBalanceOutcome::Updated
	}
}

// wrapping these imbalances in a private module is necessary to ensure absolute privacy
// of the inner member.
mod imbalances {
	use sp_std::mem;

	use super::{Imbalance, Saturating, StorageValue, Trait, TryDrop, Zero};

	/// Opaque, move-only struct with private fields that serves as a token denoting that
	/// funds have been created without any equal and opposite accounting.
	#[must_use]
	pub struct PositiveImbalance<T: Trait>(T::Balance);

	impl<T: Trait> PositiveImbalance<T> {
		/// Create a new positive imbalance from a balance.
		pub fn new(amount: T::Balance) -> Self {
			PositiveImbalance(amount)
		}
	}

	/// Opaque, move-only struct with private fields that serves as a token denoting that
	/// funds have been destroyed without any equal and opposite accounting.
	#[must_use]
	pub struct NegativeImbalance<T: Trait>(T::Balance);

	impl<T: Trait> NegativeImbalance<T> {
		/// Create a new negative imbalance from a balance.
		pub fn new(amount: T::Balance) -> Self {
			NegativeImbalance(amount)
		}
	}

	impl<T: Trait> TryDrop for PositiveImbalance<T> {
		fn try_drop(self) -> Result<(), Self> {
			self.drop_zero()
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
			self.0.clone()
		}
	}

	impl<T: Trait> TryDrop for NegativeImbalance<T> {
		fn try_drop(self) -> Result<(), Self> {
			self.drop_zero()
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
			self.0.clone()
		}
	}

	impl<T: Trait> Drop for PositiveImbalance<T> {
		/// Basic drop handler will just square up the total issuance.
		fn drop(&mut self) {
			<super::TotalIssuance<T>>::mutate(|v| *v = v.saturating_add(self.0));
		}
	}

	impl<T: Trait> Drop for NegativeImbalance<T> {
		/// Basic drop handler will just square up the total issuance.
		fn drop(&mut self) {
			<super::TotalIssuance<T>>::mutate(|v| *v = v.saturating_sub(self.0));
		}
	}
}

impl<T: Trait> Currency<T::AccountId> for Module<T>
where
	T::Balance: MaybeSerializeDeserialize + Debug,
{
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
		<TotalIssuance<T>>::get()
	}

	fn minimum_balance() -> Self::Balance {
		T::ExistentialDeposit::get()
	}

	fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
		<TotalIssuance<T>>::mutate(|issued| {
			*issued = issued.checked_sub(&amount).unwrap_or_else(|| {
				amount = *issued;
				Zero::zero()
			});
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

	// # <weight>
	// Despite iterating over a list of locks, they are limited by the number of
	// lock IDs, which means the number of runtime modules that intend to use and create locks.
	// # </weight>
	fn ensure_can_withdraw(
		who: &T::AccountId,
		_amount: T::Balance,
		reasons: WithdrawReasons,
		new_balance: T::Balance,
	) -> DispatchResult {
		if reasons.intersects(WithdrawReason::Reserve | WithdrawReason::Transfer)
			&& Self::vesting_balance(who) > new_balance
		{
			Err(Error::<T>::VestingBalance)?
		}
		let locks = Self::locks(who);
		if locks.is_empty() {
			return Ok(());
		}

		let now = T::Time::now();
		if locks
			.into_iter()
			.all(|l| l.withdraw_lock.can_withdraw(now, new_balance) || !l.reasons.intersects(reasons))
		{
			Ok(())
		} else {
			Err(Error::<T>::LiquidityRestrictions.into())
		}
	}

	fn transfer(
		transactor: &T::AccountId,
		dest: &T::AccountId,
		value: Self::Balance,
		existence_requirement: ExistenceRequirement,
	) -> DispatchResult {
		if transactor == dest {
			return Ok(());
		}

		let transfer_fee = T::TransferFee::get();

		let new_from_kton = Self::free_balance(transactor)
			.checked_sub(&value)
			.ok_or(Error::<T>::InsufficientBalance)?;
		Self::ensure_can_withdraw(transactor, value, WithdrawReason::Transfer.into(), new_from_kton)?;

		let new_to_kton = Self::free_balance(dest)
			.checked_add(&value)
			.ok_or(Error::<T>::Overflow)?;

		T::TransferPayment::pay_transfer_fee(transactor, transfer_fee, existence_requirement)?;
		Self::set_free_balance(transactor, new_from_kton);
		// Take action on the set_free_balance call.
		// This will emit events that _resulted_ from the transfer.
		Self::set_free_balance(dest, new_to_kton);

		// Emit transfer event.
		Self::deposit_event(RawEvent::Transfer(
			transactor.to_owned(),
			dest.to_owned(),
			value,
			transfer_fee,
		));

		Ok(())
	}

	fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		let free_balance = Self::free_balance(who);
		let free_slash = cmp::min(free_balance, value);

		Self::set_free_balance(who, free_balance - free_slash);
		let remaining_slash = value - free_slash;
		// NOTE: `slash()` prefers free balance, but assumes that reserve balance can be drawn
		// from in extreme circumstances. `can_slash()` should be used prior to `slash()` to avoid having
		// to draw from reserved funds, however we err on the side of punishment if things are inconsistent
		// or `can_slash` wasn't used appropriately.
		if !remaining_slash.is_zero() {
			let reserved_balance = Self::reserved_balance(who);
			let reserved_slash = cmp::min(reserved_balance, remaining_slash);
			Self::set_reserved_balance(who, reserved_balance - reserved_slash);
			(
				NegativeImbalance::new(free_slash + reserved_slash),
				remaining_slash - reserved_slash,
			)
		} else {
			(NegativeImbalance::new(value), Zero::zero())
		}
	}

	fn deposit_into_existing(
		who: &T::AccountId,
		value: Self::Balance,
	) -> Result<Self::PositiveImbalance, DispatchError> {
		if Self::total_balance(who).is_zero() {
			Err(Error::<T>::DeadAccount)?
		}
		Self::set_free_balance(who, Self::free_balance(who) + value);
		Ok(PositiveImbalance::new(value))
	}

	fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
		let (imbalance, _) = Self::make_free_balance_be(who, Self::free_balance(who) + value);
		if let SignedImbalance::Positive(p) = imbalance {
			p
		} else {
			// Impossible, but be defensive.
			Self::PositiveImbalance::zero()
		}
	}

	fn withdraw(
		who: &T::AccountId,
		value: Self::Balance,
		reasons: WithdrawReasons,
		_liveness: ExistenceRequirement,
	) -> Result<Self::NegativeImbalance, DispatchError> {
		let old_balance = Self::free_balance(who);
		if let Some(new_balance) = old_balance.checked_sub(&value) {
			Self::ensure_can_withdraw(who, value, reasons, new_balance)?;
			Self::set_free_balance(who, new_balance);
			Ok(NegativeImbalance::new(value))
		} else {
			Err(Error::<T>::InsufficientBalance)?
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
		let outcome = {
			Self::set_free_balance(who, balance);
			UpdateBalanceOutcome::Updated
		};

		(imbalance, outcome)
	}
}

impl<T: Trait> ReservableCurrency<T::AccountId> for Module<T>
where
	T::Balance: MaybeSerializeDeserialize + Debug,
{
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		Self::free_balance(who)
			.checked_sub(&value)
			.map_or(false, |new_balance| {
				Self::ensure_can_withdraw(who, value, WithdrawReason::Reserve.into(), new_balance).is_ok()
			})
	}

	fn slash_reserved(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		let b = Self::reserved_balance(who);
		let slash = cmp::min(b, value);
		// underflow should never happen, but it if does, there's nothing to be done here.
		Self::set_reserved_balance(who, b - slash);
		(NegativeImbalance::new(slash), value - slash)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		<ReservedBalance<T>>::get(who)
	}

	fn reserve(who: &T::AccountId, value: Self::Balance) -> Result<(), DispatchError> {
		let b = Self::free_balance(who);
		if b < value {
			Err(Error::<T>::InsufficientBalance)?
		}
		let new_balance = b - value;
		Self::ensure_can_withdraw(who, value, WithdrawReason::Reserve.into(), new_balance)?;
		Self::set_reserved_balance(who, Self::reserved_balance(who) + value);
		Self::set_free_balance(who, new_balance);
		Ok(())
	}

	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		let b = Self::reserved_balance(who);
		let actual = cmp::min(b, value);
		Self::set_free_balance(who, Self::free_balance(who) + actual);
		Self::set_reserved_balance(who, b - actual);
		value - actual
	}

	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		if Self::total_balance(beneficiary).is_zero() {
			Err(Error::<T>::DeadAccount)?
		}
		let b = Self::reserved_balance(slashed);
		let slash = cmp::min(b, value);
		Self::set_free_balance(beneficiary, Self::free_balance(beneficiary) + slash);
		Self::set_reserved_balance(slashed, b - slash);
		Ok(value - slash)
	}
}

impl<T: Trait> LockableCurrency<T::AccountId> for Module<T>
where
	T::Balance: MaybeSerializeDeserialize + Debug,
{
	type Moment = MomentOf<T>;

	fn set_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		withdraw_lock: WithdrawLock<Self::Balance, Self::Moment>,
		reasons: WithdrawReasons,
	) {
		let mut new_lock = Some(BalanceLock {
			id,
			withdraw_lock,
			reasons,
		});
		let mut locks = Self::locks(who)
			.into_iter()
			.filter_map(|l| if l.id == id { new_lock.take() } else { Some(l) })
			.collect::<Vec<_>>();
		if let Some(lock) = new_lock {
			locks.push(lock)
		}
		<Locks<T>>::insert(who, locks);
	}

	fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
		let locks = Self::locks(who)
			.into_iter()
			.filter_map(|l| if l.id != id { Some(l) } else { None })
			.collect::<Vec<_>>();
		<Locks<T>>::insert(who, locks);
	}
}

impl<T: Trait> VestingCurrency<T::AccountId> for Module<T>
where
	T::Balance: MaybeSerializeDeserialize + Debug,
{
	type Moment = T::BlockNumber;

	/// Get the amount that is currently being vested and cannot be transferred out of this account.
	fn vesting_balance(who: &T::AccountId) -> T::Balance {
		if let Some(v) = Self::vesting(who) {
			Self::free_balance(who).min(v.locked_at(<frame_system::Module<T>>::block_number()))
		} else {
			Zero::zero()
		}
	}

	/// Adds a vesting schedule to a given account.
	///
	/// If there already exists a vesting schedule for the given account, an `Err` is returned
	/// and nothing is updated.
	fn add_vesting_schedule(
		who: &T::AccountId,
		locked: T::Balance,
		per_block: T::Balance,
		starting_block: T::BlockNumber,
	) -> DispatchResult {
		if <Vesting<T>>::exists(who) {
			Err(Error::<T>::ExistingVestingSchedule)?
		}
		let vesting_schedule = VestingSchedule {
			locked,
			per_block,
			starting_block,
		};
		<Vesting<T>>::insert(who, vesting_schedule);
		Ok(())
	}

	/// Remove a vesting schedule for a given account.
	fn remove_vesting_schedule(who: &T::AccountId) {
		<Vesting<T>>::remove(who);
	}
}

impl<T: Trait> IsDeadAccount<T::AccountId> for Module<T>
where
	T::Balance: MaybeSerializeDeserialize + Debug,
{
	fn is_dead_account(who: &T::AccountId) -> bool {
		T::RingCurrency::total_balance(who).is_zero()
	}
}
