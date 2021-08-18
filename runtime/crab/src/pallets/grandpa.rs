pub use pallet_grandpa::AuthorityId as GrandpaId;

// --- paritytech ---
use frame_support::traits::KeyOwnerProofSystem;
use pallet_grandpa::{Config, EquivocationHandler};
use sp_core::crypto::KeyTypeId;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type Call = Call;
	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;
	type KeyOwnerProofSystem = Historical;
	type HandleEquivocation =
		EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;
	type WeightInfo = ();
}
