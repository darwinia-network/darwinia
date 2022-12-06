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
// cumulus
use cumulus_primitives_core::ParaId;
// darwinia
use super::*;
use crab_runtime::{AuraId, CrabPrecompiles, EvmConfig, Runtime};
use dc_primitives::*;
// frontier
use fp_evm::GenesisAccount;
// substrate
use sc_service::ChainType;
use sp_core::H160;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<crab_runtime::GenesisConfig, Extensions>;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn session_keys(keys: AuraId) -> crab_runtime::SessionKeys {
	crab_runtime::SessionKeys { aura: keys }
}

pub fn development_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "CRAB".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Crab2 Development",
		// ID
		"crab-dev",
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
				2105.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: 2105,
		},
	)
}

pub fn local_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "CRAB".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Crab2 Local Testnet",
		// ID
		"crab_local_testnet",
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
				2105.into(),
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("crab"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: 2105,
		},
	)
}

pub fn config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "CRAB".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 42.into());

	// TODO: update this before final release
	ChainSpec::from_genesis(
		// Name
		"Crab2",
		// ID
		"crab",
		ChainType::Live,
		move || {
			crab_runtime::GenesisConfig {
				// System stuff.
				system: crab_runtime::SystemConfig {
					code: crab_runtime::WASM_BINARY
						.expect("WASM binary was not build, please build it!")
						.to_vec(),
				},
				parachain_system: Default::default(),
				parachain_info: crab_runtime::ParachainInfoConfig { parachain_id: 2105.into() },

				// Monetary stuff.
				balances: Default::default(),
				transaction_payment: Default::default(),
				assets: Default::default(),

				// Consensus stuff.
				collator_selection: crab_runtime::CollatorSelectionConfig {
					invulnerables: vec![array_bytes::hex_n_into_unchecked(ALITH)],
					..Default::default()
				},
				session: crab_runtime::SessionConfig {
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
				polkadot_xcm: crab_runtime::PolkadotXcmConfig {
					safe_xcm_version: Some(SAFE_XCM_VERSION),
				},

				// EVM stuff.
				ethereum: Default::default(),
				evm: Default::default(),
				base_fee: Default::default(),

				// S2S stuff
				bridge_darwinia_grandpa: Default::default(),
				bridge_darwinia_messages: Default::default(),
				darwinia_fee_market: Default::default(),
			}
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("crab"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "kusama".into(), // You MUST set this to the correct network!
			para_id: 2105,
		},
	)
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> crab_runtime::GenesisConfig {
	crab_runtime::GenesisConfig {
		// System stuff.
		system: crab_runtime::SystemConfig { code: crab_runtime::WASM_BINARY.unwrap().to_vec() },
		parachain_system: Default::default(),
		parachain_info: crab_runtime::ParachainInfoConfig { parachain_id: id },

		// Monetary stuff.
		balances: crab_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 100_000_000 * UNIT)).collect(),
		},
		transaction_payment: Default::default(),
		assets: Default::default(),

		// Consensus stuff.
		collator_selection: crab_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: UNIT,
			..Default::default()
		},
		session: crab_runtime::SessionConfig {
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
		polkadot_xcm: crab_runtime::PolkadotXcmConfig { safe_xcm_version: Some(SAFE_XCM_VERSION) },

		// EVM stuff.
		ethereum: Default::default(),
		evm: EvmConfig {
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

		// S2S stuff
		bridge_darwinia_grandpa: Default::default(),
		bridge_darwinia_messages: Default::default(),
		darwinia_fee_market: Default::default(),
	}
}

pub fn genesis_config() -> ChainSpec {
	unimplemented!("TODO")
}
