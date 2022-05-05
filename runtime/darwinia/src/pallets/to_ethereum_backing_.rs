// --- paritytech ---
use frame_support::PalletId;
// --- darwinia-network ---
use crate::*;
use to_ethereum_backing::Config;

frame_support::parameter_types! {
	pub const EthereumBackingPalletId: PalletId = PalletId(*b"da/ethbk");
	pub const EthereumBackingFeePalletId: PalletId = PalletId(*b"da/ethfe");
	pub const RingLockLimit: Balance = 10_000_000 * COIN;
	pub const KtonLockLimit: Balance = 1_000 * COIN;
	// https://github.com/darwinia-network/darwinia-common/pull/377#issuecomment-730369387
	pub const AdvancedFee: Balance = 50 * COIN;
	pub const SyncReward: Balance = 1_000 * COIN;
}

impl Config for Runtime {
	type AdvancedFee = AdvancedFee;
	type EcdsaAuthorities = EthereumRelayAuthorities;
	type EthereumRelay = EthereumRelay;
	type Event = Event;
	type FeePalletId = EthereumBackingFeePalletId;
	type KtonCurrency = Kton;
	type KtonLockLimit = KtonLockLimit;
	type OnDepositRedeem = Staking;
	type PalletId = EthereumBackingPalletId;
	type RedeemAccountId = AccountId;
	type RingCurrency = Ring;
	type RingLockLimit = RingLockLimit;
	type SyncReward = SyncReward;
	type WeightInfo = ();
}
