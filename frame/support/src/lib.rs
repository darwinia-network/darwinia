#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod structs;
pub mod traits;

pub mod balance {
	pub mod lock {
		pub use structs::{BalanceLock, LockFor, LockReasons, StakingLock, Unbonding};
		pub use traits::{LockIdentifier, LockableCurrency, VestingSchedule, WithdrawReason, WithdrawReasons};

		use crate::*;
	}

	pub use structs::{AccountData, FrozenBalance};
	pub use traits::ExistentialCheck;

	use crate::*;
}
