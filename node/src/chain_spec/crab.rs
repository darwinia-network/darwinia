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
use crab_runtime::*;
// frontier
use fp_evm::GenesisAccount;
// substrate
use sc_chain_spec::Properties;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_core::{crypto::UncheckedInto, H160};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig, Extensions>;

fn properties() -> Properties {
	super::properties("CRAB")
}

// Generate the session keys from individual elements.
//
// The input must be a tuple of individual keys (a single arg for now since we have just one key).
fn session_keys(keys: AuraId) -> SessionKeys {
	SessionKeys { aura: keys }
}

pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		// Fulfill Polkadot.JS metadata upgrade requirements.
		"Crab2 D",
		"crab2-d",
		ChainType::Live,
		move || {
			testnet_genesis(
				vec![
					// Bind the `Alice` to `Alith` to make `--alice` available for testnet.
					(
						array_bytes::hex_n_into_unchecked::<_, _, 20>(ALITH),
						get_collator_keys_from_seed("Alice"),
					),
				],
				vec![
					array_bytes::hex_n_into_unchecked::<_, _, 20>(ALITH),
					array_bytes::hex_n_into_unchecked::<_, _, 20>(BALTATHAR),
					array_bytes::hex_n_into_unchecked::<_, _, 20>(CHARLETH),
					array_bytes::hex_n_into_unchecked::<_, _, 20>(DOROTHY),
					array_bytes::hex_n_into_unchecked::<_, _, 20>(ETHAN),
					array_bytes::hex_n_into_unchecked::<_, _, 20>(FAITH),
				],
				2105.into(),
			)
		},
		Vec::new(),
		None,
		Some(PROTOCOL_ID),
		None,
		Some(properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: 2105,
		},
	)
}

pub fn local_config() -> ChainSpec {
	ChainSpec::from_genesis(
		// Fulfill Polkadot.JS metadata upgrade requirements.
		"Crab2 L",
		"crab2-l",
		ChainType::Live,
		move || {
			testnet_genesis(
				vec![
					// Bind the `Alice` to `Alith` to make `--alice` available for testnet.
					(
						array_bytes::hex_n_into_unchecked::<_, _, 20>(ALITH),
						get_collator_keys_from_seed("Alice"),
					),
					// Bind the `Bob` to `Balthar` to make `--bob` available for testnet.
					(
						array_bytes::hex_n_into_unchecked::<_, _, 20>(BALTATHAR),
						get_collator_keys_from_seed("Bob"),
					),
					// Bind the `Charlie` to `CHARLETH` to make `--charlie` available for testnet.
					(
						array_bytes::hex_n_into_unchecked::<_, _, 20>(CHARLETH),
						get_collator_keys_from_seed("Charlie"),
					),
				],
				vec![
					array_bytes::hex_n_into_unchecked::<_, _, 20>(ALITH),
					array_bytes::hex_n_into_unchecked::<_, _, 20>(BALTATHAR),
					array_bytes::hex_n_into_unchecked::<_, _, 20>(CHARLETH),
					array_bytes::hex_n_into_unchecked::<_, _, 20>(DOROTHY),
					array_bytes::hex_n_into_unchecked::<_, _, 20>(ETHAN),
					array_bytes::hex_n_into_unchecked::<_, _, 20>(FAITH),
				],
				2105.into(),
			)
		},
		Vec::new(),
		None,
		Some(PROTOCOL_ID),
		None,
		Some(properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: 2105,
		},
	)
}

pub fn genesis_config() -> ChainSpec {
	let collators = [
		(
			"0x196f03b77a1acd0db080006b04d2f3a991ebbe68",
			"0xa05255010ee986b9684a444d10a74aa0ecbe781f5002e871665add894752cc7e",
		),
		(
			"0x4D3D5958B948e8d749FaB0236b179fCC55d9aAc0",
			"0xf8f82edfc6899552e5a32aa381e53723d2c39594638cb8f7e2572fef74b05255",
		),
		(
			"0x7aE2a0914db8bFBdad538b0eAc3Fa473A0e07843",
			"0xdaf5c4506b82f617245150216a73c0eb4f2603848c02413db66f991846777845",
		),
		(
			"0x9F33a4809aA708d7a399fedBa514e0A0d15EfA85",
			"0xdcff1219121687391353b17e798b10e87f6e578b2a01e032375f2f14a0712b57",
		),
		(
			"0x0a1287977578F888bdc1c7627781AF1cc000e6ab",
			"0x28a8af71db9703e6b8960d1dcb742deca13c574f81f781be5dbde84ec8d66d45",
		),
		(
			"0xEB7e82A67CDFA3E742e0f3315Fd4EEd7B05730CC",
			"0xfee21e4e4865380734882253d27612da0e4413c93e5c817e38b8c5e034de7270",
		),
	];

	ChainSpec::from_genesis(
		"Crab2",
		"crab2",
		ChainType::Live,
		move || {
			RuntimeGenesisConfig {
				// System stuff.
				system: SystemConfig { code: WASM_BINARY.unwrap().to_vec(), ..Default::default() },
				parachain_system: Default::default(),
				parachain_info: ParachainInfoConfig { parachain_id: 2105.into(), ..Default::default() },

				// Monetary stuff.
				balances: BalancesConfig {
					balances: collators
						.iter()
						.map(|(k, _)| (array_bytes::hex_n_into_unchecked::<_, _, 20>(k), 10_000 * UNIT))
						.collect(),
				},
				transaction_payment: Default::default(),
				assets: AssetsConfig {
					assets: vec![(AssetIds::CKton as _, ROOT, true, 1)],
					metadata: vec![(
						AssetIds::CKton as _,
						b"Crab Commitment Token".to_vec(),
						b"CKTON".to_vec(),
						18,
					)],
					..Default::default()
				},
				vesting: Default::default(),

				// Consensus stuff.
				darwinia_staking: DarwiniaStakingConfig {
					now: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
					elapsed_time: 0,
					collator_count: 6,
					collators: collators
						.iter()
						.map(|(k, _)| (array_bytes::hex_n_into_unchecked::<_, _, 20>(k), 1_000 * UNIT))
						.collect(),
				},
				session: SessionConfig {
					keys: collators
						.iter()
						.map(|(k, a)| {
							(
								array_bytes::hex_n_into_unchecked::<_, _, 20>(k),
								array_bytes::hex_n_into_unchecked::<_, _, 20>(k),
								session_keys(array_bytes::hex2array_unchecked(a).unchecked_into()),
							)
						})
						.collect(),
				},
				aura: Default::default(),
				aura_ext: Default::default(),

				// Governance stuff.
				technical_committee: Default::default(),
				treasury: Default::default(),

				// Utility stuff.
				tx_pause: Default::default(),

				// XCM stuff.
				polkadot_xcm: PolkadotXcmConfig { safe_xcm_version: Some(SAFE_XCM_VERSION), ..Default::default() },

				// EVM stuff.
				ethereum: Default::default(),
				evm: EVMConfig {
					accounts: {
						BTreeMap::from_iter(
							CrabPrecompiles::<Runtime>::used_addresses().iter().map(|p| {
								(
									p.to_owned(),
									GenesisAccount {
										nonce: Default::default(),
										balance: Default::default(),
										storage: Default::default(),
										code: REVERT_BYTECODE.to_vec(),
									},
								)
							}),
						)
					},
					..Default::default()
				},
			}
		},
		vec![
			"/dns/g1.crab2.darwinia.network/tcp/30333/ws/p2p/12D3KooWEDiHG6pjt53HqnfYepnLzp9rFTh8MJrBX7AZeGShBMM4".parse().unwrap()
		],
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		Some(PROTOCOL_ID),
		None,
		Some(properties()),
		Extensions {
			relay_chain: "kusama".into(), // You MUST set this to the correct network!
			para_id: 2105,
		},
	)
}

pub fn config() -> ChainSpec {
	load_config("crab2.json", 0)
}

fn testnet_genesis(
	collators: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> RuntimeGenesisConfig {
	RuntimeGenesisConfig {
		// System stuff.
		system: SystemConfig { code: WASM_BINARY.unwrap().to_vec(), ..Default::default() },
		parachain_system: Default::default(),
		parachain_info: ParachainInfoConfig { parachain_id: id, ..Default::default() },

		// Monetary stuff.
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 100_000_000 * UNIT)).collect(),
		},
		transaction_payment: Default::default(),
		assets: AssetsConfig {
			assets: vec![(AssetIds::CKton as _, ROOT, true, 1)],
			metadata: vec![(
				AssetIds::CKton as _,
				b"Crab Commitment Token".to_vec(),
				b"CKTON".to_vec(),
				18,
			)],
			..Default::default()
		},
		vesting: Default::default(),

		// Consensus stuff.
		darwinia_staking: DarwiniaStakingConfig {
			now: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
			elapsed_time: 0,
			collator_count: collators.len() as _,
			collators: collators.iter().map(|(a, _)| (a.to_owned(), UNIT)).collect(),
		},
		session: SessionConfig {
			keys: collators
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc,                // account id
						acc,                // validator id
						session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),

		// Governance stuff.
		technical_committee: Default::default(),
		treasury: Default::default(),

		// Utility stuff.
		tx_pause: Default::default(),

		// XCM stuff.
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},

		// EVM stuff.
		ethereum: Default::default(),
		evm: EVMConfig {
			accounts: {
				BTreeMap::from_iter(
					CrabPrecompiles::<Runtime>::used_addresses()
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
			},
			..Default::default()
		},
	}
}
