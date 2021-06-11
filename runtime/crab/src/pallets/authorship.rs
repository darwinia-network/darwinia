// --- substrate ---
use pallet_authorship::Config;
use pallet_session::FindAccountFromAuthorIndex;
// --- darwinia ---
use crate::*;

frame_support::parameter_types! {
	pub const UncleGenerations: BlockNumber = 0;
}

// TODO: substrate#2986 implement this properly
impl Config for Runtime {
	type FindAuthor = FindAccountFromAuthorIndex<Self, Babe>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = (Staking, ImOnline);
}
