// --- substrate ---
use frame_support::traits::KeyOwnerProofSystem;
use pallet_babe::{AuthorityId, Config, EquivocationHandler, ExternalTrigger};
use sp_core::crypto::KeyTypeId;
// --- darwinia ---
use crate::*;

frame_support::parameter_types! {
	pub const EpochDuration: u64 = BLOCKS_PER_SESSION as _;
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
	pub const ReportLongevity: u64 =
		BondingDurationInEra::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

impl Config for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	// session module is the trigger
	type EpochChangeTrigger = ExternalTrigger;
	type KeyOwnerProofSystem = Historical;
	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, AuthorityId)>>::Proof;
	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		AuthorityId,
	)>>::IdentificationTuple;
	type HandleEquivocation =
		EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;
	type WeightInfo = ();
}
