// --- paritytech ---
use pallet_session::{historical::NoteHistoricalRoot, Config};
use sp_runtime::{traits::OpaqueKeys, Perbill};
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

frame_support::parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl Config for Runtime {
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
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
