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

// darwinia
#[allow(unused_imports)]
use crate::*;
// polkadot-sdk
#[allow(unused_imports)]
use frame_support::migration;

pub struct CustomOnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		log::info!("pre");

		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		log::info!("post");

		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrate()
	}
}

fn migrate() -> frame_support::weights::Weight {
	use codec::Decode;
	use frame_support::weights::Weight;

	#[derive(Decode, Eq, Ord, PartialEq, PartialOrd)]
	enum OldAssetType {
		Xcm(xcm::v3::Location),
	}

	let supported_assets =
		if let Some(supported_assets) = frame_support::storage::migration::get_storage_value::<
			Vec<OldAssetType>,
		>(b"AssetManager", b"SupportedFeePaymentAssets", &[])
		{
			sp_std::collections::btree_set::BTreeSet::from_iter(
				supported_assets.into_iter().map(|OldAssetType::Xcm(location_v3)| location_v3),
			)
		} else {
			return Weight::default();
		};

	let mut assets: Vec<(xcm::v4::Location, (bool, u128))> = Vec::new();

	for (OldAssetType::Xcm(location_v3), units_per_seconds) in
		frame_support::storage::migration::storage_key_iter::<
			OldAssetType,
			u128,
			frame_support::Blake2_128Concat,
		>(b"AssetManager", b"AssetTypeUnitsPerSecond")
	{
		let enabled = supported_assets.contains(&location_v3);

		if let Ok(location_v4) = location_v3.try_into() {
			assets.push((location_v4, (enabled, units_per_seconds)));
		}
	}

	//***** Start mutate storage *****//

	// Write asset metadata in new pallet_xcm_weight_trader
	use frame_support::weights::WeightToFee as _;
	for (asset_location, (enabled, units_per_second)) in assets {
		let native_amount_per_second: u128 =
			<Runtime as pallet_transaction_payment::Config>::WeightToFee::weight_to_fee(
				&Weight::from_parts(
					frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND,
					0,
				),
			);
		let relative_price: u128 = native_amount_per_second
			.saturating_mul(10u128.pow(pallet_xcm_weight_trader::RELATIVE_PRICE_DECIMALS))
			.saturating_div(units_per_second);
		pallet_xcm_weight_trader::SupportedAssets::<Runtime>::insert(
			asset_location,
			(enabled, relative_price),
		);
	}

	// Remove storage value AssetManager::SupportedFeePaymentAssets
	frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
		b"AssetManager",
		b"SupportedFeePaymentAssets",
	));

	// Remove storage map AssetManager::AssetTypeUnitsPerSecond
	let _ = frame_support::storage::migration::clear_storage_prefix(
		b"AssetManager",
		b"AssetTypeUnitsPerSecond",
		&[],
		None,
		None,
	);

	use array_bytes::Dehexify;

	for (who, count) in [
		("0x43269b2cf781E9a64Df38A2Fd849eEAd690852F0", 1),
		("0x6dDf9E3168Ff67F1C0416879390D7e6557b87b66", 2),
		("0x3e25247CfF03F99a7D83b28F207112234feE73a6", 1),
		("0xB2960E11B253c107f973CD778bBe1520E35E8602", 1),
		("0xe59261f6D4088BcD69985A3D369Ff14cC54EF1E5", 1),
		("0x1a469e3E616CBe7A7C40eC6b3E097aaDc2905A0A", 1),
	] {
		let Ok(who) = <[u8; 20]>::dehexify(who) else {
			continue;
		};
		let who = AccountId::from(who);

		<frame_system::Account<Runtime>>::mutate(
			&who,
			|account: &mut frame_system::AccountInfo<
				Nonce,
				pallet_balances::AccountData<Balance>,
			>| {
				account.data.reserved = account.data.reserved.saturating_sub(5_000 * UNIT * count);
			},
		);
	}

	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(30, 40)
}
