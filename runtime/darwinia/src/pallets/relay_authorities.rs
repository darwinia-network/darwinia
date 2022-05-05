pub use darwinia_relay_authorities::Instance1 as EthereumRelayAuthoritiesInstance;

// --- paritytech ---
use frame_support::traits::LockIdentifier;
use sp_runtime::Perbill;
// --- darwinia-network ---
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
	type AddOrigin = ApproveOrigin;
	type DarwiniaMMR = DarwiniaHeaderMMR;
	type Event = Event;
	type LockId = EthereumRelayAuthoritiesLockId;
	type MaxCandidates = MaxCandidates;
	type OpCodes = OpCodes;
	type RemoveOrigin = ApproveOrigin;
	type ResetOrigin = ApproveOrigin;
	type RingCurrency = Ring;
	type Sign = EthereumBacking;
	type SignThreshold = SignThreshold;
	type SubmitDuration = SubmitDuration;
	type TermDuration = EthereumRelayAuthoritiesTermDuration;
	type WeightInfo = ();
}
