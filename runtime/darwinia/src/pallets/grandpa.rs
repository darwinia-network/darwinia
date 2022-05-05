pub use pallet_grandpa::AuthorityId as GrandpaId;

// --- paritytech ---
use frame_support::traits::KeyOwnerProofSystem;
use pallet_grandpa::{Config, EquivocationHandler};
use sp_core::crypto::KeyTypeId;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type Call = Call;
	type Event = Event;
	type HandleEquivocation =
		EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;
	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;
	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
	type KeyOwnerProofSystem = Historical;
	type MaxAuthorities = MaxAuthorities;
	type WeightInfo = ();
}
