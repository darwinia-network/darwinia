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
	[(46_u64, b"Darwinia2".as_slice()), (44, b"Crab2"), (43, b"Pangolin2")]
		.iter()
		.zip([
			[
				75, 134, 66, 181, 153, 10, 7, 244, 225, 154, 100, 68, 239, 19, 129, 51, 181, 78,
				66, 254, 167, 54, 211, 20, 171, 68, 160, 46, 216, 98, 9, 44,
			],
			[
				171, 8, 180, 157, 214, 41, 236, 80, 127, 218, 216, 136, 239, 56, 153, 31, 128, 168,
				154, 112, 70, 245, 19, 68, 53, 29, 49, 95, 238, 209, 238, 129,
			],
			[
				251, 70, 107, 65, 22, 164, 1, 85, 114, 150, 161, 208, 235, 131, 15, 111, 154, 207,
				193, 216, 110, 54, 58, 177, 15, 99, 104, 179, 13, 30, 55, 205,
			],
		])
		.for_each(|((chain_id, spec_name), message)| {
			assert_eq!(
				sr25519_signable_message(*chain_id, spec_name, &Default::default()),
				message
			);
		});
}

#[test]
fn verify_sr25519_signature_should_work() {
	Keyring::iter().enumerate().for_each(|(i, from)| {
		let to = [i as _; 20];
		let message = sr25519_signable_message(46, b"Darwinia2", &to.into());
		let signature = from.sign(&message);

		assert!(verify_sr25519_signature(&from.public().0.into(), &message, &signature));
	});
}
