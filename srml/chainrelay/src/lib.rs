//!  prototype module for bridging in ethereum poa blockcahin

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;
use support::{decl_event, decl_module, decl_storage, ensure};

use poa::BestHeader;

pub trait Trait: system::Trait {}

decl_storage! {
	trait Store for Module<T: Trait> as Bridge {
		// we don't need to start from genesis block
		pub InitialBlock get(initial_block) config(): T::BlockNumber;
		// BestHeader
		pub BestHeader get(best_header): BestHeader;

	}
}
