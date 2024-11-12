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

// std
use std::{
	collections::BTreeMap,
	str::FromStr,
	time::{SystemTime, UNIX_EPOCH},
};
// crates.io
#[cfg(feature = "dev")]
use serde_json::Value;
// darwinia
use super::*;
use crab_runtime::*;
// frontier
use fp_evm::GenesisAccount;
// polkadot-sdk
use sc_chain_spec::Properties;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_core::{crypto::UncheckedInto, H160};

const PARA_ID: u32 = 2105;

pub fn development_config() -> ChainSpec {
	let (collators, endowed_accounts) = dev_accounts(session_keys);
	let genesis_config_patch = serde_json::json!({
		// System stuff.
		"parachainInfo": { "parachainId": PARA_ID },

		// Monetary stuff.
		"balances": {
			"balances": endowed_accounts.iter().map(|a| (a, 100_000_000 * UNIT)).collect::<Vec<_>>()
		},
		"assets": {
			"assets": [(AssetIds::CKton as AssetId, ROOT, true, 1)],
			"metadata": [(
				AssetIds::CKton as AssetId,
				b"Crab Commitment Token",
				b"CKTON",
				18
			)]
		},

		// Consensus stuff.
		"darwiniaStaking": {
			"now": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
			"elapsedTime": 0,
			"collatorCount": collators.len()
		},
		"session": {
			"keys": collators.into_iter().map(|(a, sks)| (a, a, sks)).collect::<Vec<_>>()
		},

		// XCM stuff.
		"polkadotXcm": { "safeXcmVersion": Some(SAFE_XCM_VERSION) },

		// EVM stuff.
		"evm": {
			"accounts": BTreeMap::from_iter(
				Precompiles::set()
					.into_iter()
					.map(|p| {
						(
							H160(p),
							GenesisAccount {
								nonce: Default::default(),
								balance: Default::default(),
								storage: Default::default(),
								code: REVERT_BYTECODE.to_vec(),
							},
						)
					})
					.chain([
						// Benchmarking account.
						(
							H160::from_str("1000000000000000000000000000000000000001").unwrap(),
							GenesisAccount {
								nonce: 1.into(),
								balance: (10_000_000 * UNIT).into(),
								storage: Default::default(),
								code: vec![0x00],
							},
						),
					]),
			)
		}
	});
	#[cfg(feature = "dev")]
	let genesis_config_patch = if let Value::Object(mut m) = genesis_config_patch {
		m.insert(
			"sudo".into(),
			serde_json::json!({ "key": Some(array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(ALITH)) }),
		);

		Value::Object(m)
	} else {
		unreachable!();
	};

	ChainSpec::builder(
		WASM_BINARY.unwrap(),
		Extensions { relay_chain: "rococo-local".into(), para_id: PARA_ID },
	)
	.with_name("Crab2 D")
	.with_id("crab2-d")
	.with_chain_type(ChainType::Development)
	.with_protocol_id(PROTOCOL_ID)
	.with_properties(properties())
	.with_genesis_config_patch(genesis_config_patch)
	.build()
}

pub fn genesis_config() -> ChainSpec {
	let collators = [
		(
			array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
				"0x196f03b77a1acd0db080006b04d2f3a991ebbe68",
			),
			session_keys(
				array_bytes::hex2array_unchecked(
					"0xa05255010ee986b9684a444d10a74aa0ecbe781f5002e871665add894752cc7e",
				)
				.unchecked_into(),
			),
		),
		(
			array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
				"0x4D3D5958B948e8d749FaB0236b179fCC55d9aAc0",
			),
			session_keys(
				array_bytes::hex2array_unchecked(
					"0xf8f82edfc6899552e5a32aa381e53723d2c39594638cb8f7e2572fef74b05255",
				)
				.unchecked_into(),
			),
		),
		(
			array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
				"0x7aE2a0914db8bFBdad538b0eAc3Fa473A0e07843",
			),
			session_keys(
				array_bytes::hex2array_unchecked(
					"0xdaf5c4506b82f617245150216a73c0eb4f2603848c02413db66f991846777845",
				)
				.unchecked_into(),
			),
		),
		(
			array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
				"0x9F33a4809aA708d7a399fedBa514e0A0d15EfA85",
			),
			session_keys(
				array_bytes::hex2array_unchecked(
					"0xdcff1219121687391353b17e798b10e87f6e578b2a01e032375f2f14a0712b57",
				)
				.unchecked_into(),
			),
		),
		(
			array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
				"0x0a1287977578F888bdc1c7627781AF1cc000e6ab",
			),
			session_keys(
				array_bytes::hex2array_unchecked(
					"0x28a8af71db9703e6b8960d1dcb742deca13c574f81f781be5dbde84ec8d66d45",
				)
				.unchecked_into(),
			),
		),
		(
			array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
				"0xEB7e82A67CDFA3E742e0f3315Fd4EEd7B05730CC",
			),
			session_keys(
				array_bytes::hex2array_unchecked(
					"0xfee21e4e4865380734882253d27612da0e4413c93e5c817e38b8c5e034de7270",
				)
				.unchecked_into(),
			),
		),
	];
	let genesis_config_patch = serde_json::json!({
		// System stuff.
		"parachainInfo": { "parachainId": PARA_ID },

		// Monetary stuff.
		"balances": {
			"balances": collators.iter().map(|(a, _)| (a, 10_000 * UNIT)).collect::<Vec<_>>()
		},
		"assets": {
			"assets": [(AssetIds::CKton as AssetId, ROOT, true, 1)],
			"metadata": [(
				AssetIds::CKton as AssetId,
				b"Crab Commitment Token",
				b"CKTON",
				18
			)]
		},

		// Consensus stuff.
		"darwiniaStaking": {
			"now": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
			"elapsedTime": 0,
			"collatorCount": 6
		},
		"session": {
			"keys": collators.iter().map(|(a, sks)| (a, a, sks)).collect::<Vec<_>>()
		},

		// XCM stuff.
		"polkadotXcm": { "safeXcmVersion": Some(SAFE_XCM_VERSION) },

		// EVM stuff.
		"evm": {
			"accounts": BTreeMap::from_iter(
				Precompiles::set().iter().map(|p| {
					(
						p,
						GenesisAccount {
							nonce: Default::default(),
							balance: Default::default(),
							storage: Default::default(),
							code: REVERT_BYTECODE.to_vec()
						}
					)
				})
			)
		}
	});

	ChainSpec::builder(WASM_BINARY.unwrap(), Extensions {
		relay_chain: "kusama".into(),
		para_id: PARA_ID,
	})
	.with_name("Crab2")
	.with_id("crab2")
	.with_chain_type(ChainType::Live)
	.with_protocol_id(PROTOCOL_ID)
	.with_properties(properties())
	.with_genesis_config_patch(genesis_config_patch)
	.with_boot_nodes(vec![
		"/dns/g1.crab2.darwinia.network/tcp/30333/ws/p2p/12D3KooWEDiHG6pjt53HqnfYepnLzp9rFTh8MJrBX7AZeGShBMM4".parse().unwrap(),
		"/dns/g2.crab2.darwinia.network/tcp/30333/ws/p2p/12D3KooWJeZ7xoj912homUscXe6JxW1suJ1M2BvjPuGWt18HHokJ".parse().unwrap(),
	])
	.with_telemetry_endpoints(
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).unwrap()
	)
	.build()
}

pub fn config() -> ChainSpec {
	load_config("crab2.json", 0)
}

fn properties() -> Properties {
	super::properties("CRAB")
}

// Generate the session keys from individual elements.
//
// The input must be a tuple of individual keys (a single arg for now since we have just one key).
fn session_keys(keys: AuraId) -> SessionKeys {
	SessionKeys { aura: keys }
}
