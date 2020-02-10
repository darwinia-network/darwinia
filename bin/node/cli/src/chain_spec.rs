// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Substrate chain configurations.

use grandpa_primitives::AuthorityId as GrandpaId;
use hex_literal::hex;
use node_runtime::constants::currency::*;
use node_runtime::Block;
use node_runtime::{
	AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, ContractsConfig, GrandpaConfig, ImOnlineConfig,
	IndicesConfig, KtonConfig, SessionConfig, SessionKeys, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
	WASM_BINARY,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::Properties;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

pub use node_primitives::{AccountId, Balance, Signature};
pub use node_runtime::GenesisConfig;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client::ForkBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::ChainSpec<GenesisConfig, Extensions>;
/// IceFrog testnet generator
pub fn icefrog_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/icefrog.json")[..])
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys {
		grandpa,
		babe,
		im_online,
		authority_discovery,
	}
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(
	seed: &str,
) -> (
	AccountId,
	AccountId,
	GrandpaId,
	BabeId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

/// Helper function to create GenesisConfig for darwinia
/// is_testnet: under test net we will use Alice & Bob as seed to generate keys,
///             but in production enviroment, these accounts will use preset keys
pub fn darwinia_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	enable_println: bool,
	is_testnet: bool,
) -> GenesisConfig {
	let eth_relay_authorities: Vec<AccountId> = if is_testnet {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
		]
	} else {
		vec![initial_authorities[0].clone().1, initial_authorities[1].clone().1]
	};

	const RING_ENDOWMENT: Balance = 20_000_000 * COIN;
	const KTON_ENDOWMENT: Balance = 10 * COIN;
	const STASH: Balance = 1000 * COIN;

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_indices: Some(IndicesConfig {
			ids: endowed_accounts
				.iter()
				.cloned()
				.chain(initial_authorities.iter().map(|x| x.0.clone()))
				.collect::<Vec<_>>(),
		}),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		}),
		//		pallet_democracy: Some(DemocracyConfig::default()),
		//		pallet_collective_Instance1: Some(CouncilConfig {
		//			members: endowed_accounts.iter().cloned().collect::<Vec<_>>()[..(num_endowed_accounts + 1) / 2].to_vec(),
		//			phantom: Default::default(),
		//		}),
		//		pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
		//			members: endowed_accounts.iter().cloned().collect::<Vec<_>>()[..(num_endowed_accounts + 1) / 2].to_vec(),
		//			phantom: Default::default(),
		//		}),
		pallet_contracts: Some(ContractsConfig {
			current_schedule: pallet_contracts::Schedule {
				enable_println, // this should only be enabled on development chains
				..Default::default()
			},
			gas_price: 1 * MILLI,
		}),
		pallet_sudo: Some(SudoConfig { key: root_key }),
		pallet_babe: Some(BabeConfig { authorities: vec![] }),
		pallet_im_online: Some(ImOnlineConfig { keys: vec![] }),
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),
		pallet_grandpa: Some(GrandpaConfig { authorities: vec![] }),
		//		pallet_membership_Instance1: Some(Default::default()),
		//		pallet_treasury: Some(Default::default()),
		pallet_ring: Some(BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, RING_ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
			vesting: vec![],
		}),
		pallet_kton: Some(KtonConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, KTON_ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
			vesting: vec![],
		}),
		pallet_staking: Some(StakingConfig {
			current_era: 0,
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		}),
	}
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	fn staging_testnet_config_genesis() -> GenesisConfig {
		// stash, controller, session-key
		// generated with secret:
		// for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/fir/$j/$i; done; done
		// and
		// for i in 1 2 3 4 ; do for j in session; do subkey --ed25519 inspect "$secret"//fir//$j//$i; done; done

		let initial_authorities: Vec<(
			AccountId,
			AccountId,
			GrandpaId,
			BabeId,
			ImOnlineId,
			AuthorityDiscoveryId,
		)> = vec![
			(
				// 5Fbsd6WXDGiLTxunqeK5BATNiocfCqu9bS1yArVjCgeBLkVy
				hex!["9c7a2ee14e565db0c69f78c7b4cd839fbf52b607d867e9e9c5a79042898a0d12"].into(),
				// 5EnCiV7wSHeNhjW3FSUwiJNkcc2SBkPLn5Nj93FmbLtBjQUq
				hex!["781ead1e2fa9ccb74b44c19d29cb2a7a4b5be3972927ae98cd3877523976a276"].into(),
				// 5Fb9ayurnxnaXj56CjmyQLBiadfRCqUbL2VWNbbe1nZU6wiC
				hex!["9becad03e6dcac03cee07edebca5475314861492cdfc96a2144a67bbe9699332"].unchecked_into(),
				// 5EZaeQ8djPcq9pheJUhgerXQZt9YaHnMJpiHMRhwQeinqUW8
				hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"].unchecked_into(),
				// 5EZaeQ8djPcq9pheJUhgerXQZt9YaHnMJpiHMRhwQeinqUW8
				hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"].unchecked_into(),
				// 5EZaeQ8djPcq9pheJUhgerXQZt9YaHnMJpiHMRhwQeinqUW8
				hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"].unchecked_into(),
			),
			(
				// 5ERawXCzCWkjVq3xz1W5KGNtVx2VdefvZ62Bw1FEuZW4Vny2
				hex!["68655684472b743e456907b398d3a44c113f189e56d1bbfd55e889e295dfde78"].into(),
				// 5Gc4vr42hH1uDZc93Nayk5G7i687bAQdHHc9unLuyeawHipF
				hex!["c8dc79e36b29395413399edaec3e20fcca7205fb19776ed8ddb25d6f427ec40e"].into(),
				// 5EockCXN6YkiNCDjpqqnbcqd4ad35nU4RmA1ikM4YeRN4WcE
				hex!["7932cff431e748892fa48e10c63c17d30f80ca42e4de3921e641249cd7fa3c2f"].unchecked_into(),
				// 5DhLtiaQd1L1LU9jaNeeu9HJkP6eyg3BwXA7iNMzKm7qqruQ
				hex!["482dbd7297a39fa145c570552249c2ca9dd47e281f0c500c971b59c9dcdcd82e"].unchecked_into(),
				// 5DhLtiaQd1L1LU9jaNeeu9HJkP6eyg3BwXA7iNMzKm7qqruQ
				hex!["482dbd7297a39fa145c570552249c2ca9dd47e281f0c500c971b59c9dcdcd82e"].unchecked_into(),
				// 5DhLtiaQd1L1LU9jaNeeu9HJkP6eyg3BwXA7iNMzKm7qqruQ
				hex!["482dbd7297a39fa145c570552249c2ca9dd47e281f0c500c971b59c9dcdcd82e"].unchecked_into(),
			),
			(
				// 5DyVtKWPidondEu8iHZgi6Ffv9yrJJ1NDNLom3X9cTDi98qp
				hex!["547ff0ab649283a7ae01dbc2eb73932eba2fb09075e9485ff369082a2ff38d65"].into(),
				// 5FeD54vGVNpFX3PndHPXJ2MDakc462vBCD5mgtWRnWYCpZU9
				hex!["9e42241d7cd91d001773b0b616d523dd80e13c6c2cab860b1234ef1b9ffc1526"].into(),
				// 5E1jLYfLdUQKrFrtqoKgFrRvxM3oQPMbf6DfcsrugZZ5Bn8d
				hex!["5633b70b80a6c8bb16270f82cca6d56b27ed7b76c8fd5af2986a25a4788ce440"].unchecked_into(),
				// 5DhKqkHRkndJu8vq7pi2Q5S3DfftWJHGxbEUNH43b46qNspH
				hex!["482a3389a6cf42d8ed83888cfd920fec738ea30f97e44699ada7323f08c3380a"].unchecked_into(),
				// 5DhKqkHRkndJu8vq7pi2Q5S3DfftWJHGxbEUNH43b46qNspH
				hex!["482a3389a6cf42d8ed83888cfd920fec738ea30f97e44699ada7323f08c3380a"].unchecked_into(),
				// 5DhKqkHRkndJu8vq7pi2Q5S3DfftWJHGxbEUNH43b46qNspH
				hex!["482a3389a6cf42d8ed83888cfd920fec738ea30f97e44699ada7323f08c3380a"].unchecked_into(),
			),
			(
				// 5HYZnKWe5FVZQ33ZRJK1rG3WaLMztxWrrNDb1JRwaHHVWyP9
				hex!["f26cdb14b5aec7b2789fd5ca80f979cef3761897ae1f37ffb3e154cbcc1c2663"].into(),
				// 5EPQdAQ39WQNLCRjWsCk5jErsCitHiY5ZmjfWzzbXDoAoYbn
				hex!["66bc1e5d275da50b72b15de072a2468a5ad414919ca9054d2695767cf650012f"].into(),
				// 5DMa31Hd5u1dwoRKgC4uvqyrdK45RHv3CpwvpUC1EzuwDit4
				hex!["3919132b851ef0fd2dae42a7e734fe547af5a6b809006100f48944d7fae8e8ef"].unchecked_into(),
				// 5C4vDQxA8LTck2xJEy4Yg1hM9qjDt4LvTQaMo4Y8ne43aU6x
				hex!["00299981a2b92f878baaf5dbeba5c18d4e70f2a1fcd9c61b32ea18daf38f4378"].unchecked_into(),
				// 5C4vDQxA8LTck2xJEy4Yg1hM9qjDt4LvTQaMo4Y8ne43aU6x
				hex!["00299981a2b92f878baaf5dbeba5c18d4e70f2a1fcd9c61b32ea18daf38f4378"].unchecked_into(),
				// 5C4vDQxA8LTck2xJEy4Yg1hM9qjDt4LvTQaMo4Y8ne43aU6x
				hex!["00299981a2b92f878baaf5dbeba5c18d4e70f2a1fcd9c61b32ea18daf38f4378"].unchecked_into(),
			),
		];

		// generated with secret: subkey inspect "$secret"/fir
		let root_key: AccountId = hex![
			// 5Ff3iXP75ruzroPWRP2FYBHWnmGGBSb63857BgnzCoXNxfPo
			"9ee5e5bdc0ec239eb164f865ecc345ce4c88e76ee002e0f7e318097347471809"
		]
		.into();

		let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

		darwinia_genesis(initial_authorities, root_key, endowed_accounts, false, true)
	}

	let boot_nodes = vec![];
	ChainSpec::from_genesis(
		"Staging Testnet",
		"staging_testnet",
		staging_testnet_config_genesis,
		boot_nodes,
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),
		None,
		None,
		Default::default(),
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	fn development_config_genesis() -> GenesisConfig {
		darwinia_genesis(
			vec![get_authority_keys_from_seed("Alice")],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
			],
			true,
			true,
		)
	}

	ChainSpec::from_genesis(
		"Development",
		"dev",
		development_config_genesis,
		vec![],
		None,
		None,
		None,
		Default::default(),
	)
}

/// IceFrog local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	fn icefrog_config_genesis() -> GenesisConfig {
		darwinia_genesis(
			vec![
				get_authority_keys_from_seed("Alice"),
				get_authority_keys_from_seed("Bob"),
			],
			hex!["a60837b2782f7ffd23e95cd26d1aa8d493b8badc6636234ccd44db03c41fcc6c"].into(), // 5FpQFHfKd1xQ9HLZLQoG1JAQSCJoUEVBELnKsKNcuRLZejJR
			vec![
				hex!["a60837b2782f7ffd23e95cd26d1aa8d493b8badc6636234ccd44db03c41fcc6c"].into(),
				hex!["f29311a581558ded67b8bfd097e614ce8135f777e29777d07ec501adb0ddab08"].into(),
				hex!["1098e3bf7b351d6210c61b05edefb3a2b88c9611db26fbed2c7136b6d8f9c90f"].into(),
				hex!["f252bc67e45acc9b3852a0ef84ddfce6c9cef25193617ef1421c460ecc2c746f"].into(),
				hex!["90ce56f84328b180fc55146709aa7038c18efd58f1f247410be0b1ddc612df27"].into(),
				hex!["4ca516c4b95488d0e6e9810a429a010b5716168d777c6b1399d3ed61cce1715c"].into(),
				hex!["e28573bb4d9233c799defe8f85fa80a66b43d47f4c1aef64bb8fffde1ecf8606"].into(),
				hex!["20e2455350cbe36631e82ce9b12152f98a3738cb763e46e65d1a253806a26d1a"].into(),
				hex!["9eccaca8a35f0659aed4df45455a855bcb3e7bff7bfc9d672b676bbb78988f0d"].into(),
				hex!["98dba2d3252825f4cd1141ca4f41ea201a22b4e129a6c7253cea546dbb20e442"].into(),
			],
			true,
			true,
		)
	}

	ChainSpec::from_genesis(
		"Darwinia IceFrog Testnet",
		"icefrog_testnet",
		icefrog_config_genesis,
		vec![],
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),
		Some("DAR"),
		{
			let mut properties = Properties::new();

			properties.insert("ss58Format".into(), 42.into());

			properties.insert("tokenDecimals".into(), 9.into());
			properties.insert("tokenSymbol".into(), "IRING".into());
			properties.insert("ktonTokenDecimals".into(), 9.into());
			properties.insert("ktonTokenSymbol".into(), "IKTON".into());

			Some(properties)
		},
		Default::default(),
	)
}

/// IceFrog testnet config generator
pub fn gen_icefrog_testnet_config() -> ChainSpec {
	fn icefrog_config_genesis() -> GenesisConfig {
		darwinia_genesis(
			vec![
				(
					hex!["be3fd892bf0e2b33dbfcf298c99a9f71e631a57af6c017dc5ac078c5d5b3494b"].into(), //stash
					hex!["70bf51d123581d6e51af70b342cac75ae0a0fc71d1a8d388719139af9c042b18"].into(),
					get_from_seed::<GrandpaId>("Alice"),
					get_from_seed::<BabeId>("Alice"),
					get_from_seed::<ImOnlineId>("Alice"),
					get_from_seed::<AuthorityDiscoveryId>("Alice"),
				),
				(
					hex!["e2f560c01a2d8e98d313d6799185c28a39e10896332b56304ff46392f585024c"].into(), //stash
					hex!["94c51178449c09eec77918ea951fa3244f7b841eea1dd1489d2b5f2a53f8840f"].into(),
					get_from_seed::<GrandpaId>("Bob"),
					get_from_seed::<BabeId>("Bob"),
					get_from_seed::<ImOnlineId>("Bob"),
					get_from_seed::<AuthorityDiscoveryId>("Bob"),
				),
			],
			hex!["a60837b2782f7ffd23e95cd26d1aa8d493b8badc6636234ccd44db03c41fcc6c"].into(),
			vec![
				hex!["a60837b2782f7ffd23e95cd26d1aa8d493b8badc6636234ccd44db03c41fcc6c"].into(),
				hex!["f29311a581558ded67b8bfd097e614ce8135f777e29777d07ec501adb0ddab08"].into(),
				hex!["1098e3bf7b351d6210c61b05edefb3a2b88c9611db26fbed2c7136b6d8f9c90f"].into(),
				hex!["f252bc67e45acc9b3852a0ef84ddfce6c9cef25193617ef1421c460ecc2c746f"].into(),
				hex!["90ce56f84328b180fc55146709aa7038c18efd58f1f247410be0b1ddc612df27"].into(),
				hex!["4ca516c4b95488d0e6e9810a429a010b5716168d777c6b1399d3ed61cce1715c"].into(),
				hex!["e28573bb4d9233c799defe8f85fa80a66b43d47f4c1aef64bb8fffde1ecf8606"].into(),
				hex!["20e2455350cbe36631e82ce9b12152f98a3738cb763e46e65d1a253806a26d1a"].into(),
				hex!["9eccaca8a35f0659aed4df45455a855bcb3e7bff7bfc9d672b676bbb78988f0d"].into(),
				hex!["98dba2d3252825f4cd1141ca4f41ea201a22b4e129a6c7253cea546dbb20e442"].into(),
			],
			true,
			false,
		)
	}

	ChainSpec::from_genesis(
		"Darwinia IceFrog Testnet",
		"icefrog_testnet",
		icefrog_config_genesis,
		vec![],
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),
		Some("DAR"),
		{
			let mut properties = Properties::new();

			properties.insert("ss58Format".into(), 42.into());
			properties.insert("tokenDecimals".into(), 9.into());
			properties.insert("tokenSymbol".into(), "IRING".into());
			properties.insert("ktonTokenDecimals".into(), 9.into());
			properties.insert("ktonTokenSymbol".into(), "IKTON".into());

			Some(properties)
		},
		Default::default(),
	)
}
