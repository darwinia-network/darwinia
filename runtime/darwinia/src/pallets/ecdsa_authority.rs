// --- paritytech ---
use sp_runtime::Perbill;
// --- darwinia-network ---
use crate::*;
use darwinia_ecdsa_authority::Config;
use darwinia_message_gadget::MessageRootGetter;

frame_support::parameter_types! {
	pub const ChainId: &'static [u8] = b"46";
	pub const MaxEcdsaAuthorities: u32 = 7;
	pub const MaxPendingPeriod: BlockNumber = 100;
	pub const SignThreshold: Perbill = Perbill::from_percent(60);
	pub const SyncInterval: BlockNumber = 10;
}
static_assertions::const_assert!(SyncInterval::get() < MaxPendingPeriod::get());

impl Config for Runtime {
	type ChainId = ChainId;
	type Event = Event;
	type MaxAuthorities = MaxEcdsaAuthorities;
	type MaxPendingPeriod = MaxPendingPeriod;
	type MessageRoot = MessageRootGetter<Self>;
	type SignThreshold = SignThreshold;
	type SyncInterval = SyncInterval;
	type WeightInfo = ();
}
