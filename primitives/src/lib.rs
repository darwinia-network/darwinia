// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
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

//! Darwinia types shared between the runtime and the Node-side code.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

// --- paritytech ---
use sp_core::H256;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiSignature, OpaqueExtrinsic,
};

/// An index to a block.
/// 32-bits will allow for 136 years of blocks assuming 1 block per second.
pub type BlockNumber = u32;

/// An instant or duration in time.
pub type Moment = u64;

/// Alias to type for a signature for a transaction on the relay chain. This allows one of several
/// kinds of underlying crypto to be used, so isn't a fixed size when encoded.
pub type Signature = MultiSignature;

/// Alias to the public key used for this chain, actually a `MultiSigner`. Like the signature, this
/// also isn't a fixed size when encoded, as different cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;

/// Alias to the opaque account ID type for this chain, actually a `AccountId32`. This is always
/// 32 bytes.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// A hash of some data used by the relay chain.
pub type Hash = H256;

/// Hashing algorithm used by the chain.
pub type Hashing = BlakeTwo256;

/// Index of a transaction in the relay chain. 32-bit should be plenty.
pub type Nonce = u32;

/// The balance of an account.
/// 128-bits (or 38 significant decimal figures) will allow for 10m currency (10^7) at a resolution
/// to all for one second's worth of an annualised 50% reward be paid to a unit holder (10^11 unit
/// denomination), or 10^18 total atomic units, to grow at 50%/year for 51 years (10^9 multiplier)
/// for an eventual total of 10^27 units (27 significant decimal figures).
/// We round denomination to 10^12 (12 sdf), and leave the other redundancy at the upper end so
/// that 32 bits may be multiplied with a balance in 128 bits without worrying about overflow.
pub type Balance = u128;

/// The power of an account.
pub type Power = u32;

/// Header type.
pub type Header = generic::Header<BlockNumber, Hashing>;

/// Block type.
pub type OpaqueBlock = generic::Block<Header, OpaqueExtrinsic>;

#[ignore]
#[test]
fn print_module_account() {
	// --- paritytech ---
	use frame_support::PalletId;
	use sp_core::crypto::{set_default_ss58_version, Ss58AddressFormat, Ss58AddressFormat::*};
	use sp_runtime::traits::AccountIdConversion;

	fn account_of(alias: [u8; 8], ss58_version: Ss58AddressFormat) {
		set_default_ss58_version(ss58_version);

		let alias_str = unsafe { core::str::from_utf8_unchecked(&alias) };
		let id = <PalletId as AccountIdConversion<AccountId>>::into_account(&PalletId(alias));

		eprintln!("{}:\n\t{}\n\t{:?}", alias_str, id, id);
	}

	// da/trsry:
	// 5EYCAe5gKAhKhPeR7nUZzpcX2f9eYoAhqtEHqnG433EfnCpQ
	// 6d6f646c64612f74727372790000000000000000000000000000000000000000 (5EYCAe5g...)
	account_of(*b"da/trsry", SubstrateAccount);
	// da/ethbk:
	// 2qeMxq616BhqvTW8a1bp2g7VKPAmpda1vXuAAz5TxV5ehivG
	// 6d6f646c64612f657468626b0000000000000000000000000000000000000000 (2qeMxq61...)
	account_of(*b"da/ethbk", DarwiniaAccount);
	// da/crais:
	// 5EYCAe5gKAhHQ8Hp3UUSqEGzsUtdrevrhUadXKWuwzDYmX9T
	// 6d6f646c64612f63726169730000000000000000000000000000000000000000 (5EYCAe5g...)
	account_of(*b"da/crais", SubstrateAccount);
	// da/crabk:
	// 2qeMxq616BhqeiaffX3gbqb4PPhBo3usSkjx7ZRRTkWexMAo
	// 6d6f646c64612f637261626b0000000000000000000000000000000000000000 (2qeMxq61...)
	account_of(*b"da/crabk", DarwiniaAccount);
	// da/staki:
	// 2qeMxq616BhspChjTR7DN4GHvDMvRApmawT35ayQijghNchk
	// 6d6f646c64612f7374616b690000000000000000000000000000000000000000 (2qeMxq61...)
	account_of(*b"da/staki", DarwiniaAccount);
	// da/trobk:
	// 2qeMxq616BhswyueZhqkyWntaMt8QXshns9rBbmWBs1k9G4V
	// 6d6f646c64612f74726f626b0000000000000000000000000000000000000000 (2qeMxq61...)
	account_of(*b"da/trobk", DarwiniaAccount);
}
