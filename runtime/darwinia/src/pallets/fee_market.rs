pub use pallet_fee_market::Instance1 as WithCrabFeeMarket;

// --- core ---
use core::cmp;
// --- paritytech ---
use frame_support::{traits::LockIdentifier, PalletId};
use sp_runtime::{traits::UniqueSaturatedInto, Permill};
// --- darwinia-network ---
use crate::*;
use pallet_fee_market::{Config, RingBalance, Slasher};

/// Slash 2 COINs for every delayed delivery each block.
pub struct FeeMarketSlasher;
impl<T, I> Slasher<T, I> for FeeMarketSlasher
where
	T: Config<I>,
	I: 'static,
{
	fn slash(locked_collateral: RingBalance<T, I>, timeout: T::BlockNumber) -> RingBalance<T, I> {
		let slash_each_block = 2 * COIN;
		let slash_value = UniqueSaturatedInto::<Balance>::unique_saturated_into(timeout)
			.saturating_mul(UniqueSaturatedInto::<Balance>::unique_saturated_into(slash_each_block))
			.unique_saturated_into();

		cmp::min(locked_collateral, slash_value)
	}
}

frame_support::parameter_types! {
	pub const FeeMarketPalletId: PalletId = PalletId(*b"da/feemk");
	pub const TreasuryPalletId: PalletId = PalletId(*b"da/trsry");
	pub const FeeMarketLockId: LockIdentifier = *b"da/feelf";

	pub const MinimumRelayFee: Balance = 15 * COIN;
	pub const CollateralPerOrder: Balance = 50 * COIN;
	pub const Slot: BlockNumber = 600;

	pub const AssignedRelayersRewardRatio: Permill = Permill::from_percent(60);
	pub const MessageRelayersRewardRatio: Permill = Permill::from_percent(80);
	pub const ConfirmRelayersRewardRatio: Permill = Permill::from_percent(20);
}

impl Config<WithCrabFeeMarket> for Runtime {
	type AssignedRelayersRewardRatio = AssignedRelayersRewardRatio;
	type CollateralPerOrder = CollateralPerOrder;
	type CollateralPerOrder = CollateralPerOrder;
	type ConfirmRelayersRewardRatio = ConfirmRelayersRewardRatio;
	type Event = Event;
	type LockId = FeeMarketLockId;
	type LockId = FeeMarketLockId;
	type MessageRelayersRewardRatio = MessageRelayersRewardRatio;
	type MinimumRelayFee = MinimumRelayFee;
	type MinimumRelayFee = MinimumRelayFee;
	type PalletId = FeeMarketPalletId;
	type PalletId = FeeMarketPalletId;
	type RingCurrency = Ring;
	type Slasher = FeeMarketSlasher;
	type Slot = Slot;
	type Slot = Slot;
	type TreasuryPalletId = TreasuryPalletId;
	type TreasuryPalletId = TreasuryPalletId;
	type WeightInfo = ();
}
