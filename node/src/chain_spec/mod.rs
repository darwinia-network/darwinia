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

#[cfg(feature = "darwinia-runtime")]
pub mod darwinia;

#[cfg(feature = "crab-runtime")]
pub mod crab;

// std
use std::{env, fs, thread};
// crates.io
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::runtime::Runtime as TokioRuntime;
use trauma::{download::Download, downloader::DownloaderBuilder};
// darwinia
use dc_primitives::AccountId;
// polkadot-sdk
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup, GenericChainSpec, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{Pair, Public};

pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

const PROTOCOL_ID: &str = "dar";

const ALITH: &str = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
const BALTATHAR: &str = "0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0";
const CHARLETH: &str = "0x798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc";
const DOROTHY: &str = "0x773539d4Ac0e786233D90A233654ccEE26a613D9";
const ETHAN: &str = "0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB";
const FAITH: &str = "0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d";

// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

// This is the simplest bytecode to revert without returning any data.
// We will pre-deploy it under all of our precompiles to ensure they can be called from within
// contracts. (PUSH1 0x00 PUSH1 0x00 REVERT)
const REVERT_BYTECODE: [u8; 5] = [0x60, 0x00, 0x60, 0x00, 0xFD];

const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit";

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
	fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
		TPublic::Pair::from_string(&format!("//{}", seed), None)
			.expect("static values are valid; qed")
			.public()
	}

	get_from_seed::<AuraId>(seed)
}

fn dev_accounts<F, Sks>(f: F) -> ([(AccountId, Sks); 1], [AccountId; 6])
where
	F: Fn(AuraId) -> Sks,
{
	(
		[
			// Bind the `Alice` to `Alith` to make `--alice` work for development.
			(
				array_bytes::dehexify_array_then_into::<_, _, 20>(ALITH).unwrap(),
				f(get_collator_keys_from_seed("Alice")),
			),
		],
		[
			array_bytes::dehexify_array_then_into::<_, _, 20>(ALITH).unwrap(),
			array_bytes::dehexify_array_then_into::<_, _, 20>(BALTATHAR).unwrap(),
			array_bytes::dehexify_array_then_into::<_, _, 20>(CHARLETH).unwrap(),
			array_bytes::dehexify_array_then_into::<_, _, 20>(DOROTHY).unwrap(),
			array_bytes::dehexify_array_then_into::<_, _, 20>(ETHAN).unwrap(),
			array_bytes::dehexify_array_then_into::<_, _, 20>(FAITH).unwrap(),
		],
	)
}

fn load_config<E>(name: &'static str, mut retries: u8) -> GenericChainSpec<E>
where
	E: DeserializeOwned,
{
	let d = env::current_exe().unwrap().parent().unwrap().to_path_buf();
	let p = d.join(name);
	let download = || {
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
	};

	if !p.is_file() {
		download();
	} else if let Ok(metadata) = p.metadata() {
		// Fetch the latest spec every 14 days.
		if let Ok(date) = metadata.modified() {
			if let Ok(elapsed) = date.elapsed() {
				if elapsed.as_secs() > 60 * 60 * 24 * 14 && fs::remove_file(p.clone()).is_ok() {
					download();
				}
			}
		}
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
