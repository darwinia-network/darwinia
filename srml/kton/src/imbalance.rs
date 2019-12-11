// TODO: move this into lib.rs to be a inner mod

use rstd::mem;

use crate::{result, Imbalance, Saturating, StorageValue, Trait, Zero};

pub struct PositiveImbalance<T: Trait>(T::Balance);

impl<T: Trait> PositiveImbalance<T> {
	pub fn new(amount: T::Balance) -> Self {
		PositiveImbalance(amount)
	}
}

pub struct NegativeImbalance<T: Trait>(T::Balance);

impl<T: Trait> NegativeImbalance<T> {
	pub fn new(amount: T::Balance) -> Self {
		NegativeImbalance(amount)
	}
}

impl<T: Trait> Imbalance<T::Balance> for PositiveImbalance<T> {
	type Opposite = NegativeImbalance<T>;
	fn zero() -> Self {
		Self(Zero::zero())
	}

	fn drop_zero(self) -> result::Result<(), Self> {
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
	fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
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

impl<T: Trait> Imbalance<T::Balance> for NegativeImbalance<T> {
	type Opposite = PositiveImbalance<T>;

	fn zero() -> Self {
		Self(Zero::zero())
	}
	fn drop_zero(self) -> result::Result<(), Self> {
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
	fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
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
