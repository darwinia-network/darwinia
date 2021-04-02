#[cfg(feature = "std")]
pub use darwinia_claims::ClaimsList;

// --- substrate ---
use sp_runtime::ModuleId;
// --- darwinia ---
use crate::*;
use darwinia_claims::Config;

frame_support::parameter_types! {
	pub const ClaimsModuleId: ModuleId = ModuleId(*b"da/claim");
	pub Prefix: &'static [u8] = b"Pay RINGs to the Crab account:";
}
impl Config for Runtime {
	type Event = Event;
	type ModuleId = ClaimsModuleId;
	type Prefix = Prefix;
	type RingCurrency = Ring;
	type MoveClaimOrigin = EnsureRootOrMoreThanHalfCouncil;
}
