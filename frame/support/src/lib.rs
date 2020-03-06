#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod structs;
pub mod traits;

pub mod balance {
	pub mod lock {
		pub use frame_support::traits::{LockIdentifier, WithdrawReason, WithdrawReasons};

		pub use structs::{BalanceLock, LockFor, LockReasons, StakingLock, Unbonding};
		pub use traits::LockableCurrency;

		use crate::*;
	}

	pub use traits::ExistentialCheck;

	use crate::*;
}
