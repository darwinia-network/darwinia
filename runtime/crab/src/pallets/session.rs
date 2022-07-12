// --- paritytech ---
use pallet_session::{historical::NoteHistoricalRoot, Config};
use sp_runtime::traits::OpaqueKeys;
use sp_std::prelude::*;
// --- darwinia-network ---
use crate::*;
use darwinia_staking::StashOf;

sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub babe: Babe,
		pub grandpa: Grandpa,
		pub im_online: ImOnline,
		pub authority_discovery: AuthorityDiscovery,
	}
}

impl Config for Runtime {
	type Event = Event;
	type Keys = SessionKeys;
	type NextSessionRotation = Babe;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type SessionManager = NoteHistoricalRoot<Self, Staking>;
	type ShouldEndSession = Babe;
	type ValidatorId = AccountId;
	type ValidatorIdOf = StashOf<Self>;
	type WeightInfo = ();
}
