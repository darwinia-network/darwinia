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
	let items: &[(&[u8], &[&[u8]])] = &[
		(b"MessageGadget", &[b"CommitmentContract"]),
		(
			b"EcdsaAuthority",
			&[
				b"Authorities",
				b"NextAuthorities",
				b"Nonce",
				b"AuthoritiesChangeToSign",
				b"MessageRootToSign",
			],
		),
		(
			b"Council",
			&[b"Proposals", b"ProposalOf", b"Voting", b"ProposalCount", b"Members", b"Prime"],
		),
		(
			b"Democracy",
			&[
				b"PublicPropCount",
				b"PublicProps",
				b"DepositOf",
				b"ReferendumCount",
				b"LowestUnbaked",
				b"ReferendumInfoOf",
				b"VotingOf",
				b"LastTabledWasExternal",
				b"NextExternal",
				b"Blacklist",
				b"Cancellations",
				b"MetadataOf",
			],
		),
	];

	let w = items.iter().fold(0, |w, (p, is)| {
		w + is.iter().fold(0, |w, i| {
			w + migration::clear_storage_prefix(p, i, &[], None, None).backend as u64
		})
	});

	// frame_support::weights::Weight::zero()
	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(0, w)
}
