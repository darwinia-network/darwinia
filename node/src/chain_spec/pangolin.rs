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
use pangolin_runtime::*;
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
	super::properties("PRING")
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
		"Pangolin2 D",
		"pangolin2-d",
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
		"Pangolin2 L",
		"pangolin2-l",
		// Fulfill Polkadot.JS metadata upgrade requirements.
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
	ChainSpec::from_genesis(
		"Pangolin2",
		"pangolin2",
		ChainType::Live,
		move || {
			RuntimeGenesisConfig {
				// System stuff.
				system: SystemConfig { code: WASM_BINARY.unwrap().to_vec(), ..Default::default() },
				parachain_system: Default::default(),
				parachain_info: ParachainInfoConfig { parachain_id: 2105.into(), ..Default::default() },

				// Monetary stuff.
				balances: BalancesConfig {
					balances: vec![
						(array_bytes::hex_n_into_unchecked::<_, _, 20>(C1), 10_000 * UNIT),
						(array_bytes::hex_n_into_unchecked::<_, _, 20>(C2), 10_000 * UNIT),
						(array_bytes::hex_n_into_unchecked::<_, _, 20>(C3), 10_000 * UNIT),
					],
				},
				transaction_payment: Default::default(),
				assets: AssetsConfig {
					assets: vec![(AssetIds::PKton as _, ROOT, true, 1)],
					metadata: vec![(
						AssetIds::PKton as _,
						b"Pangolin Commitment Token".to_vec(),
						b"PKTON".to_vec(),
						18,
					)],
					..Default::default()
				},

				// Consensus stuff.
				darwinia_staking: DarwiniaStakingConfig {
					now: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
					elapsed_time: 0,
					collator_count: 3,
					collators: vec![
						(array_bytes::hex_n_into_unchecked::<_, _, 20>(C1), UNIT),
						(array_bytes::hex_n_into_unchecked::<_, _, 20>(C2), UNIT),
						(array_bytes::hex_n_into_unchecked::<_, _, 20>(C3), UNIT),
					],
				},
				session: SessionConfig {
					keys: vec![
						(
							array_bytes::hex_n_into_unchecked::<_, _, 20>(C1),
							array_bytes::hex_n_into_unchecked::<_, _, 20>(C1),
							session_keys(
								array_bytes::hex2array_unchecked(C1_AURA).unchecked_into(),
							),
						),
						(
							array_bytes::hex_n_into_unchecked::<_, _, 20>(C2),
							array_bytes::hex_n_into_unchecked::<_, _, 20>(C2),
							session_keys(
								array_bytes::hex2array_unchecked(C2_AURA).unchecked_into(),
							),
						),
						(
							array_bytes::hex_n_into_unchecked::<_, _, 20>(C3),
							array_bytes::hex_n_into_unchecked::<_, _, 20>(C3),
							session_keys(
								array_bytes::hex2array_unchecked(C3_AURA).unchecked_into(),
							),
						),
					],
				},
				aura: Default::default(),
				aura_ext: Default::default(),
				message_gadget: Default::default(),
				ecdsa_authority: Default::default(),

				// Governance stuff.
				technical_committee: Default::default(),
				treasury: Default::default(),

				// Utility stuff.
				sudo: SudoConfig { key: Some(array_bytes::hex_n_into_unchecked::<_, _, 20>(SUDO)) },
				tx_pause: Default::default(),

				// XCM stuff.
				polkadot_xcm: PolkadotXcmConfig { safe_xcm_version: Some(SAFE_XCM_VERSION), ..Default::default() },

				// EVM stuff.
				ethereum: Default::default(),
				evm: EVMConfig {
					accounts: {
						BTreeMap::from_iter(
							PangolinPrecompiles::<Runtime>::used_addresses().iter().map(|p| {
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

				// S2S stuff.
				bridge_moonbase_grandpa: Default::default(),
				bridge_moonbase_parachain: Default::default(),
				bridge_pangoro_messages: Default::default(),
				pangoro_fee_market: Default::default(),
			}
		},
		vec![
			"/dns/g1.testnets.darwinia.network/tcp/30330/p2p/12D3KooWLjJE7oNzQrEM26vUZ1uKuNYhvqjYrEATt1RdoAMTnvL9".parse().unwrap()
		],
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		Some(PROTOCOL_ID),
		None,
		Some(properties()),
		Extensions {
			relay_chain: "rococo".into(), // You MUST set this to the correct network!
			para_id: 2105,
		},
	)
}

pub fn config() -> ChainSpec {
	load_config("pangolin2.json", 0)
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
			assets: vec![(AssetIds::PKton as _, ROOT, true, 1)],
			metadata: vec![(
				AssetIds::PKton as _,
				b"Pangolin Commitment Token".to_vec(),
				b"PKTON".to_vec(),
				18,
			)],
			..Default::default()
		},

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
		message_gadget: Default::default(),
		ecdsa_authority: Default::default(),

		// Governance stuff.
		technical_committee: TechnicalCommitteeConfig {
			members: vec![
				array_bytes::hex_n_into_unchecked::<_, _, 20>(ALITH),
				array_bytes::hex_n_into_unchecked::<_, _, 20>(BALTATHAR),
			],
			..Default::default()
		},
		treasury: Default::default(),

		// Utility stuff.
		sudo: SudoConfig { key: Some(array_bytes::hex_n_into_unchecked::<_, _, 20>(ALITH)) },
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
					PangolinPrecompiles::<Runtime>::used_addresses()
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

		// S2S stuff.
		bridge_moonbase_grandpa: Default::default(),
		bridge_moonbase_parachain: Default::default(),
		bridge_pangoro_messages: Default::default(),
		pangoro_fee_market: Default::default(),
	}
}
