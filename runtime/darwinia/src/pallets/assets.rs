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
use crate::*;

/// List of the assets existed in this runtime.
pub enum AssetIds {
	Kton = 1026,
}

/// Keep Darwinia KTON immutable while its balances are migrated to Ethereum.
///
/// Asset precompiles dispatch these same calls, so this also covers ERC-20 writes.
/// Root and explicit `dispatch_bypass_filter` callers remain trusted governance paths.
pub struct KtonCallFilter;
impl frame_support::traits::Contains<RuntimeCall> for KtonCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		use pallet_assets::Call;

		let RuntimeCall::Assets(call) = call else {
			return true;
		};

		// Check every asset call, including thawing, team changes, refunds, and destruction.
		let id = match call {
			Call::create { id, .. }
			| Call::force_create { id, .. }
			| Call::start_destroy { id, .. }
			| Call::destroy_accounts { id, .. }
			| Call::destroy_approvals { id, .. }
			| Call::finish_destroy { id, .. }
			| Call::mint { id, .. }
			| Call::burn { id, .. }
			| Call::transfer { id, .. }
			| Call::transfer_keep_alive { id, .. }
			| Call::force_transfer { id, .. }
			| Call::freeze { id, .. }
			| Call::thaw { id, .. }
			| Call::freeze_asset { id, .. }
			| Call::thaw_asset { id, .. }
			| Call::transfer_ownership { id, .. }
			| Call::set_team { id, .. }
			| Call::set_metadata { id, .. }
			| Call::clear_metadata { id, .. }
			| Call::force_set_metadata { id, .. }
			| Call::force_clear_metadata { id, .. }
			| Call::force_asset_status { id, .. }
			| Call::approve_transfer { id, .. }
			| Call::cancel_approval { id, .. }
			| Call::force_cancel_approval { id, .. }
			| Call::transfer_approved { id, .. }
			| Call::touch { id, .. }
			| Call::refund { id, .. }
			| Call::set_min_balance { id, .. }
			| Call::touch_other { id, .. }
			| Call::refund_other { id, .. }
			| Call::block { id, .. }
			| Call::transfer_all { id, .. } => id.0,
			// New SDK asset calls require an explicit review of their asset identifier.
			_ => return false,
		};

		id != AssetIds::Kton as AssetId
	}
}

impl pallet_assets::Config for Runtime {
	type ApprovalDeposit = ();
	#[cfg(feature = "runtime-benchmarks")]
	type AssetAccountDeposit = ConstU128<1>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type AssetAccountDeposit = ();
	type AssetDeposit = ();
	type AssetId = AssetId;
	type AssetIdParameter = codec::Compact<AssetId>;
	type Balance = Balance;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = benchmark_helper::Assets;
	type CallbackHandle = ();
	type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<
		frame_system::EnsureSignedBy<pallet_config::AssetCreator, Self::AccountId>,
	>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = RootOr<GeneralAdmin>;
	type Freezer = ();
	type MetadataDepositBase = ();
	type MetadataDepositPerByte = ();
	type RemoveItemsLimit = ConstU32<1000>;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = ConstU32<50>;
	type WeightInfo = weights::pallet_assets::WeightInfo<Self>;
}
