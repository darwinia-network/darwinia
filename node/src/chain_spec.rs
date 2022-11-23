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

// std
use std::{collections::BTreeMap, str::FromStr};
// crates.io
use serde::{Deserialize, Serialize};
// cumulus
use cumulus_primitives_core::ParaId;
// darwinia
use darwinia_runtime::{AuraId, DarwiniaPrecompiles, EvmConfig, Runtime};
use dc_primitives::*;
// frontier
use fp_evm::GenesisAccount;
// substrate
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use sp_core::{Pair, Public, H160};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<darwinia_runtime::GenesisConfig, Extensions>;

/// This is the simplest bytecode to revert without returning any data.
/// We will pre-deploy it under all of our precompiles to ensure they can be called from within
/// contracts. (PUSH1 0x00 PUSH1 0x00 REVERT)
pub const REVERT_BYTECODE: [u8; 5] = [0x60, 0x00, 0x60, 0x00, 0xFD];

// These are are testnet-only keys.
const ALITH: &str = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
const BALTATHAR: &str = "0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0";
const CHARLETH: &str = "0x798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc";
const DOROTHY: &str = "0x773539d4Ac0e786233D90A233654ccEE26a613D9";
const ETHAN: &str = "0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB";
const FAITH: &str = "0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d";

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

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

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_from_seed::<AuraId>(seed)
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn session_keys(keys: AuraId) -> darwinia_runtime::SessionKeys {
	darwinia_runtime::SessionKeys { aura: keys }
}

pub fn development_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "RING".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 18.into());

	ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					// Bind the `Alice` to `Alith` to make `--alice` available for testnet.
					(
						array_bytes::hex_n_into_unchecked(ALITH),
						get_collator_keys_from_seed("Alice"),
					),
					// Bind the `Bob` to `Balthar` to make `--bob` available for testnet.
					(
						array_bytes::hex_n_into_unchecked(BALTATHAR),
						get_collator_keys_from_seed("Bob"),
					),
				],
				vec![
					array_bytes::hex_n_into_unchecked(ALITH),
					array_bytes::hex_n_into_unchecked(BALTATHAR),
					array_bytes::hex_n_into_unchecked(CHARLETH),
					array_bytes::hex_n_into_unchecked(DOROTHY),
					array_bytes::hex_n_into_unchecked(ETHAN),
					array_bytes::hex_n_into_unchecked(FAITH),
				],
				1000.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: 1000,
		},
	)
}

pub fn local_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "RING".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 18.into());

	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					// Bind the `Alice` to `Alith` to make `--alice` available for testnet.
					(
						array_bytes::hex_n_into_unchecked(ALITH),
						get_collator_keys_from_seed("Alice"),
					),
					// Bind the `Bob` to `Balthar` to make `--bob` available for testnet.
					(
						array_bytes::hex_n_into_unchecked(BALTATHAR),
						get_collator_keys_from_seed("Bob"),
					),
				],
				vec![
					array_bytes::hex_n_into_unchecked(ALITH),
					array_bytes::hex_n_into_unchecked(BALTATHAR),
					array_bytes::hex_n_into_unchecked(CHARLETH),
					array_bytes::hex_n_into_unchecked(DOROTHY),
					array_bytes::hex_n_into_unchecked(ETHAN),
					array_bytes::hex_n_into_unchecked(FAITH),
				],
				1000.into(),
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("darwinia"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: 1000,
		},
	)
}

pub fn shell_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "RING".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 18.into());

	// TODO: update this before final release
	ChainSpec::from_genesis(
		// Name
		"Darwinia",
		// ID
		"darwinia",
		ChainType::Live,
		move || {
			darwinia_runtime::GenesisConfig {
				// System stuff.
				system: darwinia_runtime::SystemConfig {
					code: darwinia_runtime::WASM_BINARY
						.expect("WASM binary was not build, please build it!")
						.to_vec(),
				},
				parachain_system: Default::default(),
				parachain_info: darwinia_runtime::ParachainInfoConfig { parachain_id: 2046.into() },

				// Monetary stuff.
				balances: Default::default(),
				transaction_payment: Default::default(),

				// Consensus stuff.
				collator_selection: darwinia_runtime::CollatorSelectionConfig {
					invulnerables: vec![array_bytes::hex_n_into_unchecked(ALITH)],
					..Default::default()
				},
				session: darwinia_runtime::SessionConfig {
					keys: vec![(
						array_bytes::hex_n_into_unchecked(ALITH),
						array_bytes::hex_n_into_unchecked(ALITH),
						session_keys(get_collator_keys_from_seed("Alice")),
					)],
				},
				// no need to pass anything to aura, in fact it will panic if we do. Session will
				// take care of this.
				aura: Default::default(),
				aura_ext: Default::default(),

				// Governance stuff.
				democracy: Default::default(),
				council: Default::default(),
				technical_committee: Default::default(),
				phragmen_election: Default::default(),
				technical_membership: Default::default(),
				treasury: Default::default(),

				// Utility stuff.
				sudo: Default::default(),
				vesting: Default::default(),

				// XCM stuff.
				polkadot_xcm: darwinia_runtime::PolkadotXcmConfig {
					safe_xcm_version: Some(SAFE_XCM_VERSION),
				},

				// EVM stuff.
				ethereum: Default::default(),
				evm: Default::default(),
				base_fee: Default::default(),
			}
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("darwinia"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "polkadot".into(), // You MUST set this to the correct network!
			para_id: 2046,
		},
	)
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> darwinia_runtime::GenesisConfig {
	darwinia_runtime::GenesisConfig {
		// System stuff.
		system: darwinia_runtime::SystemConfig {
			code: darwinia_runtime::WASM_BINARY.unwrap().to_vec(),
		},
		parachain_system: Default::default(),
		parachain_info: darwinia_runtime::ParachainInfoConfig { parachain_id: id },

		// Monetary stuff.
		balances: darwinia_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 100_000_000 * UNIT)).collect(),
		},
		transaction_payment: Default::default(),

		// Consensus stuff.
		collator_selection: darwinia_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: UNIT,
			..Default::default()
		},
		session: darwinia_runtime::SessionConfig {
			keys: invulnerables
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
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),

		// Governance stuff.
		democracy: Default::default(),
		council: Default::default(),
		technical_committee: Default::default(),
		phragmen_election: Default::default(),
		technical_membership: Default::default(),
		treasury: Default::default(),

		// Utility stuff.
		sudo: Default::default(),
		vesting: Default::default(),

		// XCM stuff.
		polkadot_xcm: darwinia_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},

		// EVM stuff.
		ethereum: Default::default(),
		evm: EvmConfig {
			accounts: {
				BTreeMap::from_iter(
					DarwiniaPrecompiles::<Runtime>::used_addresses()
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
							// Testing account.
							(
								H160::from_str("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b")
									.unwrap(),
								GenesisAccount {
									balance: (10_000_000 * UNIT).into(),
									code: Default::default(),
									nonce: Default::default(),
									storage: Default::default(),
								},
							),
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
		},
		base_fee: Default::default(),
	}
}
