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
	// substrate
	use pallet_balances::Locks;
	use sp_core::H160;
	use sp_std::str::FromStr;

	[
		("0xabcf7060a68f62624f7569ada9d78b5a5db0782a", b"phrelect"),
		("0x88a39b052d477cfde47600a7c9950a441ce61cb4", b"phrelect"),
		("0x9f33a4809aa708d7a399fedba514e0a0d15efa85", b"phrelect"),
		("0x0a1287977578f888bdc1c7627781af1cc000e6ab", b"phrelect"),
		("0xe59261f6d4088bcd69985a3d369ff14cc54ef1e5", b"phrelect"),
		("0x7ae2a0914db8bfbdad538b0eac3fa473a0e07843", b"phrelect"),
		("0x3e25247cff03f99a7d83b28f207112234fee73a6", b"phrelect"),
		("0xb2960e11b253c107f973cd778bbe1520e35e8602", b"phrelect"),
	]
	.iter()
	.for_each(|(acct, lid)| {
		if let Ok(acct) = array_bytes::hex_n_into::<_, AccountId, 20>(acct) {
			<Locks<Runtime>>::mutate(acct, |ls| {
				ls.retain(|l| &l.id != *lid);
			});
		}
	});

	const REVERT_BYTECODE: [u8; 5] = [0x60, 0x00, 0x60, 0x00, 0xFD];
	// PINK equals to the 0x404 in the pallet-evm runtime.
	const ADDRESS: &str = "0x0000000000000000000000000000000000000404";
	if let Ok(addr) = H160::from_str(ADDRESS) {
		EVM::create_account(addr, REVERT_BYTECODE.to_vec());
	}

	// frame_support::weights::Weight::zero()
	RuntimeBlockWeights::get().max_block
}
