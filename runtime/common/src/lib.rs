// This file is part of Darwinia.
//
// Copyright (C) 2018-2023 Darwinia Network
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

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unused_crate_dependencies)]
// TODO: FIX ME
// #![deny(missing_docs)]

pub mod evm;
pub mod gov_origin;
pub mod messages;
pub mod system;
pub mod xcm_configs;

pub use bp_darwinia_core as bp_crab;
pub use bp_darwinia_core as bp_darwinia;
// pub use bp_darwinia_core as bp_pangolin;
// pub use bp_darwinia_core as bp_pangoro;

#[cfg(feature = "test")]
pub mod test;

#[macro_export]
macro_rules! fast_runtime_or_not {
	($name:ident, $development_type:ty, $production_type:ty) => {
		#[cfg(feature = "fast-runtime")]
		type $name = $development_type;
		#[cfg(not(feature = "fast-runtime"))]
		type $name = $production_type;
	};
}

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl frame_support::weights::WeightToFeePolynomial for WeightToFee {
	type Balance = dc_primitives::Balance;

	fn polynomial() -> frame_support::weights::WeightToFeeCoefficients<Self::Balance> {
		let p = dc_primitives::UNIT;
		let q = 10
			* dc_primitives::Balance::from(
				frame_support::weights::constants::ExtrinsicBaseWeight::get().ref_time(),
			);

		smallvec::smallvec![frame_support::weights::WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: sp_runtime::Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> frame_support::traits::OnUnbalanced<pallet_balances::NegativeImbalance<R>>
	for DealWithFees<R>
where
	R: pallet_balances::Config,
	R: pallet_balances::Config + pallet_treasury::Config,
	pallet_treasury::Pallet<R>:
		frame_support::traits::OnUnbalanced<pallet_balances::NegativeImbalance<R>>,
{
	// this seems to be called for substrate-based transactions
	fn on_unbalanceds<B>(
		mut fees_then_tips: impl Iterator<Item = pallet_balances::NegativeImbalance<R>>,
	) {
		if let Some(fees) = fees_then_tips.next() {
			// substrate
			use frame_support::traits::Imbalance;

			// for fees, 80% are burned, 20% to the treasury
			let (_, to_treasury) = fees.ration(80, 20);

			// Balances pallet automatically burns dropped Negative Imbalances by decreasing
			// total_supply accordingly
			<pallet_treasury::Pallet<R> as frame_support::traits::OnUnbalanced<_>>::on_unbalanced(
				to_treasury,
			);
		}
	}

	// this is called from pallet_evm for Ethereum-based transactions
	// (technically, it calls on_unbalanced, which calls this when non-zero)
	fn on_nonzero_unbalanced(amount: pallet_balances::NegativeImbalance<R>) {
		// substrate
		use frame_support::traits::Imbalance;

		// Balances pallet automatically burns dropped Negative Imbalances by decreasing
		// total_supply accordingly
		let (_, to_treasury) = amount.ration(80, 20);

		<pallet_treasury::Pallet<R> as frame_support::traits::OnUnbalanced<_>>::on_unbalanced(
			to_treasury,
		);
	}
}

/// Deposit calculator for Darwinia.
/// 100 UNIT for the base fee, 102.4 UNIT/MB.
pub const fn darwinia_deposit(items: u32, bytes: u32) -> dc_primitives::Balance {
	// First try.
	items as dc_primitives::Balance * 100 * dc_types::UNIT
		+ (bytes as dc_primitives::Balance) * 100 * dc_types::MICROUNIT
	// items as Balance * 100 * UNIT + (bytes as Balance) * 100 * MILLIUNIT
}

/// Helper for pallet-assets benchmarking.
#[cfg(feature = "runtime-benchmarks")]
pub struct AssetsBenchmarkHelper;
#[cfg(feature = "runtime-benchmarks")]
impl pallet_assets::BenchmarkHelper<codec::Compact<u64>> for AssetsBenchmarkHelper {
	fn create_asset_id_parameter(id: u32) -> codec::Compact<u64> {
		u64::from(id).into()
	}
}
