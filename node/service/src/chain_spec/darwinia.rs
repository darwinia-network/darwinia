// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
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
// --- substrate ---
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainType;
use sc_finality_grandpa::AuthorityId as GrandpaId;
use sc_service::Properties;
use sc_telemetry::TelemetryEndpoints;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_runtime::Perbill;
// --- darwinia ---
use super::{
	get_account_id_from_seed, get_authority_keys_from_seed, testnet_accounts, Extensions,
	DEFAULT_PROTOCOL_ID,
};
use darwinia_primitives::{AccountId, BlockNumber};
use darwinia_runtime::{
	constants::{currency::COIN, time::DAYS},
	*,
};

/// The `ChainSpec parametrised for Darwinia runtime`.
pub type DarwiniaChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

const DARWINIA_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

pub fn darwinia_config() -> Result<DarwiniaChainSpec, String> {
	DarwiniaChainSpec::from_json_bytes(&include_bytes!("../../res/darwinia/darwinia.json")[..])
}

/// Session keys for Darwinia.
pub fn darwinia_session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys {
		babe,
		grandpa,
		im_online,
		authority_discovery,
	}
}

/// Properties for Darwinia.
pub fn darwinia_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 18.into());
	properties.insert("tokenDecimals".into(), vec![9, 9].into());
	properties.insert("tokenSymbol".into(), vec!["RING", "KTON"].into());

	properties
}

pub fn darwinia_build_spec_genesis() -> GenesisConfig {
	const ROOT: &'static str = "0x0a66532a23c418cca12183fee5f6afece770a0bb8725f459d7d1b1b598f91c49";
	const DA_CRABK: &'static str =
		"0x6d6f646c64612f637261626b0000000000000000000000000000000000000000";
	const TEAM_VESTING: &'static str =
		"0x88db6cf10428d2608cd2ca2209971d0227422dc1f53c6ec0848fa610848a6ed3";
	const FOUNDATION_VESTING: &'static str =
		"0x8db5c746c14cf05e182b10576a9ee765265366c3b7fd53c41d43640c97f4a8b8";
	const GENESIS_VALIDATOR_1_STASH: &'static str =
		"0xb4f7f03bebc56ebe96bc52ea5ed3159d45a0ce3a8d7f082983c33ef133274747";
	const GENESIS_VALIDATOR_1_CONTROLLER: &'static str =
		"0x7e450358b1768b8cc1df515292a97ac9f14f3f2ec9705a7352ec70b380c7fa60";
	const GENESIS_VALIDATOR_1_SESSION: &'static str =
		"0x0ae0f956e21c3f0ca9ea9121b41a1c1fc567f6ba6ce8abfed000073bb3352511";
	const GENESIS_VALIDATOR_1_GRANDPA: &'static str =
		"0x14342647be14beb21000d518a326be1e9b01d96ef1415148043e4ae2c726d463";
	const GENESIS_VALIDATOR_2_STASH: &'static str =
		"0xb62d88e3f439fe9b5ea799b27bf7c6db5e795de1784f27b1bc051553499e420f";
	const GENESIS_VALIDATOR_2_CONTROLLER: &'static str =
		"0xb62d88e3f439fe9b5ea799b27bf7c6db5e795de1784f27b1bc051553499e420f";
	const GENESIS_VALIDATOR_2_SESSION: &'static str =
		"0xc8053dc90b1e4f4741c5c9088dcc1ee8758600fe8aa8702c178d91af1d191a17";
	const GENESIS_VALIDATOR_2_GRANDPA: &'static str =
		"0x229af404837dda8416b3f9ef22f4c3a8cc0103cd091bcdeb0d80776e6c3b99f1";

	const TOKEN_REDEEM_ADDRESS: &'static str = "0xea7938985898af7fd945b03b7bc2e405e744e913";
	const DEPOSIT_REDEEM_ADDRESS: &'static str = "0x649fdf6ee483a96e020b889571e93700fbd82d88";
	const RING_TOKEN_ADDRESS: &'static str = "0x9469d013805bffb7d3debe5e7839237e535ec483";
	const KTON_TOKEN_ADDRESS: &'static str = "0x9f284e1337a815fe77d2ff4ae46544645b20c5ff";

	let mut rings = BTreeMap::new();
	let mut ktons = BTreeMap::new();
	let mut swapped_ring_for_crab = 0;
	let mut da_crabk_endowed = false;
	let mut root_endowed = false;
	let mut genesis_validator_1_stash_endowed = false;
	let mut genesis_validator_2_stash_endowed = false;

	// Initialize Crab genesis swap
	for (address, ring) in
		genesis_loader::load_genesis_swap_from_file("node/service/res/darwinia/swapped-cring.json")
			.unwrap()
	{
		match format!("0x{}", address).as_ref() {
			ROOT => root_endowed = true,
			GENESIS_VALIDATOR_1_STASH => genesis_validator_1_stash_endowed = true,
			GENESIS_VALIDATOR_2_STASH => genesis_validator_2_stash_endowed = true,
			_ => (),
		}

		rings
			.entry(array_bytes::hex_into_unchecked(address))
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
		match format!("0x{}", address).as_ref() {
			DA_CRABK => da_crabk_endowed = true,
			_ => (),
		}

		let ring = ring / COIN;

		rings
			.entry(array_bytes::hex_into_unchecked(address))
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
			.entry(array_bytes::hex_into_unchecked(address))
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
			darwinia_session_keys(
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
			darwinia_session_keys(
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
		frame_system: SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
		},
		darwinia_balances_Instance1: BalancesConfig {
			balances: rings.into_iter().collect(),
		},
		darwinia_balances_Instance2: KtonConfig {
			balances: ktons.into_iter().collect(),
		},
		darwinia_staking: StakingConfig {
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
		pallet_session: SessionConfig {
			keys: vec![
				(
					genesis_validator_1.0.clone(),
					genesis_validator_1.0,
					genesis_validator_1.2,
				),
				(
					genesis_validator_2.0.clone(),
					genesis_validator_2.0,
					genesis_validator_2.2,
				),
			],
		},
		pallet_grandpa: Default::default(),
		pallet_im_online: Default::default(),
		pallet_authority_discovery: Default::default(),
		pallet_collective_Instance1: Default::default(),
		pallet_collective_Instance2: Default::default(),
		darwinia_elections_phragmen: Default::default(),
		pallet_membership_Instance1: Default::default(),
		darwinia_vesting: VestingConfig {
			vesting: vec![
				// Team vesting: 1 year period start after 1 year since mainnet lanuch
				(team_vesting, 365 * DAYS, 365 * DAYS, 0),
				// Foundation vesting: 5 years period start when mainnet launch
				(
					foundation_vesting,
					0,
					(5.00_f64 * 365.25_f64) as BlockNumber * DAYS,
					0,
				),
			],
		},
		pallet_sudo: SudoConfig { key: root },
		darwinia_ethereum_backing: EthereumBackingConfig {
			token_redeem_address: array_bytes::hex_into_unchecked(TOKEN_REDEEM_ADDRESS),
			deposit_redeem_address: array_bytes::hex_into_unchecked(DEPOSIT_REDEEM_ADDRESS),
			ring_token_address: array_bytes::hex_into_unchecked(RING_TOKEN_ADDRESS),
			kton_token_address: array_bytes::hex_into_unchecked(KTON_TOKEN_ADDRESS),
			// Los Angeles: 9/24/2020, 7:42:52 PM
			// Berlin :     9/25/2020, 10:42:52 AM
			// Beijing:     9/25/2020, 9:42:52 AM
			// New York :   9/24/2020, 9:42:52 PM
			backed_ring: 1_141_998_248_692_824_029_753_349_753_u128 / COIN + 1,
			backed_kton: 55_760_225_171_204_355_332_737_u128 / COIN + 1,
			..Default::default()
		},
		darwinia_ethereum_relay: EthereumRelayConfig {
			genesis_header_parcel: r#"{
				"header": {
					"difficulty": "0x1a76e148d47e3f",
					"extraData": "0x626565706f6f6c2e6f72675f3622e3",
					"gasLimit": "0xe4c4d7",
					"gasUsed": "0xe49a99",
					"hash": "0xf2d555dc534b52389877dc6c3db074fc40810e3c858949bbb9bfbc907318bdd3",
					"logsBloom": "0x1226e522b431888c52a186a6bc361a1375930081649a046025492940aec1890d111c0120082c0d1117435383000b4930122689d62c22adc01ad27e8a42e12104b0500454200121bd79d6382ea0c65af40235a904046409b8b44c3602ca883503930483040a270970b566100c2009e940e016522b580d845f4f290e5a2d0d63360800b4c09e002ab632fc12c8028869a2973911011360c5483a26216a1d9803b9aadca1893005a8f36603eac4a942058098300280da0c8b2620e387aa12092a04ec13a063718d56c02aa31330279830048e1d1706150bb97666c877b2023221c7173ae13dd9c10080b964b14d208c64108dc32590e95880500e838a0f22129221",
					"miner": "0x99c85bb64564d9ef9a99621301f22c9993cb89e3",
					"mixHash": "0x37690063a5583ebc490c22f1ad1e96d82d5b18c5f69e34aaf048d4f9346bad61",
					"nonce": "0x40b7a2b472de4a3c",
					"number": "0xc5814b",
					"parentHash": "0xaf7ec781faedb3524013da0561afef52aeae79f8f24b5a064d8b6e55c3d54587",
					"receiptsRoot": "0x789dc0e01565ea1e4291c2b5bdd5e930aebc83bc3ab4646bb78d65fe3daad484",
					"sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
					"size": "0x1092c",
					"stateRoot": "0x2836918ac7e0f151d5594fc3276cf6c34cd7b6512d622e72fa393daff72752e1",
					"timestamp": "0x61076b22",
					"totalDifficulty": "0x5ffec465fc78e1f3319",
					"transactions": [],
					"transactionsRoot": "0xdca3e219bda6af76807a78ee412054aba59acc347b209d8db55e49836beb8d50",
					"uncles": []
				},
				"parent_mmr_root": "0xef3b0e8e1c5b782216791ba8ed63982cdbd18287210f697add38085d832b4381"
			}"#.into(),
			dags_merkle_roots_loader: DagsMerkleRootsLoader::from_file(
				"node/service/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		},
		darwinia_tron_backing: TronBackingConfig {
			// Los Angeles: 9/24/2020, 7:42:52 PM
			// Berlin :     9/25/2020, 10:42:52 AM
			// Beijing:     9/25/2020, 9:42:52 AM
			// New York :   9/24/2020, 9:42:52 PM
			backed_ring: 90_403_994_952_547_849_178_882_078_u128 / COIN + 1,
			backed_kton: 1_357_120_581_926_771_954_238_u128 / COIN + 1,
		},
		darwinia_democracy: Default::default(),
	}
}

/// Darwinia config.
pub fn darwinia_build_spec_config() -> DarwiniaChainSpec {
	let boot_nodes = vec![
		"/dns4/g1.p2p.darwinia.network/tcp/30333/p2p/12D3KooWANEQE69Td86QUy68Lim3rZR5mxsMviGYdi14ErzCfdht".parse().unwrap(),
		"/dns4/g2.p2p.darwinia.network/tcp/30333/p2p/12D3KooWBxWFD4zdSd2HQTxXNysJ7s248PsKjKKW4DnyiS47i49D".parse().unwrap()
	];

	DarwiniaChainSpec::from_genesis(
		"Darwinia",
		"darwinia",
		ChainType::Live,
		darwinia_build_spec_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(DARWINIA_TELEMETRY_URL.to_string(), 0)])
				.expect("Darwinia telemetry url is valid; qed"),
		),
		Some(DEFAULT_PROTOCOL_ID),
		Some(darwinia_properties()),
		Default::default(),
	)
}

/// Helper function to create Darwinia GenesisConfig for testing
pub fn darwinia_testnet_genesis(
	root: AccountId,
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	endowed_accounts: Option<Vec<AccountId>>,
) -> GenesisConfig {
	const TOKEN_REDEEM_ADDRESS: &'static str = "0xea7938985898af7fd945b03b7bc2e405e744e913";
	const DEPOSIT_REDEEM_ADDRESS: &'static str = "0x649fdf6ee483a96e020b889571e93700fbd82d88";
	const RING_TOKEN_ADDRESS: &'static str = "0x9469d013805bffb7d3debe5e7839237e535ec483";
	const KTON_TOKEN_ADDRESS: &'static str = "0x9f284e1337a815fe77d2ff4ae46544645b20c5ff";

	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(testnet_accounts);

	GenesisConfig {
		frame_system: SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
		},
		darwinia_balances_Instance1: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 56))
				.collect(),
		},
		darwinia_balances_Instance2: KtonConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 56))
				.collect(),
		},
		darwinia_staking: StakingConfig {
			minimum_validator_count: 1,
			validator_count: 15,
			stakers: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0, x.1, 1 << 56, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().cloned().map(|x| x.0).collect(),
			force_era: Forcing::ForceAlways,
			slash_reward_fraction: Perbill::from_percent(10),
			payout_fraction: Perbill::from_percent(50),
			..Default::default()
		},
		pallet_session: SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0.clone(), x.0, darwinia_session_keys(x.2, x.3, x.4, x.5)))
				.collect(),
		},
		pallet_grandpa: Default::default(),
		pallet_im_online: Default::default(),
		pallet_authority_discovery: Default::default(),
		pallet_collective_Instance1: Default::default(),
		pallet_collective_Instance2: Default::default(),
		darwinia_elections_phragmen: Default::default(),
		pallet_membership_Instance1: Default::default(),
		darwinia_vesting: Default::default(),
		pallet_sudo: SudoConfig { key: root },
		darwinia_ethereum_backing: EthereumBackingConfig {
			token_redeem_address: array_bytes::hex_into_unchecked(TOKEN_REDEEM_ADDRESS),
			deposit_redeem_address: array_bytes::hex_into_unchecked(DEPOSIT_REDEEM_ADDRESS),
			ring_token_address: array_bytes::hex_into_unchecked(RING_TOKEN_ADDRESS),
			kton_token_address: array_bytes::hex_into_unchecked(KTON_TOKEN_ADDRESS),
			backed_ring: 1 << 56,
			backed_kton: 1 << 56,
			..Default::default()
		},
		darwinia_ethereum_relay: EthereumRelayConfig {
			genesis_header_parcel: r#"{
				"header": {
					"difficulty": "0x1a76e148d47e3f",
					"extraData": "0x626565706f6f6c2e6f72675f3622e3",
					"gasLimit": "0xe4c4d7",
					"gasUsed": "0xe49a99",
					"hash": "0xf2d555dc534b52389877dc6c3db074fc40810e3c858949bbb9bfbc907318bdd3",
					"logsBloom": "0x1226e522b431888c52a186a6bc361a1375930081649a046025492940aec1890d111c0120082c0d1117435383000b4930122689d62c22adc01ad27e8a42e12104b0500454200121bd79d6382ea0c65af40235a904046409b8b44c3602ca883503930483040a270970b566100c2009e940e016522b580d845f4f290e5a2d0d63360800b4c09e002ab632fc12c8028869a2973911011360c5483a26216a1d9803b9aadca1893005a8f36603eac4a942058098300280da0c8b2620e387aa12092a04ec13a063718d56c02aa31330279830048e1d1706150bb97666c877b2023221c7173ae13dd9c10080b964b14d208c64108dc32590e95880500e838a0f22129221",
					"miner": "0x99c85bb64564d9ef9a99621301f22c9993cb89e3",
					"mixHash": "0x37690063a5583ebc490c22f1ad1e96d82d5b18c5f69e34aaf048d4f9346bad61",
					"nonce": "0x40b7a2b472de4a3c",
					"number": "0xc5814b",
					"parentHash": "0xaf7ec781faedb3524013da0561afef52aeae79f8f24b5a064d8b6e55c3d54587",
					"receiptsRoot": "0x789dc0e01565ea1e4291c2b5bdd5e930aebc83bc3ab4646bb78d65fe3daad484",
					"sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
					"size": "0x1092c",
					"stateRoot": "0x2836918ac7e0f151d5594fc3276cf6c34cd7b6512d622e72fa393daff72752e1",
					"timestamp": "0x61076b22",
					"totalDifficulty": "0x5ffec465fc78e1f3319",
					"transactions": [],
					"transactionsRoot": "0xdca3e219bda6af76807a78ee412054aba59acc347b209d8db55e49836beb8d50",
					"uncles": []
				},
				"parent_mmr_root": "0xef3b0e8e1c5b782216791ba8ed63982cdbd18287210f697add38085d832b4381"
			}"#.into(),
			dags_merkle_roots_loader: DagsMerkleRootsLoader::from_file(
				"node/service/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		},
		darwinia_tron_backing: TronBackingConfig {
			backed_ring: 1 << 56,
			backed_kton: 1 << 56,
		},
		darwinia_democracy: Default::default(),
	}
}

/// Darwinia development config (single validator Alice)
pub fn darwinia_development_config() -> DarwiniaChainSpec {
	fn darwinia_development_genesis() -> GenesisConfig {
		darwinia_testnet_genesis(
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			vec![get_authority_keys_from_seed("Alice")],
			Some(vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			]),
		)
	}

	DarwiniaChainSpec::from_genesis(
		"Development",
		"darwinia_dev",
		ChainType::Development,
		darwinia_development_genesis,
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		Some(darwinia_properties()),
		Default::default(),
	)
}
