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
	let _ = migration::clear_storage_prefix(b"Council", b"Proposals", &[], None);
	let _ = migration::clear_storage_prefix(b"Council", b"ProposalOf", &[], None);
	let _ = migration::clear_storage_prefix(b"Council", b"Voting", &[], None);
	let _ = migration::clear_storage_prefix(b"Council", b"ProposalCount", &[], None);
	let _ = migration::clear_storage_prefix(b"Council", b"Members", &[], None);
	let _ = migration::clear_storage_prefix(b"Council", b"Prime", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"PublicPropCount", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"PublicProps", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"DepositOf", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"ReferendumCount", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"LowestUnbaked", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"ReferendumInfoOf", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"VotingOf", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"LastTabledWasExternal", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"NextExternal", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"Blacklist", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"Cancellations", &[], None);
	let _ = migration::clear_storage_prefix(b"Democracy", b"MetadataOf", &[], None);

	// frame_support::weights::Weight::zero()
	RuntimeBlockWeights::get().max_block
	// <Runtime as frame_system::Config>::DbWeight::get().reads_writes(5, 5)
}
