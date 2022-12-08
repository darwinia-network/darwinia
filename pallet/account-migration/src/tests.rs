// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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
	[b"Darwinia2".as_slice(), b"Crab2", b"Pangolin2"]
		.iter()
		.zip([
			[
				81, 88, 182, 48, 229, 179, 23, 224, 117, 134, 146, 124, 106, 211, 130, 135, 68,
				191, 187, 224, 116, 219, 61, 45, 126, 38, 77, 144, 214, 132, 173, 77,
			],
			[
				253, 68, 12, 181, 246, 68, 61, 64, 106, 115, 48, 108, 235, 106, 2, 40, 115, 99, 84,
				46, 106, 132, 116, 241, 65, 214, 128, 8, 88, 85, 137, 86,
			],
			[
				227, 102, 33, 150, 94, 161, 161, 187, 83, 242, 232, 50, 184, 187, 169, 235, 148,
				146, 90, 88, 85, 45, 61, 24, 224, 10, 33, 182, 177, 57, 219, 61,
			],
		])
		.for_each(|(spec_name, message)| {
			assert_eq!(sr25519_signable_message(spec_name, &Default::default()), message);
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
