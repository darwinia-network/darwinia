//! Polkadot chain configurations.

// --- std ---
use std::{env, fs::File, path::Path};
// --- third-party ---
use hex_literal::hex;
use serde::{Deserialize, Serialize};
// --- substrate ---
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_finality_grandpa::AuthorityId as GrandpaId;
use sc_telemetry::TelemetryEndpoints;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{traits::IdentifyAccount, Perbill};
// --- darwinia ---
use crab_runtime::{constants::currency::COIN as CRING, GenesisConfig as CrabGenesisConfig};
use darwinia_primitives::{AccountId, AccountPublic, Balance};

const CKTON: Balance = CRING;
const CRAB_STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "dar";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client::ForkBlocks<darwinia_primitives::Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client::BadBlocks<darwinia_primitives::Block>,
}

/// The `ChainSpec parametrised for Crab runtime`.
pub type CrabChainSpec = sc_service::GenericChainSpec<CrabGenesisConfig, Extensions>;

pub fn crab_config() -> Result<CrabChainSpec, String> {
	CrabChainSpec::from_json_bytes(&include_bytes!("../res/crab.json")[..])
}

fn crab_session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> crab_runtime::SessionKeys {
	crab_runtime::SessionKeys {
		babe,
		grandpa,
		im_online,
		authority_discovery,
	}
}

fn load_claims_list(path: &str) -> crab_runtime::ClaimsList {
	if !Path::new(&path).is_file() && env::var("CLAIMS_LIST_PATH").is_err() {
		Default::default()
	} else {
		serde_json::from_reader(
			File::open(env::var("CLAIMS_LIST_PATH").unwrap_or(path.to_owned())).unwrap(),
		)
		.unwrap()
	}
}

fn crab_staging_testnet_config_genesis() -> CrabGenesisConfig {
	const RING_ENDOWMENT: Balance = 20_000_000 * CRING;
	const KTON_ENDOWMENT: Balance = 10 * CKTON;
	const STASH: Balance = 1000 * CRING;

	// subkey inspect "$SECRET"
	let endowed_accounts: Vec<AccountId> = vec![
		// 5CVFESwfkk7NmhQ6FwHCM9roBvr9BGa4vJHFYU8DnGQxrXvz
		hex!["12b782529c22032ed4694e0f6e7d486be7daa6d12088f6bc74d593b3900b8438"].into(),
	];

	// for i in 1 2 3 4; do for j in stash controller; do subkey inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in babe; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in grandpa; do subkey --ed25519 inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in im_online; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in parachains; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
	let initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)> = vec![
		(
			// 5DD7Q4VEfPTLEdn11CnThoHT5f9xKCrnofWJL5SsvpTghaAT
			hex!["32a5718e87d16071756d4b1370c411bbbb947eb62f0e6e0b937d5cbfc0ea633b"].into(),
			// 5GNzaEqhrZAtUQhbMe2gn9jBuNWfamWFZHULryFwBUXyd1cG
			hex!["bee39fe862c85c91aaf343e130d30b643c6ea0b4406a980206f1df8331f7093b"].into(),
			// 5FpewyS2VY8Cj3tKgSckq8ECkjd1HKHvBRnWhiHqRQsWfFC1
			hex!["a639b507ee1585e0b6498ff141d6153960794523226866d1b44eba3f25f36356"]
				.unchecked_into(),
			// 5EjvdwATjyFFikdZibVvx1q5uBHhphS2Mnsq5c7yfaYK25vm
			hex!["76620f7c98bce8619979c2b58cf2b0aff71824126d2b039358729dad993223db"]
				.unchecked_into(),
			// 5FpewyS2VY8Cj3tKgSckq8ECkjd1HKHvBRnWhiHqRQsWfFC1
			hex!["a639b507ee1585e0b6498ff141d6153960794523226866d1b44eba3f25f36356"]
				.unchecked_into(),
			// 5FpewyS2VY8Cj3tKgSckq8ECkjd1HKHvBRnWhiHqRQsWfFC1
			hex!["a639b507ee1585e0b6498ff141d6153960794523226866d1b44eba3f25f36356"]
				.unchecked_into(),
		),
		(
			// 5G9VGb8ESBeS8Ca4or43RfhShzk9y7T5iTmxHk5RJsjZwsRx
			hex!["b496c98a405ceab59b9e970e59ef61acd7765a19b704e02ab06c1cdfe171e40f"].into(),
			// 5F7V9Y5FcxKXe1aroqvPeRiUmmeQwTFcL3u9rrPXcMuMiCNx
			hex!["86d3a7571dd60139d297e55d8238d0c977b2e208c5af088f7f0136b565b0c103"].into(),
			// 5GvuM53k1Z4nAB5zXJFgkRSHv4Bqo4BsvgbQWNWkiWZTMwWY
			hex!["765e46067adac4d1fe6c783aa2070dfa64a19f84376659e12705d1734b3eae01"]
				.unchecked_into(),
			// 5HBDAaybNqjmY7ww8ZcZZY1L5LHxvpnyfqJwoB7HhR6raTmG
			hex!["e2234d661bee4a04c38392c75d1566200aa9e6ae44dd98ee8765e4cc9af63cb7"]
				.unchecked_into(),
			// 5GvuM53k1Z4nAB5zXJFgkRSHv4Bqo4BsvgbQWNWkiWZTMwWY
			hex!["765e46067adac4d1fe6c783aa2070dfa64a19f84376659e12705d1734b3eae01"]
				.unchecked_into(),
			// 5GvuM53k1Z4nAB5zXJFgkRSHv4Bqo4BsvgbQWNWkiWZTMwWY
			hex!["765e46067adac4d1fe6c783aa2070dfa64a19f84376659e12705d1734b3eae01"]
				.unchecked_into(),
		),
		(
			// 5FzwpgGvk2kk9agow6KsywLYcPzjYc8suKej2bne5G5b9YU3
			hex!["ae12f70078a22882bf5135d134468f77301927aa67c376e8c55b7ff127ace115"].into(),
			// 5EqoZhVC2BcsM4WjvZNidu2muKAbu5THQTBKe3EjvxXkdP7A
			hex!["7addb914ec8486bbc60643d2647685dcc06373401fa80e09813b630c5831d54b"].into(),
			// 5CXNq1mSKJT4Sc2CbyBBdANeSkbUvdWvE4czJjKXfBHi9sX5
			hex!["664eae1ca4713dd6abf8c15e6c041820cda3c60df97dc476c2cbf7cb82cb2d2e"]
				.unchecked_into(),
			// 5E8ULLQrDAtWhfnVfZmX41Yux86zNAwVJYguWJZVWrJvdhBe
			hex!["5b57ed1443c8967f461db1f6eb2ada24794d163a668f1cf9d9ce3235dfad8799"]
				.unchecked_into(),
			// 5CXNq1mSKJT4Sc2CbyBBdANeSkbUvdWvE4czJjKXfBHi9sX5
			hex!["664eae1ca4713dd6abf8c15e6c041820cda3c60df97dc476c2cbf7cb82cb2d2e"]
				.unchecked_into(),
			// 5CXNq1mSKJT4Sc2CbyBBdANeSkbUvdWvE4czJjKXfBHi9sX5
			hex!["664eae1ca4713dd6abf8c15e6c041820cda3c60df97dc476c2cbf7cb82cb2d2e"]
				.unchecked_into(),
		),
		(
			// 5CFj6Kg9rmVn1vrqpyjau2ztyBzKeVdRKwNPiA3tqhB5HPqq
			hex!["0867dbb49721126df589db100dda728dc3b475cbf414dad8f72a1d5e84897252"].into(),
			// 5CwQXP6nvWzigFqNhh2jvCaW9zWVzkdveCJY3tz2MhXMjTon
			hex!["26ab2b4b2eba2263b1e55ceb48f687bb0018130a88df0712fbdaf6a347d50e2a"].into(),
			// 5FCd9Y7RLNyxz5wnCAErfsLbXGG34L2BaZRHzhiJcMUMd5zd
			hex!["2adb17a5cafbddc7c3e00ec45b6951a8b12ce2264235b4def342513a767e5d3d"]
				.unchecked_into(),
			// 5HGLmrZsiTFTPp3QoS1W8w9NxByt8PVq79reqvdxNcQkByqK
			hex!["e60d23f49e93c1c1f2d7c115957df5bbd7faf5ebf138d1e9d02e8b39a1f63df0"]
				.unchecked_into(),
			// 5FCd9Y7RLNyxz5wnCAErfsLbXGG34L2BaZRHzhiJcMUMd5zd
			hex!["2adb17a5cafbddc7c3e00ec45b6951a8b12ce2264235b4def342513a767e5d3d"]
				.unchecked_into(),
			// 5FCd9Y7RLNyxz5wnCAErfsLbXGG34L2BaZRHzhiJcMUMd5zd
			hex!["2adb17a5cafbddc7c3e00ec45b6951a8b12ce2264235b4def342513a767e5d3d"]
				.unchecked_into(),
		),
	];

	let root_key = hex!["12b782529c22032ed4694e0f6e7d486be7daa6d12088f6bc74d593b3900b8438"].into();

	CrabGenesisConfig {
		// --- substrate ---
		frame_system: Some(crab_runtime::SystemConfig {
			code: crab_runtime::WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_babe: Some(Default::default()),
		pallet_indices: Some(Default::default()),
		pallet_session: Some(crab_runtime::SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						crab_session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		}),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(Default::default()),
		pallet_collective_Instance1: Some(Default::default()),
		pallet_collective_Instance2: Some(Default::default()),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_sudo: Some(crab_runtime::SudoConfig { key: root_key }),
		// --- darwinia ---
		darwinia_balances_Instance0: Some(crab_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.map(|k| (k.clone(), RING_ENDOWMENT))
				.collect(),
		}),
		darwinia_balances_Instance1: Some(crab_runtime::KtonConfig {
			balances: endowed_accounts
				.iter()
				.map(|k| (k.clone(), KTON_ENDOWMENT))
				.collect(),
		}),
		darwinia_staking: Some(crab_runtime::StakingConfig {
			minimum_validator_count: 1,
			validator_count: 2,
			stakers: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.1.clone(),
						STASH,
						crab_runtime::StakerStatus::Validator,
					)
				})
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			force_era: crab_runtime::Forcing::NotForcing,
			slash_reward_fraction: Perbill::from_percent(10),
			payout_fraction: Perbill::from_percent(50),
			..Default::default()
		}),
		darwinia_claims: Some({
			crab_runtime::ClaimsConfig {
				claims_list: load_claims_list("./bin/node/cli/res/claims_list_genesis.json"),
			}
		}),
		darwinia_eth_backing: Some(crab_runtime::EthBackingConfig {
			ring_redeem_address: hex!["dbc888d701167cbfb86486c516aafbefc3a4de6e"].into(),
			kton_redeem_address: hex!["dbc888d701167cbfb86486c516aafbefc3a4de6e"].into(),
			deposit_redeem_address: hex!["6ef538314829efa8386fc43386cb13b4e0a67d1e"].into(),
			ring_locked: 2_000_000_000 * CRING,
			kton_locked: 50_000 * CRING,
			..Default::default()
		}),
		darwinia_eth_relay: Some(crab_runtime::EthRelayConfig {
			authorities: vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
			],
			genesis: Some((
				vec![
					249, 2, 21, 160, 110, 20, 148, 173, 142, 2, 161, 38, 251, 134, 207, 130, 177,
					54, 39, 196, 94, 110, 143, 30, 171, 5, 225, 66, 112, 166, 154, 5, 215, 254, 94,
					38, 160, 134, 82, 165, 254, 194, 63, 139, 211, 63, 152, 152, 192, 92, 205, 149,
					60, 44, 31, 174, 176, 211, 222, 145, 140, 150, 122, 36, 183, 9, 37, 72, 20,
					148, 90, 11, 84, 213, 220, 23, 224, 170, 220, 56, 61, 45, 180, 59, 10, 13, 62,
					2, 156, 76, 160, 83, 252, 235, 143, 137, 30, 50, 28, 81, 36, 65, 74, 155, 253,
					151, 238, 57, 171, 185, 117, 6, 105, 54, 189, 34, 5, 151, 165, 120, 199, 101,
					90, 160, 102, 26, 255, 46, 202, 108, 87, 179, 203, 15, 11, 85, 212, 71, 97,
					212, 38, 156, 101, 72, 237, 18, 189, 235, 170, 55, 223, 54, 210, 229, 183, 88,
					160, 75, 87, 116, 76, 217, 125, 35, 122, 135, 81, 169, 99, 23, 174, 203, 231,
					219, 82, 243, 2, 222, 211, 98, 70, 212, 23, 130, 250, 206, 129, 193, 124, 185,
					1, 0, 120, 144, 204, 128, 145, 92, 164, 64, 81, 198, 224, 193, 1, 80, 80, 132,
					237, 201, 128, 225, 81, 1, 32, 16, 180, 108, 0, 98, 62, 64, 32, 168, 58, 88,
					19, 89, 208, 139, 9, 94, 173, 1, 22, 164, 8, 218, 11, 156, 55, 130, 96, 80,
					136, 33, 8, 38, 19, 52, 64, 234, 104, 36, 152, 28, 120, 37, 0, 96, 246, 74,
					163, 166, 200, 144, 16, 40, 0, 194, 53, 231, 32, 65, 100, 37, 38, 72, 164, 229,
					162, 64, 230, 231, 32, 104, 0, 0, 48, 50, 1, 4, 4, 92, 65, 45, 224, 174, 68,
					138, 18, 108, 16, 64, 14, 36, 72, 100, 80, 11, 44, 36, 158, 0, 174, 176, 97,
					20, 48, 100, 183, 184, 16, 212, 230, 1, 160, 24, 84, 37, 66, 192, 149, 136, 12,
					82, 27, 137, 133, 59, 69, 132, 0, 24, 97, 107, 8, 22, 206, 144, 242, 192, 26,
					100, 33, 36, 178, 12, 61, 0, 140, 251, 224, 135, 2, 96, 123, 164, 242, 104, 32,
					12, 41, 78, 28, 32, 2, 179, 40, 15, 58, 174, 147, 18, 17, 148, 33, 226, 87, 8,
					64, 187, 64, 35, 49, 49, 6, 75, 64, 141, 58, 0, 51, 120, 153, 64, 5, 193, 9,
					10, 128, 115, 193, 80, 20, 147, 176, 83, 236, 196, 128, 202, 80, 24, 94, 129,
					5, 210, 64, 118, 42, 103, 12, 164, 58, 96, 54, 64, 138, 180, 98, 4, 210, 30,
					12, 146, 61, 26, 135, 7, 219, 22, 241, 164, 64, 46, 131, 148, 110, 239, 131,
					152, 131, 190, 131, 152, 55, 7, 132, 94, 120, 169, 245, 148, 115, 112, 97, 114,
					107, 112, 111, 111, 108, 45, 101, 116, 104, 45, 99, 110, 45, 104, 122, 51, 160,
					28, 248, 29, 120, 88, 139, 237, 244, 239, 138, 13, 176, 7, 187, 163, 27, 23,
					193, 8, 107, 234, 217, 185, 186, 222, 202, 141, 52, 177, 93, 180, 32, 136, 169,
					141, 24, 64, 4, 34, 198, 78,
				],
				0x7db16f1a4402e,
			)),
			..Default::default()
		}),
		darwinia_vesting: Some(Default::default()),
	}
}

/// Staging testnet config.
pub fn crab_staging_testnet_config() -> CrabChainSpec {
	let boot_nodes = vec![];
	CrabChainSpec::from_genesis(
		"Crab Staging Testnet",
		"crab_staging_testnet",
		crab_staging_testnet_config_genesis,
		boot_nodes,
		Some(TelemetryEndpoints::new(vec![(
			CRAB_STAGING_TELEMETRY_URL.to_string(),
			0,
		)])),
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Default::default(),
	)
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
	BabeId,
	GrandpaId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

fn testnet_accounts() -> Vec<AccountId> {
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

/// Helper function to create Crab GenesisConfig for testing
pub fn crab_testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
) -> CrabGenesisConfig {
	const RING_ENDOWMENT: Balance = 20_000_000 * CRING;
	const KTON_ENDOWMENT: Balance = 10 * CKTON;
	const STASH: Balance = 1000 * CRING;

	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(testnet_accounts);

	CrabGenesisConfig {
		// --- substrate ---
		frame_system: Some(crab_runtime::SystemConfig {
			code: crab_runtime::WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_babe: Some(Default::default()),
		pallet_indices: Some(Default::default()),
		pallet_session: Some(crab_runtime::SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						crab_session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		}),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(Default::default()),
		pallet_collective_Instance1: Some(Default::default()),
		pallet_collective_Instance2: Some(Default::default()),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_sudo: Some(crab_runtime::SudoConfig { key: root_key }),
		// --- darwinia ---
		darwinia_balances_Instance0: Some(crab_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.map(|k| (k.clone(), RING_ENDOWMENT))
				.collect(),
		}),
		darwinia_balances_Instance1: Some(crab_runtime::KtonConfig {
			balances: endowed_accounts
				.iter()
				.map(|k| (k.clone(), KTON_ENDOWMENT))
				.collect(),
		}),
		darwinia_staking: Some(crab_runtime::StakingConfig {
			minimum_validator_count: 1,
			validator_count: 2,
			stakers: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.1.clone(),
						STASH,
						crab_runtime::StakerStatus::Validator,
					)
				})
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			force_era: crab_runtime::Forcing::NotForcing,
			slash_reward_fraction: Perbill::from_percent(10),
			payout_fraction: Perbill::from_percent(50),
			..Default::default()
		}),
		darwinia_claims: Some({
			crab_runtime::ClaimsConfig {
				claims_list: load_claims_list("./service/res/crab_claims_list.json"),
			}
		}),
		darwinia_eth_backing: Some(crab_runtime::EthBackingConfig {
			ring_redeem_address: hex!["dbc888d701167cbfb86486c516aafbefc3a4de6e"].into(),
			kton_redeem_address: hex!["dbc888d701167cbfb86486c516aafbefc3a4de6e"].into(),
			deposit_redeem_address: hex!["6ef538314829efa8386fc43386cb13b4e0a67d1e"].into(),
			ring_locked: 2_000_000_000 * CRING,
			kton_locked: 50_000 * CRING,
			..Default::default()
		}),
		darwinia_eth_relay: Some(crab_runtime::EthRelayConfig {
			authorities: vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
			],
			genesis: Some((
				vec![
					249, 2, 21, 160, 110, 20, 148, 173, 142, 2, 161, 38, 251, 134, 207, 130, 177,
					54, 39, 196, 94, 110, 143, 30, 171, 5, 225, 66, 112, 166, 154, 5, 215, 254, 94,
					38, 160, 134, 82, 165, 254, 194, 63, 139, 211, 63, 152, 152, 192, 92, 205, 149,
					60, 44, 31, 174, 176, 211, 222, 145, 140, 150, 122, 36, 183, 9, 37, 72, 20,
					148, 90, 11, 84, 213, 220, 23, 224, 170, 220, 56, 61, 45, 180, 59, 10, 13, 62,
					2, 156, 76, 160, 83, 252, 235, 143, 137, 30, 50, 28, 81, 36, 65, 74, 155, 253,
					151, 238, 57, 171, 185, 117, 6, 105, 54, 189, 34, 5, 151, 165, 120, 199, 101,
					90, 160, 102, 26, 255, 46, 202, 108, 87, 179, 203, 15, 11, 85, 212, 71, 97,
					212, 38, 156, 101, 72, 237, 18, 189, 235, 170, 55, 223, 54, 210, 229, 183, 88,
					160, 75, 87, 116, 76, 217, 125, 35, 122, 135, 81, 169, 99, 23, 174, 203, 231,
					219, 82, 243, 2, 222, 211, 98, 70, 212, 23, 130, 250, 206, 129, 193, 124, 185,
					1, 0, 120, 144, 204, 128, 145, 92, 164, 64, 81, 198, 224, 193, 1, 80, 80, 132,
					237, 201, 128, 225, 81, 1, 32, 16, 180, 108, 0, 98, 62, 64, 32, 168, 58, 88,
					19, 89, 208, 139, 9, 94, 173, 1, 22, 164, 8, 218, 11, 156, 55, 130, 96, 80,
					136, 33, 8, 38, 19, 52, 64, 234, 104, 36, 152, 28, 120, 37, 0, 96, 246, 74,
					163, 166, 200, 144, 16, 40, 0, 194, 53, 231, 32, 65, 100, 37, 38, 72, 164, 229,
					162, 64, 230, 231, 32, 104, 0, 0, 48, 50, 1, 4, 4, 92, 65, 45, 224, 174, 68,
					138, 18, 108, 16, 64, 14, 36, 72, 100, 80, 11, 44, 36, 158, 0, 174, 176, 97,
					20, 48, 100, 183, 184, 16, 212, 230, 1, 160, 24, 84, 37, 66, 192, 149, 136, 12,
					82, 27, 137, 133, 59, 69, 132, 0, 24, 97, 107, 8, 22, 206, 144, 242, 192, 26,
					100, 33, 36, 178, 12, 61, 0, 140, 251, 224, 135, 2, 96, 123, 164, 242, 104, 32,
					12, 41, 78, 28, 32, 2, 179, 40, 15, 58, 174, 147, 18, 17, 148, 33, 226, 87, 8,
					64, 187, 64, 35, 49, 49, 6, 75, 64, 141, 58, 0, 51, 120, 153, 64, 5, 193, 9,
					10, 128, 115, 193, 80, 20, 147, 176, 83, 236, 196, 128, 202, 80, 24, 94, 129,
					5, 210, 64, 118, 42, 103, 12, 164, 58, 96, 54, 64, 138, 180, 98, 4, 210, 30,
					12, 146, 61, 26, 135, 7, 219, 22, 241, 164, 64, 46, 131, 148, 110, 239, 131,
					152, 131, 190, 131, 152, 55, 7, 132, 94, 120, 169, 245, 148, 115, 112, 97, 114,
					107, 112, 111, 111, 108, 45, 101, 116, 104, 45, 99, 110, 45, 104, 122, 51, 160,
					28, 248, 29, 120, 88, 139, 237, 244, 239, 138, 13, 176, 7, 187, 163, 27, 23,
					193, 8, 107, 234, 217, 185, 186, 222, 202, 141, 52, 177, 93, 180, 32, 136, 169,
					141, 24, 64, 4, 34, 198, 78,
				],
				0x7db16f1a4402e,
			)),
			..Default::default()
		}),
		darwinia_vesting: Some(Default::default()),
	}
}

/// Crab development config (single validator Alice)
pub fn crab_development_config() -> CrabChainSpec {
	fn crab_development_genesis() -> CrabGenesisConfig {
		crab_testnet_genesis(
			vec![get_authority_keys_from_seed("Alice")],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
		)
	}

	CrabChainSpec::from_genesis(
		"Development",
		"crab_dev",
		crab_development_genesis,
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Default::default(),
	)
}

/// Crab local testnet config (multivalidator Alice + Bob)
pub fn crab_local_testnet_config() -> CrabChainSpec {
	fn crab_local_testnet_genesis() -> CrabGenesisConfig {
		crab_testnet_genesis(
			vec![
				get_authority_keys_from_seed("Alice"),
				get_authority_keys_from_seed("Bob"),
			],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
		)
	}

	CrabChainSpec::from_genesis(
		"Crab Local Testnet",
		"crab_local_testnet",
		crab_local_testnet_genesis,
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Default::default(),
	)
}
