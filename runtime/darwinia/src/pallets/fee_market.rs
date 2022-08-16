pub use pallet_fee_market::Instance1 as WithCrabFeeMarket;

// --- core ---
use core::cmp;
// --- paritytech ---
use frame_support::traits::LockIdentifier;
use sp_runtime::{Permill, SaturatedConversion};
// --- darwinia-network ---
use crate::*;
use pallet_fee_market::{BalanceOf, Config, Slasher};

/// Slash 2 COINs for every delayed delivery each block.
pub struct FeeMarketSlasher;
impl<T, I> Slasher<T, I> for FeeMarketSlasher
where
	T: Config<I>,
	I: 'static,
	{
		fn cal_slash_amount(
			collateral_per_order: BalanceOf<T, I>,
			timeout: T::BlockNumber,
		) -> BalanceOf<T, I> {
			const SLASH_PER_BLOCK: Balance = 2 * COIN;

			let collateral_per_order = collateral_per_order.saturated_into::<Balance>();
			let timeout = timeout.saturated_into::<Balance>();
			let slash_value = timeout.saturating_mul(SLASH_PER_BLOCK);

			cmp::min(collateral_per_order, slash_value).saturated_into()
		}
	}

frame_support::parameter_types! {
	pub const MinimumRelayFee: Balance = 15 * COIN;
	pub const CollateralPerOrder: Balance = 50 * COIN;
	pub const Slot: BlockNumber = 300;

	pub const GuardRelayersRewardRatio: Permill = Permill::from_percent(20);
	pub const MessageRelayersRewardRatio: Permill = Permill::from_percent(80);
	pub const ConfirmRelayersRewardRatio: Permill = Permill::from_percent(20);
	pub const AssignedRelayerSlashRatio: Permill = Permill::from_percent(20);

	pub const FeeMarketLockId: LockIdentifier = *b"da/feelf";
}

impl Config<WithCrabFeeMarket> for Runtime {
	type AssignedRelayerSlashRatio = AssignedRelayerSlashRatio;
	type CollateralPerOrder = CollateralPerOrder;
	type ConfirmRelayersRewardRatio = ConfirmRelayersRewardRatio;
	type Currency = Ring;
	type Event = Event;
	type GuardRelayersRewardRatio = GuardRelayersRewardRatio;
	type LockId = FeeMarketLockId;
	type MessageRelayersRewardRatio = MessageRelayersRewardRatio;
	type MinimumRelayFee = MinimumRelayFee;
	type Slasher = FeeMarketSlasher;
	type Slot = Slot;
	type TreasuryPalletId = TreasuryPalletId;
	type WeightInfo = ();
}
