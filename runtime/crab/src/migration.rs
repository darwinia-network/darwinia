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
	let _ = migration::clear_storage_prefix(b"PhragmenElection", b"Members", &[], None, None);
	let _ = migration::clear_storage_prefix(b"PhragmenElection", b"RunnersUp", &[], None, None);
	let _ = migration::clear_storage_prefix(b"PhragmenElection", b"Candidates", &[], None, None);
	let _ =
		migration::clear_storage_prefix(b"PhragmenElection", b"ElectionRounds", &[], None, None);
	let _ = migration::clear_storage_prefix(b"PhragmenElection", b"Voting", &[], None, None);
	let _ = migration::clear_storage_prefix(b"TechnicalMembership", b"Members", &[], None, None);
	let _ = migration::clear_storage_prefix(b"TechnicalMembership", b"Prime", &[], None, None);

	const REVERT_BYTECODE: [u8; 5] = [0x60, 0x00, 0x60, 0x00, 0xFD];
	// CONVICTION_VOTING_ADDRESS equals to the addr(0x602) in the pallet-evm runtime.
	const CONVICTION_VOTING_ADDRESS: &str = "0x0000000000000000000000000000000000000602";
	if let Some(addr) = H160::from_str(CONVICTION_VOTING_ADDRESS) {
		EVM::create_account(addr, REVERT_BYTECODE.to_vec());
	}

	// frame_support::weights::Weight::zero()
	RuntimeBlockWeights::get().max_block
	// <Runtime as frame_system::Config>::DbWeight::get().reads_writes(0, 2)
}
