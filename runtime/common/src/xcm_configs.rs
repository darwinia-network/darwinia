// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

// core
use core::marker::PhantomData;
// crates.io
use codec::Encode;
// darwinia
use dc_primitives::GWEI;
// polkadot-sdk
use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::currency::Currency as CurrencyT, ConstU128, Contains, OnUnbalanced as OnUnbalancedT,
	},
	weights::{Weight, WeightToFee as WeightToFeeT},
};
use sp_core::Get;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::{SaturatedConversion, Saturating, Zero};
use sp_std::{prelude::*, result::Result};
use xcm::latest::{prelude::*, Weight as XcmWeight};
use xcm_builder::TakeRevenue;
use xcm_executor::traits::{ConvertLocation, WeightTrader};

/// Base balance required for the XCM unit weight.
pub type XcmBaseWeightFee = ConstU128<GWEI>;

frame_support::match_types! {
	pub type ParentOrParentsPlurality: impl Contains<Location> = {
		Location { parents: 1, interior: Here } |
		Location { parents: 1, interior: Junctions::X1(Plurality { id: BodyId::Administration, .. }) }|
		Location { parents: 1, interior: Junctions::X1(Plurality { id: BodyId::Executive, .. }) }|
		Location { parents: 1, interior: Junctions::X1(Plurality { id: BodyId::Technical, .. }) }
	};
}

/// Struct that converts a given Location into a 20 bytes account id by hashing
/// with blake2_256 and taking the first 20 bytes
pub struct Account20Hash<AccountId>(PhantomData<AccountId>);
impl<AccountId: From<[u8; 20]> + Into<[u8; 20]> + Clone> ConvertLocation<AccountId>
	for Account20Hash<AccountId>
{
	fn convert_location(location: &Location) -> Option<AccountId> {
		let hash: [u8; 32] = ("multiloc", location).using_encoded(blake2_256);
		let mut account_id = [0u8; 20];

		account_id.copy_from_slice(&hash[0..20]);

		Some(account_id.into())
	}
}

/// Weight trader to set the right price for weight and then places any weight bought into the right
/// account. Refer to: https://github.com/paritytech/polkadot/blob/release-v0.9.30/xcm/xcm-builder/src/weight.rs#L242-L305
pub struct LocalAssetTrader<
	WeightToFee: WeightToFeeT<Balance = Currency::Balance>,
	AssetId: Get<Location>,
	AccountId,
	Currency: CurrencyT<AccountId>,
	OnUnbalanced: OnUnbalancedT<Currency::NegativeImbalance>,
	R: TakeRevenue,
>(
	Weight,
	Currency::Balance,
	PhantomData<(WeightToFee, AssetId, AccountId, Currency, OnUnbalanced, R)>,
);
impl<
		WeightToFee: WeightToFeeT<Balance = Currency::Balance>,
		AssetId: Get<Location>,
		AccountId,
		Currency: CurrencyT<AccountId>,
		OnUnbalanced: OnUnbalancedT<Currency::NegativeImbalance>,
		R: TakeRevenue,
	> WeightTrader for LocalAssetTrader<WeightToFee, AssetId, AccountId, Currency, OnUnbalanced, R>
{
	fn new() -> Self {
		Self(Weight::zero(), Zero::zero(), PhantomData)
	}

	fn buy_weight(
		&mut self,
		weight: XcmWeight,
		payment: Assets,
		_context: &XcmContext,
	) -> Result<Assets, XcmError> {
		log::trace!(target: "xcm::weight", "LocalAssetTrader::buy_weight weight: {:?}, payment:
		{:?}", weight, payment);
		let amount = WeightToFee::weight_to_fee(&weight);
		let u128_amount: u128 = amount.try_into().map_err(|_| XcmError::Overflow)?;
		let required: Assets = (Concrete(AssetId::get()), u128_amount).into();
		let unused = payment.checked_sub(required.clone()).map_err(|_| XcmError::TooExpensive)?;
		self.0 = self.0.saturating_add(weight);
		self.1 = self.1.saturating_add(amount);
		R::take_revenue(required);
		Ok(unused)
	}

	fn refund_weight(&mut self, weight: XcmWeight, _context: &XcmContext) -> Option<MultiAsset> {
		log::trace!(target: "xcm::weight", "LocalAssetTrader::refund_weight weight: {:?}",
		weight);
		let weight = weight.min(self.0);
		let amount = WeightToFee::weight_to_fee(&weight);
		self.0 -= weight;
		self.1 = self.1.saturating_sub(amount);
		let amount: u128 = amount.saturated_into();
		if amount > 0 {
			Some((AssetId::get(), amount).into())
		} else {
			None
		}
	}
}
impl<
		WeightToFee: WeightToFeeT<Balance = Currency::Balance>,
		AssetId: Get<Location>,
		AccountId,
		Currency: CurrencyT<AccountId>,
		OnUnbalanced: OnUnbalancedT<Currency::NegativeImbalance>,
		R: TakeRevenue,
	> Drop for LocalAssetTrader<WeightToFee, AssetId, AccountId, Currency, OnUnbalanced, R>
{
	fn drop(&mut self) {
		OnUnbalanced::on_unbalanced(Currency::issue(self.1));
	}
}

/// Filter to check if a given location is the parent Relay Chain or a sibling parachain.
///
/// This type should only be used within the context of a parachain, since it does not verify that
/// the parent is indeed a Relay Chain.
pub struct ParentRelayOrSiblingParachains;
impl Contains<Location> for ParentRelayOrSiblingParachains {
	fn contains(location: &Location) -> bool {
		matches!(
			location,
			Location { parents: 1, interior: Here }
				| Location { parents: 1, interior: Junctions::X1(Parachain(_)) }
		)
	}
}

// TODO: move to other place.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub struct AssetRegistrarMetadata {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub is_frozen: bool,
}
