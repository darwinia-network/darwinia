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
// substrate
#[allow(unused_imports)]
use frame_support::{migration, storage::unhashed};

pub struct CustomOnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrate()
	}
}

fn migrate() -> frame_support::weights::Weight {
	let mut w = 0;

	w += migration_helper::PalletCleaner {
		name: b"BridgeMoonbaseGrandpa",
		values: &[
			b"RequestCount",
			b"InitialHash",
			b"BestFinalized",
			b"ImportedHashesPointer",
			b"CurrentAuthoritySet",
			b"PalletOwner",
			b"PalletOperatingMode",
		],
		maps: &[b"ImportedHashes", b"ImportedHeaders"],
	}
	.remove_all();
	w += migration_helper::PalletCleaner {
		name: b"BridgeMoonbaseParachain",
		values: &[b"PalletOwner", b"PalletOperatingMode"],
		maps: &[b"ParasInfo", b"ImportedParaHeads", b"ImportedParaHashes"],
	}
	.remove_all();
	w += migration_helper::PalletCleaner {
		name: b"BridgePangoroMessages",
		values: &[b"PalletOwner", b"PalletOperatingMode"],
		maps: &[b"InboundLanes", b"OutboundLanes", b"OutboundMessages"],
	}
	.remove_all();
	w += migration_helper::PalletCleaner {
		name: b"PangoroFeeMarket",
		values: &[
			b"Relayers",
			b"AssignedRelayers",
			b"CollateralSlashProtect",
			b"AssignedRelayersNumber",
		],
		maps: &[b"Orders", b"RelayersMap"],
	}
	.remove_all();

	// frame_support::weights::Weight::zero()
	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(0, w as _)
}
