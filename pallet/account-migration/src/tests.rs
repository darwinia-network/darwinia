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
use crate::*;
// substrate
use sp_keyring::sr25519::Keyring;

#[test]
fn sr25519_signable_message_should_work() {
	["Darwinia2", "Crab2", "Pangolin2"].iter().for_each(|s| {
		assert_eq!(
			sr25519_signable_message(s.as_bytes(), &Default::default()),
			format!(
				"<Bytes>I authorize the migration to {}, an unused address on {}. Sign this message to authorize using the Substrate key associated with the account on {} that you wish to migrate.</Bytes>",
				"0x0000000000000000000000000000000000000000",
				s,
				&s[..s.len() - 1],
			).as_bytes()
		);
	});
}

#[test]
fn verify_sr25519_signature_should_work() {
	Keyring::iter().enumerate().for_each(|(i, from)| {
		let to = [i as _; 20];
		let message = sr25519_signable_message(b"Darwinia2", &to.into());
		let signature = from.sign(&message);

		assert!(verify_sr25519_signature(&from.public().0.into(), &message, &signature));
	});
}
