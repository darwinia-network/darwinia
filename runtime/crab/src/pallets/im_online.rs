// --- substrate ---
use pallet_im_online::{sr25519::AuthorityId, Config};
use sp_runtime::transaction_validity::TransactionPriority;
// --- darwinia ---
use crate::{weights::pallet_im_online::WeightInfo, *};

frame_support::parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
}

impl Config for Runtime {
	type AuthorityId = AuthorityId;
	type Event = Event;
	type NextSessionRotation = Babe;
	type ValidatorSet = Historical;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = WeightInfo<Runtime>;
}
