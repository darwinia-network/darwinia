pub use darwinia_relay_authorities::Instance0 as EthereumRelayAuthoritiesInstance;

// --- substrate ---
use frame_support::traits::LockIdentifier;
use sp_runtime::Perbill;
// --- darwinia ---
use crate::*;
use darwinia_relay_authorities::Config;
use darwinia_relay_primitives::relay_authorities::OpCode;

frame_support::parameter_types! {
	pub const EthereumRelayAuthoritiesLockId: LockIdentifier = *b"ethrauth";
	pub const EthereumRelayAuthoritiesTermDuration: BlockNumber = 7 * DAYS;
	pub const MaxCandidates: usize = 7;
	pub const OpCodes: (OpCode, OpCode) = (
		[71, 159, 189, 249],
		[180, 188, 244, 151]
	);
	pub const SignThreshold: Perbill = Perbill::from_percent(60);
	pub const SubmitDuration: BlockNumber = 300;
}
impl Config<EthereumRelayAuthoritiesInstance> for Runtime {
	type Event = Event;
	type RingCurrency = Ring;
	type LockId = EthereumRelayAuthoritiesLockId;
	type TermDuration = EthereumRelayAuthoritiesTermDuration;
	type MaxCandidates = MaxCandidates;
	type AddOrigin = ApproveOrigin;
	type RemoveOrigin = ApproveOrigin;
	type ResetOrigin = ApproveOrigin;
	type DarwiniaMMR = HeaderMMR;
	type Sign = EthereumBacking;
	type OpCodes = OpCodes;
	type SignThreshold = SignThreshold;
	type SubmitDuration = SubmitDuration;
	type WeightInfo = ();
}
