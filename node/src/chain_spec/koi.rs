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
// cumulus
use cumulus_primitives_core::ParaId;
// darwinia
use super::*;
use koi_runtime::*;
// frontier
use fp_evm::GenesisAccount;
// substrate
use sc_chain_spec::Properties;
use sc_service::{ChainType, GenericChainSpec};
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
			"assets": [(AssetIds::KKton as AssetId, ROOT, true, 1)],
			"metadata": [(
				AssetIds::KKton as AssetId,
				b"Koi Commitment Token",
				b"PKTON",
				18,
			)]
		},

		// Consensus stuff.
		"darwiniaStaking": {
			"now": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
			"elapsedTime": 0,
			"rateLimit": 20_000_000 * UNIT,
			"collatorCount": collators.len(),
			"collators": collators.iter().map(|(a, _)| (a, UNIT)).collect::<Vec<_>>()
		},
		"session": {
			"keys": collators.into_iter().map(|(a, sks)| (a, a, sks)).collect::<Vec<_>>()
		},

		// Governance stuff.
		"technicalCommittee": {
			"members": [
				array_bytes::hex_n_into_unchecked::<_, _, 20>(ALITH),
				array_bytes::hex_n_into_unchecked::<_, _, 20>(BALTATHAR)
			]
		},

		// Utility stuff.
		"sudo": { "key": Some(array_bytes::hex_n_into_unchecked::<_, _, 20>(ALITH)) },

		// XCM stuff.
		"polkadotXcm": { "safeXcmVersion": Some(SAFE_XCM_VERSION) },

		// EVM stuff.
		"evm": {
			"accounts": BTreeMap::from_iter(
				<KoiPrecompiles<Runtime>>::used_addresses()
					.iter()
					.map(|p| {
						(
							p.to_owned(),
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

	ChainSpec::builder(
		WASM_BINARY.unwrap(),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: PARA_ID,
		},
	)
	.with_name("Darwinia Koi D")
	.with_id("darwinia-koi-d")
	.with_chain_type(ChainType::Live)
	.with_protocol_id(PROTOCOL_ID)
	.with_properties(properties())
	.with_genesis_config_patch(genesis_config_patch)
	.build()
}

pub fn genesis_config() -> ChainSpec {
	let collators = [
		(
			array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(C1),
			session_keys(array_bytes::hex2array_unchecked(C1_AURA).unchecked_into()),
		),
		(
			array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(C2),
			session_keys(array_bytes::hex2array_unchecked(C2_AURA).unchecked_into()),
		),
		(
			array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(C3),
			session_keys(array_bytes::hex2array_unchecked(C3_AURA).unchecked_into()),
		),
	];
	let genesis_config_patch = serde_json::json!({
		// System stuff.
		"parachainInfo": { "parachainId": PARA_ID },

		// Monetary stuff.
		"balances": {
			"balances": collators.iter().map(|a| (a, 10_000 * UNIT)).collect::<Vec<_>>(),
		},
		"assets": {
			// TODO: migration.
			"assets": [(AssetIds::KKton as AssetId, ROOT, true, 1)],
			"metadata": [(
				AssetIds::KKton as AssetId,
				b"Koi Commitment Token",
				b"PKTON",
				18,
			)]
		},

		// Consensus stuff.
		"darwiniaStaking": {
			"now": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
			"elapsedTime": 0,
			"rateLimit": 20_000_000 * UNIT,
			"collatorCount": 3,
			"collators": collators.iter().map(|(a, _)| (a, UNIT)).collect::<Vec<_>>()
		},
		"session": {
			"keys": collators.iter().map(|(a, sks)| (a, a, sks)).collect::<Vec<_>>()
		},

		// Utility stuff.
		"sudo": { "key": Some(array_bytes::hex_n_into_unchecked::<_, _, 20>(SUDO)) },

		// XCM stuff.
		"polkadotXcm": { "safeXcmVersion": Some(SAFE_XCM_VERSION) },

		// EVM stuff.
		"evm": {
			"accounts": BTreeMap::from_iter(
				<KoiPrecompiles<Runtime>>::used_addresses().iter().map(|p| {
					(
						p,
						GenesisAccount {
							nonce: Default::default(),
							balance: Default::default(),
							storage: Default::default(),
							code: REVERT_BYTECODE.to_vec(),
						},
					)
				}),
			)
		}
	});

	ChainSpec::builder(
		WASM_BINARY.unwrap(),
		Extensions {
			relay_chain: "paseo".into(), // You MUST set this to the correct network!
			para_id: PARA_ID,
		},
	)
	.with_name("Darwinia Koi")
	.with_id("darwinia-koi")
	.with_chain_type(ChainType::Live)
	.with_protocol_id(PROTOCOL_ID)
	.with_properties(properties())
	.with_genesis_config_patch(genesis_config_patch)
	.with_boot_nodes(
		vec!["/dns/g1.testnets.darwinia.network/tcp/30330/p2p/12D3KooWLjJE7oNzQrEM26vUZ1uKuNYhvqjYrEATt1RdoAMTnvL9".parse().unwrap()]
	)
	.with_telemetry_endpoints(TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).unwrap())
	.build()
}

pub fn config() -> ChainSpec {
	load_config("koi.json", 0)
}

fn properties() -> Properties {
	super::properties("KRING")
}

// Generate the session keys from individual elements.
//
// The input must be a tuple of individual keys (a single arg for now since we have just one key).
fn session_keys(keys: AuraId) -> SessionKeys {
	SessionKeys { aura: keys }
}
