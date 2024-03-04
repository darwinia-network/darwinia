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

#![allow(clippy::derive_partial_eq_without_eq)]

#[cfg(feature = "darwinia-native")]
pub mod darwinia;
#[cfg(feature = "darwinia-native")]
pub use darwinia::{self as darwinia_chain_spec, ChainSpec as DarwiniaChainSpec};
#[cfg(not(feature = "darwinia-native"))]
pub type DarwiniaChainSpec = DummyChainSpec;

#[cfg(feature = "crab-native")]
pub mod crab;
#[cfg(feature = "crab-native")]
pub use crab::{self as crab_chain_spec, ChainSpec as CrabChainSpec};
#[cfg(not(feature = "crab-native"))]
pub type CrabChainSpec = DummyChainSpec;

#[cfg(feature = "pangolin-native")]
pub mod pangolin;
#[cfg(feature = "pangolin-native")]
pub use pangolin::{self as pangolin_chain_spec, ChainSpec as PangolinChainSpec};
#[cfg(not(feature = "pangolin-native"))]
pub type PangolinChainSpec = DummyChainSpec;

#[cfg(feature = "pangolin-native")]
mod testnet_keys {
	pub const C1: &str = "0x0eef9fabb6eb6fed2ab24a842931f8950426070a";
	pub const C1_AURA: &str = "0xeed007f04d568b2d3bf329945a48c21a8ed030c81ca1dce61ad41c916599f405";
	pub const C2: &str = "0xa858cde8f6cf178786578a3b0becf5c27d18300c";
	pub const C2_AURA: &str = "0x28273ae24cc6048c515e6bcaefe98cbfaa50c69290d70cf8eca1de3329015c2f";
	pub const C3: &str = "0x986b41d07776aa48f6d7a80caad49485f9a71714";
	pub const C3_AURA: &str = "0xe25d860707bd1b88b9851cf40df3af3368cd57e5e82824cabac9c75fe537600f";
	pub const SUDO: &str = "0x2748def2f9c3cfbbb963002935bc6d2e1c36ce2e";
}
#[cfg(feature = "pangolin-native")]
use testnet_keys::*;

// std
use std::{env, fs, thread};
// crates.io
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::runtime::Runtime as TokioRuntime;
use trauma::{download::Download, downloader::DownloaderBuilder};
// substrate
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup, GenericChainSpec, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{Pair, Public};

// Dummy chain spec, in case when we don't have the native runtime.
#[allow(unused)]
pub type DummyChainSpec = sc_chain_spec::GenericChainSpec<(), Extensions>;

const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit";

const ALITH: &str = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
const BALTATHAR: &str = "0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0";
const CHARLETH: &str = "0x798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc";
const DOROTHY: &str = "0x773539d4Ac0e786233D90A233654ccEE26a613D9";
const ETHAN: &str = "0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB";
const FAITH: &str = "0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d";

// This is the simplest bytecode to revert without returning any data.
// We will pre-deploy it under all of our precompiles to ensure they can be called from within
// contracts. (PUSH1 0x00 PUSH1 0x00 REVERT)
const REVERT_BYTECODE: [u8; 5] = [0x60, 0x00, 0x60, 0x00, 0xFD];

// The default XCM version to set in genesis config.
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

fn properties(token_symbol: &str) -> Properties {
	let mut properties = Properties::new();

	properties.insert("tokenSymbol".into(), token_symbol.into());
	properties.insert("tokenDecimals".into(), 18.into());

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

fn load_config<G, E>(name: &'static str, mut retries: u8) -> GenericChainSpec<G, E>
where
	E: DeserializeOwned,
{
	let d = env::current_exe().unwrap().parent().unwrap().to_path_buf();
	let p = d.join(name);

	if !p.is_file() {
		println!("Downloading `{name}` to `{}`", d.display());

		thread::spawn(move || {
			TokioRuntime::new().unwrap().block_on(
				DownloaderBuilder::new().directory(d).build().download(&[Download::try_from(
					format!(
						"https://github.com/darwinia-network/darwinia/releases/download/{}/{name}",
						name.strip_suffix(".json").unwrap()
					)
					.as_str(),
				)
				.unwrap()]),
			);
		})
		.join()
		.unwrap();
	}

	println!("Loading genesis from `{}`", p.display());

	let f_name = p.display().to_string();

	if let Ok(c) = GenericChainSpec::from_json_file(p) {
		c
	} else {
		retries += 1;

		println!("Failed to load genesis from `{f_name}`, starting the `{retries}` retries");

		// Try removing the invalid file.
		//
		// Sometimes, it might not exist.
		let _ = fs::remove_file(f_name);

		if retries == 5 {
			panic!("Exit after {retries} retries");
		}

		load_config(name, retries)
	}
}
