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

// --- std ---
use std::collections::BTreeMap;
// --- paritytech ---
use fp_evm::GenesisAccount;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainType;
use sc_finality_grandpa::AuthorityId as GrandpaId;
use sc_service::Properties;
use sc_telemetry::TelemetryEndpoints;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_runtime::Perbill;
// --- darwinia-network ---
use super::*;
use darwinia_primitives::{AccountId, BlockNumber, COIN, DAYS};
use darwinia_runtime::*;

/// The `ChainSpec parametrised for Darwinia runtime`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

const DARWINIA_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Session keys for Darwinia.
pub fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { babe, grandpa, im_online, authority_discovery }
}

/// Properties for Darwinia.
pub fn properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 18.into());
	properties.insert("tokenDecimals".into(), vec![9, 9].into());
	properties.insert("tokenSymbol".into(), vec!["RING", "KTON"].into());

	properties
}

pub fn config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../res/darwinia/darwinia.json")[..])
}

/// Darwinia config.
pub fn genesis_config() -> ChainSpec {
	fn genesis() -> GenesisConfig {
		const ROOT: &str = "0x0a66532a23c418cca12183fee5f6afece770a0bb8725f459d7d1b1b598f91c49";
		const DA_CRABK: &str = "0x6d6f646c64612f637261626b0000000000000000000000000000000000000000";
		const TEAM_VESTING: &str =
			"0x88db6cf10428d2608cd2ca2209971d0227422dc1f53c6ec0848fa610848a6ed3";
		const FOUNDATION_VESTING: &str =
			"0x8db5c746c14cf05e182b10576a9ee765265366c3b7fd53c41d43640c97f4a8b8";
		const GENESIS_VALIDATOR_1_STASH: &str =
			"0xb4f7f03bebc56ebe96bc52ea5ed3159d45a0ce3a8d7f082983c33ef133274747";
		const GENESIS_VALIDATOR_1_CONTROLLER: &str =
			"0x7e450358b1768b8cc1df515292a97ac9f14f3f2ec9705a7352ec70b380c7fa60";
		const GENESIS_VALIDATOR_1_SESSION: &str =
			"0x0ae0f956e21c3f0ca9ea9121b41a1c1fc567f6ba6ce8abfed000073bb3352511";
		const GENESIS_VALIDATOR_1_GRANDPA: &str =
			"0x14342647be14beb21000d518a326be1e9b01d96ef1415148043e4ae2c726d463";
		const GENESIS_VALIDATOR_2_STASH: &str =
			"0xb62d88e3f439fe9b5ea799b27bf7c6db5e795de1784f27b1bc051553499e420f";
		const GENESIS_VALIDATOR_2_CONTROLLER: &str =
			"0xb62d88e3f439fe9b5ea799b27bf7c6db5e795de1784f27b1bc051553499e420f";
		const GENESIS_VALIDATOR_2_SESSION: &str =
			"0xc8053dc90b1e4f4741c5c9088dcc1ee8758600fe8aa8702c178d91af1d191a17";
		const GENESIS_VALIDATOR_2_GRANDPA: &str =
			"0x229af404837dda8416b3f9ef22f4c3a8cc0103cd091bcdeb0d80776e6c3b99f1";

		let mut rings = BTreeMap::new();
		let mut ktons = BTreeMap::new();
		let mut swapped_ring_for_crab = 0;
		let mut da_crabk_endowed = false;
		let mut root_endowed = false;
		let mut genesis_validator_1_stash_endowed = false;
		let mut genesis_validator_2_stash_endowed = false;

		// Initialize Crab genesis swap
		for (address, ring) in genesis_loader::load_genesis_swap_from_file(
			"node/service/res/darwinia/swapped-crab.json",
		)
		.unwrap()
		{
			match format!("0x{}", address).as_ref() {
				ROOT => root_endowed = true,
				GENESIS_VALIDATOR_1_STASH => genesis_validator_1_stash_endowed = true,
				GENESIS_VALIDATOR_2_STASH => genesis_validator_2_stash_endowed = true,
				_ => (),
			}

			rings
				.entry(array_bytes::hex_into_unchecked(&address))
				.and_modify(|ring_| *ring_ += ring)
				.or_insert(ring);

			swapped_ring_for_crab += ring;
		}

		// Initialize Ethereum/Tron genesis swap (RING)
		for (address, ring) in [
			genesis_loader::load_genesis_swap_from_file(
				"node/service/res/darwinia/swapped-erc20-ring.json",
			)
			.unwrap(),
			genesis_loader::load_genesis_swap_from_file(
				"node/service/res/darwinia/swapped-trc20-ring.json",
			)
			.unwrap(),
		]
		.concat()
		{
			if let DA_CRABK = format!("0x{}", address).as_ref() {
				da_crabk_endowed = true
			}

			let ring = ring / COIN;

			rings
				.entry(array_bytes::hex_into_unchecked(&address))
				.and_modify(|ring_| *ring_ += ring)
				.or_insert(ring);
		}
		// Initialize Ethereum/Tron genesis swap (KTON)
		for (address, kton) in [
			genesis_loader::load_genesis_swap_from_file(
				"node/service/res/darwinia/swapped-erc20-kton.json",
			)
			.unwrap(),
			genesis_loader::load_genesis_swap_from_file(
				"node/service/res/darwinia/swapped-trc20-kton.json",
			)
			.unwrap(),
		]
		.concat()
		{
			let kton = kton / COIN;

			ktons
				.entry(array_bytes::hex_into_unchecked(&address))
				.and_modify(|kton_| *kton_ += kton)
				.or_insert(kton);
		}

		// Important account MUST be initialized
		assert!(da_crabk_endowed);
		assert!(root_endowed);
		assert!(genesis_validator_1_stash_endowed);
		assert!(genesis_validator_2_stash_endowed);

		let root: AccountId = array_bytes::hex_into_unchecked(ROOT);
		let da_crabk: AccountId = array_bytes::hex_into_unchecked(DA_CRABK);
		let team_vesting: AccountId = array_bytes::hex_into_unchecked(TEAM_VESTING);
		let foundation_vesting: AccountId = array_bytes::hex_into_unchecked(FOUNDATION_VESTING);
		let genesis_validator_1: (AccountId, AccountId, SessionKeys) = {
			let stash = array_bytes::hex_into_unchecked(GENESIS_VALIDATOR_1_STASH);
			let controller = array_bytes::hex_into_unchecked(GENESIS_VALIDATOR_1_CONTROLLER);
			let session = array_bytes::hex2array_unchecked(GENESIS_VALIDATOR_1_SESSION);
			let grandpa = array_bytes::hex2array_unchecked(GENESIS_VALIDATOR_1_GRANDPA);

			(
				stash,
				controller,
				session_keys(
					session.unchecked_into(),
					grandpa.unchecked_into(),
					session.unchecked_into(),
					session.unchecked_into(),
				),
			)
		};
		let genesis_validator_2: (AccountId, AccountId, SessionKeys) = {
			let stash = array_bytes::hex_into_unchecked(GENESIS_VALIDATOR_2_STASH);
			let controller = array_bytes::hex_into_unchecked(GENESIS_VALIDATOR_2_CONTROLLER);
			let session = array_bytes::hex2array_unchecked(GENESIS_VALIDATOR_2_SESSION);
			let grandpa = array_bytes::hex2array_unchecked(GENESIS_VALIDATOR_2_GRANDPA);

			(
				stash,
				controller,
				session_keys(
					session.unchecked_into(),
					grandpa.unchecked_into(),
					session.unchecked_into(),
					session.unchecked_into(),
				),
			)
		};

		// Crab backing: 40M - claimed
		*rings.get_mut(&da_crabk).unwrap() -= swapped_ring_for_crab;
		// Team vesting: 300M
		rings
			.entry(team_vesting.clone())
			.and_modify(|ring| *ring += 300_000_000 * COIN)
			.or_insert(300_000_000 * COIN);
		// Foundation vesting: 400M
		rings
			.entry(foundation_vesting.clone())
			.and_modify(|ring| *ring += 400_000_000 * COIN)
			.or_insert(400_000_000 * COIN);

		GenesisConfig {
			system: SystemConfig { code: wasm_binary_unwrap().to_vec() },
			babe: BabeConfig { authorities: vec![], epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG) },
			balances: BalancesConfig { balances: rings.into_iter().collect() },
			kton: KtonConfig { balances: ktons.into_iter().collect() },
			staking: StakingConfig {
				minimum_validator_count: 1,
				validator_count: 15,
				stakers: vec![
					(
						genesis_validator_1.0.clone(),
						genesis_validator_1.1.clone(),
						COIN,
						StakerStatus::Validator,
					),
					(
						genesis_validator_2.0.clone(),
						genesis_validator_2.1.clone(),
						COIN,
						StakerStatus::Validator,
					),
				],
				force_era: Forcing::ForceNew,
				slash_reward_fraction: Perbill::from_percent(10),
				payout_fraction: Perbill::from_percent(50),
				..Default::default()
			},
			session: SessionConfig {
				keys: vec![
					(genesis_validator_1.0.clone(), genesis_validator_1.0, genesis_validator_1.2),
					(genesis_validator_2.0.clone(), genesis_validator_2.0, genesis_validator_2.2),
				],
			},
			grandpa: Default::default(),
			im_online: Default::default(),
			authority_discovery: Default::default(),
			message_gadget: Default::default(),
			ecdsa_authority: EcdsaAuthorityConfig {
				authorities: [
					"0x953d65e6054b7eb1629f996238c0aa9b4e2dbfe9",
					"0x7c9b3d4cfc78c681b7460acde2801452aef073a9",
					"0x717c38fd5fdecb1b105a470f861b33a6b0f9f7b8",
					"0x3e25247CfF03F99a7D83b28F207112234feE73a6",
				]
				.iter()
				.filter_map(|s| array_bytes::hex_into(s).ok())
				.collect(),
			},
			democracy: Default::default(),
			council: Default::default(),
			technical_committee: Default::default(),
			phragmen_election: Default::default(),
			technical_membership: Default::default(),
			treasury: Default::default(),
			kton_treasury: Default::default(),
			sudo: SudoConfig { key: root },
			vesting: VestingConfig {
				vesting: vec![
					// Team vesting: 1 year period start after 1 year since mainnet launch
					(team_vesting, 365 * DAYS, 365 * DAYS, 0),
					// Foundation vesting: 5 years period start when mainnet launch
					(foundation_vesting, 0, (5.00_f64 * 365.25_f64) as BlockNumber * DAYS, 0),
				],
			},
			tron_backing: TronBackingConfig {
				// Los Angeles: 9/24/2020, 7:42:52 PM
				// Berlin :     9/25/2020, 10:42:52 AM
				// Beijing:     9/25/2020, 9:42:52 AM
				// New York :   9/24/2020, 9:42:52 PM
				backed_ring: 90_403_994_952_547_849_178_882_078_u128 / COIN + 1,
				backed_kton: 1_357_120_581_926_771_954_238_u128 / COIN + 1,
			},
			to_crab_backing: Default::default(),
			evm: EVMConfig { accounts: BTreeMap::new() },
			ethereum: Default::default(),
			base_fee: Default::default(),
		}
	}

	let boot_nodes = vec![
		"/dns4/g1.p2p.darwinia.network/tcp/30333/p2p/12D3KooWANEQE69Td86QUy68Lim3rZR5mxsMviGYdi14ErzCfdht".parse().unwrap(),
		"/dns4/g2.p2p.darwinia.network/tcp/30333/p2p/12D3KooWBxWFD4zdSd2HQTxXNysJ7s248PsKjKKW4DnyiS47i49D".parse().unwrap(),
		"/dns4/g1.p2p.darwinia.network/tcp/30334/ws/p2p/12D3KooWANEQE69Td86QUy68Lim3rZR5mxsMviGYdi14ErzCfdht".parse().unwrap(),
		"/dns4/g2.p2p.darwinia.network/tcp/30334/ws/p2p/12D3KooWBxWFD4zdSd2HQTxXNysJ7s248PsKjKKW4DnyiS47i49D".parse().unwrap(),
	];

	ChainSpec::from_genesis(
		"Darwinia",
		"darwinia",
		ChainType::Live,
		genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(DARWINIA_TELEMETRY_URL.to_string(), 0)])
				.expect("Darwinia telemetry url is valid; qed"),
		),
		Some(DEFAULT_PROTOCOL_ID),
		Some(properties()),
		Default::default(),
	)
}

/// Darwinia development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	fn genesis() -> GenesisConfig {
		let root = get_account_id_from_seed::<sr25519::Public>("Alice");
		let initial_authorities = vec![get_authority_keys_from_seed("Alice")];
		let endowed_accounts = vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		];
		let evm_accounts = BTreeMap::from_iter([(
			array_bytes::hex_into_unchecked("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b"),
			GenesisAccount {
				balance: (123_456_789_000_000_000_000_090 as Balance).into(),
				code: Default::default(),
				nonce: Default::default(),
				storage: Default::default(),
			},
		)]);

		GenesisConfig {
			system: SystemConfig { code: wasm_binary_unwrap().to_vec() },
			babe: BabeConfig { authorities: vec![], epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG) },
			balances: BalancesConfig {
				balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 56)).collect(),
			},
			kton: KtonConfig {
				balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 56)).collect(),
			},
			staking: StakingConfig {
				minimum_validator_count: 1,
				validator_count: 15,
				stakers: initial_authorities
					.iter()
					.cloned()
					.map(|x| (x.0, x.1, COIN, StakerStatus::Validator))
					.collect(),
				invulnerables: initial_authorities.iter().cloned().map(|x| x.0).collect(),
				force_era: Forcing::ForceNew,
				slash_reward_fraction: Perbill::from_percent(10),
				payout_fraction: Perbill::from_percent(50),
				..Default::default()
			},
			session: SessionConfig {
				keys: initial_authorities
					.iter()
					.cloned()
					.map(|x| (x.0.clone(), x.0, session_keys(x.2, x.3, x.4, x.5)))
					.collect(),
			},
			grandpa: Default::default(),
			im_online: Default::default(),
			authority_discovery: Default::default(),
			message_gadget: Default::default(),
			ecdsa_authority: EcdsaAuthorityConfig {
				authorities: vec![array_bytes::hex_into_unchecked(
					"0x68898db1012808808c903f390909c52d9f706749",
				)],
			},
			democracy: Default::default(),
			council: Default::default(),
			technical_committee: Default::default(),
			phragmen_election: Default::default(),
			technical_membership: Default::default(),
			treasury: Default::default(),
			kton_treasury: Default::default(),
			sudo: SudoConfig { key: root },
			vesting: Default::default(),
			tron_backing: TronBackingConfig { backed_ring: 1 << 56, backed_kton: 1 << 56 },
			to_crab_backing: Default::default(),
			evm: EVMConfig { accounts: evm_accounts },
			ethereum: Default::default(),
			base_fee: Default::default(),
		}
	}

	ChainSpec::from_genesis(
		"Darwinia Development Testnet",
		"darwinia_dev",
		ChainType::Development,
		genesis,
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		Some(properties()),
		Default::default(),
	)
}
