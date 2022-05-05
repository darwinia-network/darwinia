// --- paritytech ---
use pallet_im_online::{sr25519::AuthorityId, Config};
use sp_runtime::transaction_validity::TransactionPriority;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const MaxPeerDataEncodingSize: u32 = 1_000;
}

impl Config for Runtime {
	type AuthorityId = AuthorityId;
	type Event = Event;
	type MaxKeys = MaxKeys;
	type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
	type NextSessionRotation = Babe;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type ValidatorSet = Historical;
	type WeightInfo = ();
}
