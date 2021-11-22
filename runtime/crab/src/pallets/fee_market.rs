// --- core ---
use core::cmp;
// --- substrate ---
use frame_support::{traits::LockIdentifier, PalletId};
use sp_runtime::{traits::UniqueSaturatedInto, Permill};
// --- darwinia ---
use crate::*;
use darwinia_fee_market::{Config, RingBalance, Slasher};

frame_support::parameter_types! {
	pub const FeeMarketPalletId: PalletId = PalletId(*b"da/feemk");
	pub const TreasuryPalletId: PalletId = PalletId(*b"da/trsry");
	pub const FeeMarketLockId: LockIdentifier = *b"da/feelf";

	pub const AssignedRelayersNumber: u64 = 3;
	pub const MinimumRelayFee: Balance = 15 * COIN;
	pub const CollateralPerOrder: Balance = 100 * COIN;
	pub const Slot: BlockNumber = 600;

	pub const AssignedRelayersRewardRatio: Permill = Permill::from_percent(60);
	pub const MessageRelayersRewardRatio: Permill = Permill::from_percent(80);
	pub const ConfirmRelayersRewardRatio: Permill = Permill::from_percent(20);
}

impl Config for Runtime {
	type PalletId = FeeMarketPalletId;
	type TreasuryPalletId = TreasuryPalletId;
	type LockId = FeeMarketLockId;

	type MinimumRelayFee = MinimumRelayFee;
	type CollateralPerOrder = CollateralPerOrder;
	type Slot = Slot;

	type AssignedRelayersNumber = AssignedRelayersNumber;
	type AssignedRelayersRewardRatio = AssignedRelayersRewardRatio;
	type MessageRelayersRewardRatio = MessageRelayersRewardRatio;
	type ConfirmRelayersRewardRatio = ConfirmRelayersRewardRatio;

	type Slasher = FeeMarketSlasher;
	type RingCurrency = Ring;
	type Event = Event;
	type WeightInfo = ();
}

pub struct FeeMarketSlasher;
impl<T: Config> Slasher<T> for FeeMarketSlasher {
	fn slash(locked_collateral: RingBalance<T>, timeout: T::BlockNumber) -> RingBalance<T> {
		let slash_each_block = 2 * COIN;
		let slash_value = UniqueSaturatedInto::<Balance>::unique_saturated_into(timeout)
			.saturating_mul(UniqueSaturatedInto::<Balance>::unique_saturated_into(
				slash_each_block,
			))
			.unique_saturated_into();

		cmp::min(locked_collateral, slash_value)
	}
}
