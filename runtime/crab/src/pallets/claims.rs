#[cfg(feature = "std")]
pub use darwinia_claims::ClaimsList;

// --- substrate ---
use frame_support::PalletId;
// --- darwinia ---
use crate::*;
use darwinia_claims::Config;

frame_support::parameter_types! {
	pub const ClaimsPalletId: PalletId = PalletId(*b"da/claim");
	pub Prefix: &'static [u8] = b"Pay RINGs to the Crab account:";
}

impl Config for Runtime {
	type Event = Event;
	type PalletId = ClaimsPalletId;
	type Prefix = Prefix;
	type RingCurrency = Ring;
	type MoveClaimOrigin = EnsureRootOrMoreThanHalfCouncil;
}
