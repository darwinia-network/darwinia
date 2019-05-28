
//TODO: move this into lib.rs to be a inner mod
use super::{result, Imbalance, Trait, Zero, Saturating, StorageValue, Subtrait};
use rstd::mem;


pub struct PositiveImbalance<T: Subtrait>(T::Balance);

impl<T: Subtrait> PositiveImbalance<T> {
    pub fn new(amount: T::Balance) -> Self {
        PositiveImbalance(amount)
    }
}

pub struct NegativeImbalance<T: Subtrait>(T::Balance);

impl<T: Subtrait> NegativeImbalance<T> {
    pub fn new(amount: T::Balance) -> Self {
        NegativeImbalance(amount)
    }
}

impl<T: Subtrait> Imbalance<T::Balance> for PositiveImbalance<T> {
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
        self.0.clone()
    }
}

impl<T: Subtrait> Imbalance<T::Balance> for NegativeImbalance<T> {
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
        self.0.clone()
    }
}

