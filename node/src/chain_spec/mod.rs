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

#![allow(clippy::derive_partial_eq_without_eq)]

pub mod darwinia;
pub use darwinia::{self as darwinia_chain_spec, ChainSpec as DarwiniaChainSpec};

pub mod crab;
pub use crab::{self as crab_chain_spec, ChainSpec as CrabChainSpec};

pub mod pangolin;
pub use pangolin::{self as pangolin_chain_spec, ChainSpec as PangolinChainSpec};

// crates.io
use serde::{Deserialize, Serialize};
// substrate
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{Pair, Public};

// This is the simplest bytecode to revert without returning any data.
// We will pre-deploy it under all of our precompiles to ensure they can be called from within
// contracts. (PUSH1 0x00 PUSH1 0x00 REVERT)
const REVERT_BYTECODE: [u8; 5] = [0x60, 0x00, 0x60, 0x00, 0xFD];

// These are are testnet-only keys.
const ALITH: &str = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
const BALTATHAR: &str = "0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0";
const CHARLETH: &str = "0x798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc";
const DOROTHY: &str = "0x773539d4Ac0e786233D90A233654ccEE26a613D9";
const ETHAN: &str = "0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB";
const FAITH: &str = "0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d";

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

const PROTOCOL_ID: &str = "dar";

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}
impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

fn properties(token_symbol: &str, ss58_format: u16) -> Properties {
	let mut properties = Properties::new();

	properties.insert("tokenSymbol".into(), token_symbol.into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), ss58_format.into());

	properties
}

fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_from_seed::<AuraId>(seed)
}
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}
