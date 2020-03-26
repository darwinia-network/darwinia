//! Darwinia chain configurations.

pub use node_primitives::{AccountId, Balance, Signature};
pub use node_runtime::GenesisConfig;

use std::{env, fs::File, path::Path};

use grandpa_primitives::AuthorityId as GrandpaId;
use hex_literal::hex;
use node_runtime::{
	constants::currency::*, AuthorityDiscoveryConfig, BabeConfig, BalancesConfig as RingConfig, Block, ClaimsConfig,
	ContractsConfig, CouncilConfig, EthBackingConfig, EthRelayConfig, GrandpaConfig, ImOnlineConfig, IndicesConfig,
	KtonConfig, SessionConfig, SessionKeys, SocietyConfig, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, WASM_BINARY,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::Properties;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client::BadBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;
/// Crab network generator
pub fn crab_network_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/crab.json")[..])
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

/// Properties for Darwinia Crab Network
fn crab_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 42.into());
	properties.insert("tokenDecimals".into(), 9.into());
	properties.insert("tokenSymbol".into(), "CRING".into());
	properties.insert("ktonTokenDecimals".into(), 9.into());
	properties.insert("ktonTokenSymbol".into(), "CKTON".into());

	properties
}

/// Endowed accounts for development
fn development_endowed_accounts() -> Vec<AccountId> {
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
	]
}
/// Endowed accounts for production
fn production_endowed_accounts() -> Vec<AccountId> {
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
	]
}

// TODO: doc
fn load_claims_list(path: &str) -> pallet_claims::ClaimsList {
	if !Path::new(&path).is_file() && env::var("CLAIMS_LIST_PATH").is_err() {
		Default::default()
	} else {
		serde_json::from_reader(File::open(env::var("CLAIMS_LIST_PATH").unwrap_or(path.to_owned())).unwrap()).unwrap()
	}
}

/// Helper function to create GenesisConfig for darwinia
/// is_testnet: under test net we will use Alice & Bob as seed to generate keys,
/// but in production environment, these accounts will use preset keys
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
	let num_endowed_accounts = endowed_accounts.len();

	const RING_ENDOWMENT: Balance = 20_000_000 * COIN;
	const KTON_ENDOWMENT: Balance = 10 * COIN;
	const STASH: Balance = 1000 * COIN;

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_indices: Some(IndicesConfig { indices: vec![] }),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		}),
		// pallet_democracy: Some(DemocracyConfig::default()),
		pallet_collective_Instance1: Some(CouncilConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		}),
		pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		}),
		pallet_contracts: Some(ContractsConfig {
			current_schedule: pallet_contracts::Schedule {
				enable_println: is_testnet, // this should only be enabled on development chains
				..Default::default()
			},
			gas_price: 1 * MICRO,
		}),
		pallet_sudo: Some(SudoConfig { key: root_key }),
		pallet_babe: Some(BabeConfig { authorities: vec![] }),
		pallet_im_online: Some(ImOnlineConfig { keys: vec![] }),
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),
		pallet_grandpa: Some(GrandpaConfig { authorities: vec![] }),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_society: Some(SocietyConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			pot: 0,
			max_members: 999,
		}),
		//  Custom Module
		pallet_claims: Some({
			ClaimsConfig {
				claims_list: load_claims_list("./bin/node/cli/res/claims_list_genesis.json"),
			}
		}),
		pallet_eth_backing: Some(EthBackingConfig {
			ring_redeem_address: hex!["dbc888d701167cbfb86486c516aafbefc3a4de6e"].into(),
			kton_redeem_address: hex!["dbc888d701167cbfb86486c516aafbefc3a4de6e"].into(),
			deposit_redeem_address: hex!["6ef538314829efa8386fc43386cb13b4e0a67d1e"].into(),
			ring_locked: 2_000_000_000 * COIN,
			kton_locked: 50_000 * COIN,
			..Default::default()
		}),
		pallet_eth_relay: Some(EthRelayConfig {
			authorities: eth_relay_authorities,
			genesis: Some((
				vec![
					249, 2, 21, 160, 110, 20, 148, 173, 142, 2, 161, 38, 251, 134, 207, 130, 177, 54, 39, 196, 94, 110,
					143, 30, 171, 5, 225, 66, 112, 166, 154, 5, 215, 254, 94, 38, 160, 134, 82, 165, 254, 194, 63, 139,
					211, 63, 152, 152, 192, 92, 205, 149, 60, 44, 31, 174, 176, 211, 222, 145, 140, 150, 122, 36, 183,
					9, 37, 72, 20, 148, 90, 11, 84, 213, 220, 23, 224, 170, 220, 56, 61, 45, 180, 59, 10, 13, 62, 2,
					156, 76, 160, 83, 252, 235, 143, 137, 30, 50, 28, 81, 36, 65, 74, 155, 253, 151, 238, 57, 171, 185,
					117, 6, 105, 54, 189, 34, 5, 151, 165, 120, 199, 101, 90, 160, 102, 26, 255, 46, 202, 108, 87, 179,
					203, 15, 11, 85, 212, 71, 97, 212, 38, 156, 101, 72, 237, 18, 189, 235, 170, 55, 223, 54, 210, 229,
					183, 88, 160, 75, 87, 116, 76, 217, 125, 35, 122, 135, 81, 169, 99, 23, 174, 203, 231, 219, 82,
					243, 2, 222, 211, 98, 70, 212, 23, 130, 250, 206, 129, 193, 124, 185, 1, 0, 120, 144, 204, 128,
					145, 92, 164, 64, 81, 198, 224, 193, 1, 80, 80, 132, 237, 201, 128, 225, 81, 1, 32, 16, 180, 108,
					0, 98, 62, 64, 32, 168, 58, 88, 19, 89, 208, 139, 9, 94, 173, 1, 22, 164, 8, 218, 11, 156, 55, 130,
					96, 80, 136, 33, 8, 38, 19, 52, 64, 234, 104, 36, 152, 28, 120, 37, 0, 96, 246, 74, 163, 166, 200,
					144, 16, 40, 0, 194, 53, 231, 32, 65, 100, 37, 38, 72, 164, 229, 162, 64, 230, 231, 32, 104, 0, 0,
					48, 50, 1, 4, 4, 92, 65, 45, 224, 174, 68, 138, 18, 108, 16, 64, 14, 36, 72, 100, 80, 11, 44, 36,
					158, 0, 174, 176, 97, 20, 48, 100, 183, 184, 16, 212, 230, 1, 160, 24, 84, 37, 66, 192, 149, 136,
					12, 82, 27, 137, 133, 59, 69, 132, 0, 24, 97, 107, 8, 22, 206, 144, 242, 192, 26, 100, 33, 36, 178,
					12, 61, 0, 140, 251, 224, 135, 2, 96, 123, 164, 242, 104, 32, 12, 41, 78, 28, 32, 2, 179, 40, 15,
					58, 174, 147, 18, 17, 148, 33, 226, 87, 8, 64, 187, 64, 35, 49, 49, 6, 75, 64, 141, 58, 0, 51, 120,
					153, 64, 5, 193, 9, 10, 128, 115, 193, 80, 20, 147, 176, 83, 236, 196, 128, 202, 80, 24, 94, 129,
					5, 210, 64, 118, 42, 103, 12, 164, 58, 96, 54, 64, 138, 180, 98, 4, 210, 30, 12, 146, 61, 26, 135,
					7, 219, 22, 241, 164, 64, 46, 131, 148, 110, 239, 131, 152, 131, 190, 131, 152, 55, 7, 132, 94,
					120, 169, 245, 148, 115, 112, 97, 114, 107, 112, 111, 111, 108, 45, 101, 116, 104, 45, 99, 110, 45,
					104, 122, 51, 160, 28, 248, 29, 120, 88, 139, 237, 244, 239, 138, 13, 176, 7, 187, 163, 27, 23,
					193, 8, 107, 234, 217, 185, 186, 222, 202, 141, 52, 177, 93, 180, 32, 136, 169, 141, 24, 64, 4, 34,
					198, 78,
				],
				0x7db16f1a4402e,
			)),
			..Default::default()
		}),
		pallet_kton: Some(KtonConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, KTON_ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
		}),
		pallet_ring: Some(RingConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, RING_ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
		}),
		pallet_staking: Some(StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			// --- custom ---
			payout_fraction: Perbill::from_percent(50),
			..Default::default()
		}),
		pallet_treasury: Some(Default::default()),
		pallet_vesting: Some(Default::default()),
	}
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	fn development_config_genesis() -> GenesisConfig {
		darwinia_genesis(
			vec![get_authority_keys_from_seed("Alice")],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			development_endowed_accounts(),
			true,
		)
	}

	ChainSpec::from_genesis(
		"Development",
		"dev",
		development_config_genesis,
		vec![],
		None,
		Some("DAR"),
		Some(crab_properties()),
		Default::default(),
	)
}

/// Crab local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	fn crab_config_genesis() -> GenesisConfig {
		darwinia_genesis(
			vec![
				get_authority_keys_from_seed("Alice"),
				get_authority_keys_from_seed("Bob"),
			],
			hex!["a60837b2782f7ffd23e95cd26d1aa8d493b8badc6636234ccd44db03c41fcc6c"].into(), // 5FpQFHfKd1xQ9HLZLQoG1JAQSCJoUEVBELnKsKNcuRLZejJR
			production_endowed_accounts(),
			true,
		)
	}

	ChainSpec::from_genesis(
		"Darwinia Crab Local Testnet",
		"crab_local_testnet",
		crab_config_genesis,
		vec![],
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),
		Some("DAR"),
		Some(crab_properties()),
		Default::default(),
	)
}

/// Crab network config generator
pub fn gen_crab_network_config() -> ChainSpec {
	fn gen_crab_config_genesis() -> GenesisConfig {
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
			production_endowed_accounts(),
			false,
		)
	}

	ChainSpec::from_genesis(
		"Darwinia Crab Network",
		"darwinia_crab_network",
		gen_crab_config_genesis,
		vec![],
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),
		Some("DAR"),
		Some(crab_properties()),
		Default::default(),
	)
}

#[cfg(test)]
pub(crate) mod tests {
	use sp_runtime::BuildStorage;

	use super::*;
	use crate::service::{new_full, new_light};

	// fn local_testnet_genesis_instant_single() -> GenesisConfig {
	// 	darwinia_genesis(
	// 		vec![get_authority_keys_from_seed("Alice")],
	// 		get_account_id_from_seed::<sr25519::Public>("Alice"),
	// 		vec![],
	// 		false,
	// 	)
	// }
	//
	// /// Local testnet config (single validator - Alice)
	// pub fn integration_test_config_with_single_authority() -> ChainSpec {
	// 	ChainSpec::from_genesis(
	// 		"Integration Test",
	// 		"test",
	// 		local_testnet_genesis_instant_single,
	// 		vec![],
	// 		None,
	// 		None,
	// 		None,
	// 		Default::default(),
	// 	)
	// }

	/// Local testnet config (multivalidator Alice + Bob)
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		fn local_testnet_genesis() -> GenesisConfig {
			darwinia_genesis(
				vec![
					get_authority_keys_from_seed("Alice"),
					get_authority_keys_from_seed("Bob"),
				],
				hex!["a60837b2782f7ffd23e95cd26d1aa8d493b8badc6636234ccd44db03c41fcc6c"].into(), // 5FpQFHfKd1xQ9HLZLQoG1JAQSCJoUEVBELnKsKNcuRLZejJR
				production_endowed_accounts(),
				true,
			)
		}

		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			local_testnet_genesis,
			vec![],
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sc_service_test::connectivity(
			integration_test_config_with_two_authorities(),
			|config| new_full(config),
			|config| new_light(config),
		);
	}

	#[test]
	fn test_create_development_chain_spec() {
		development_config().build_storage().unwrap();
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		local_testnet_config().build_storage().unwrap();
	}

	#[test]
	fn test_gen_crab_network_chain_spec() {
		gen_crab_network_config().build_storage().unwrap();
	}

	#[test]
	fn build_genesis_claims_list() {
		let claims = load_claims_list("./res/claims_list_genesis.json");
		println!("{:#?}", &claims.eth[0..10]);
	}
}
