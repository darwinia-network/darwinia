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

// core
use core::str::FromStr;
// darwinia
use super::*;

const TRACKS_DATA: [(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>); 4] = [
	(
		0,
		pallet_referenda::TrackInfo {
			// Name of this track.
			name: "root",
			// A limit for the number of referenda on this track that can be being decided at once.
			// For Root origin this should generally be just one.
			max_deciding: 5,
			// Amount that must be placed on deposit before a decision can be made.
			decision_deposit: DARWINIA_PROPOSAL_REQUIREMENT,
			// Amount of time this must be submitted for before a decision can be made.
			prepare_period: TIME_1,
			// Amount of time that a decision may take to be approved prior to cancellation.
			decision_period: TIME_1,
			// Amount of time that the approval criteria must hold before it can be approved.
			confirm_period: TIME_1,
			// Minimum amount of time that an approved proposal must be in the dispatch queue.
			min_enactment_period: TIME_1,
			// Minimum aye votes as percentage of overall conviction-weighted votes needed for
			// approval as a function of time into decision period.
			min_approval: pallet_referenda::Curve::make_reciprocal(
				1,
				2,
				percent(80),
				percent(50),
				percent(100),
			),
			// Minimum pre-conviction aye-votes ("support") as percentage of overall population that
			// is needed for approval as a function of time into decision period.
			min_support: pallet_referenda::Curve::make_linear(2, 2, permill(5), percent(25)),
		},
	),
	(
		1,
		pallet_referenda::TrackInfo {
			name: "whitelisted_caller",
			max_deciding: 100,
			decision_deposit: DARWINIA_PROPOSAL_REQUIREMENT,
			prepare_period: TIME_1,
			decision_period: TIME_1,
			confirm_period: TIME_1,
			min_enactment_period: TIME_1,
			min_approval: pallet_referenda::Curve::make_reciprocal(
				1,
				2,
				percent(96),
				percent(50),
				percent(100),
			),
			min_support: pallet_referenda::Curve::make_reciprocal(
				1,
				2,
				percent(1),
				percent(0),
				percent(2),
			),
		},
	),
	// 2 for deprecated general admin.
	(
		3,
		pallet_referenda::TrackInfo {
			name: "referendum_canceller",
			max_deciding: 20,
			decision_deposit: DARWINIA_PROPOSAL_REQUIREMENT,
			prepare_period: TIME_1,
			decision_period: TIME_1,
			confirm_period: TIME_1,
			min_enactment_period: TIME_1,
			min_approval: pallet_referenda::Curve::make_reciprocal(
				1,
				2,
				percent(96),
				percent(50),
				percent(100),
			),
			min_support: pallet_referenda::Curve::make_reciprocal(
				1,
				2,
				percent(1),
				percent(0),
				percent(10),
			),
		},
	),
	(
		4,
		pallet_referenda::TrackInfo {
			name: "referendum_killer",
			max_deciding: 100,
			decision_deposit: DARWINIA_PROPOSAL_REQUIREMENT,
			prepare_period: TIME_1,
			decision_period: TIME_1,
			confirm_period: TIME_1,
			min_enactment_period: TIME_1,
			min_approval: pallet_referenda::Curve::make_reciprocal(
				1,
				2,
				percent(96),
				percent(50),
				percent(100),
			),
			min_support: pallet_referenda::Curve::make_reciprocal(
				1,
				2,
				percent(1),
				percent(0),
				percent(10),
			),
		},
	),
	// 5 for deprecated medium spender.
	// 6 for deprecated big spender.
];

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
	type Id = u16;
	type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;

	fn tracks() -> &'static [(Self::Id, pallet_referenda::TrackInfo<Balance, BlockNumber>)] {
		&TRACKS_DATA[..]
	}

	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => {
					if let Some((track_id, _)) =
						Self::tracks().iter().find(|(_, track)| track.name == "root")
					{
						Ok(*track_id)
					} else {
						Err(())
					}
				},
				_ => Err(()),
			}
		} else if let Ok(custom_origin) = custom_origins::Origin::try_from(id.clone()) {
			if let Some((track_id, _)) = Self::tracks().iter().find(|(_, track)| {
				if let Ok(track_custom_origin) = custom_origins::Origin::from_str(track.name) {
					track_custom_origin == custom_origin
				} else {
					false
				}
			}) {
				Ok(*track_id)
			} else {
				Err(())
			}
		} else {
			Err(())
		}
	}
}

// To ensure voters are always locked into their vote
#[test]
fn vote_locking_always_longer_than_enactment_period() {
	for (_, track) in TRACKS_DATA {
		let vote_locking_period = <<Runtime as pallet_conviction_voting::Config>::VoteLockingPeriod as frame_support::traits::Get<BlockNumber>>::get();

		assert!(
			vote_locking_period >= track.min_enactment_period,
			"Track {} has enactment period {} < vote locking period {vote_locking_period}",
			track.name,
			track.min_enactment_period,
		);
	}
}

#[test]
fn all_tracks_have_origins() {
	for (_, track) in TRACKS_DATA {
		// check name.into() is successful either converts into "root" or custom origin
		let track_is_root = track.name == "root";
		let track_has_custom_origin = custom_origins::Origin::from_str(track.name).is_ok();

		assert!(track_is_root || track_has_custom_origin);
	}
}
