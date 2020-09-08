//! Darwinia chain configurations.

// --- crates ---
use serde::{Deserialize, Serialize};
// --- substrate ---
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::{ChainSpecExtension, ChainType};
use sc_finality_grandpa::AuthorityId as GrandpaId;
use sc_service::Properties;
use sc_telemetry::TelemetryEndpoints;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{traits::IdentifyAccount, Perbill, Perquintill};
// --- darwinia ---
use array_bytes::fixed_hex_bytes_unchecked;
use crab_runtime::{constants::currency::COIN as CRING, GenesisConfig as CrabGenesisConfig};
use darwinia_primitives::{AccountId, AccountPublic, Balance};

/// The `ChainSpec parametrised for Crab runtime`.
pub type CrabChainSpec = sc_service::GenericChainSpec<CrabGenesisConfig, Extensions>;

const CKTON: Balance = CRING;
const CRAB_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "dar";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<darwinia_primitives::Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<darwinia_primitives::Block>,
}

pub fn crab_config() -> Result<CrabChainSpec, String> {
	CrabChainSpec::from_json_bytes(&include_bytes!("../res/crab.json")[..])
}

/// Session keys for Crab.
pub fn crab_session_keys(
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

/// Properties for Crab.
pub fn crab_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 42.into());
	properties.insert("tokenDecimals".into(), 9.into());
	properties.insert("tokenSymbol".into(), "CRING".into());
	properties.insert("ktonTokenDecimals".into(), 9.into());
	properties.insert("ktonTokenSymbol".into(), "CKTON".into());

	properties
}

pub fn crab_build_spec_genesis() -> CrabGenesisConfig {
	const RING_ENDOWMENT: Balance = 1_000_000 * CRING;
	const KTON_ENDOWMENT: Balance = 10_000 * CKTON;

	struct Staker {
		sr: [u8; 32],
		ed: [u8; 32],
	}
	impl Staker {
		fn build_init_auth(
			&self,
		) -> (
			AccountId,
			AccountId,
			BabeId,
			GrandpaId,
			ImOnlineId,
			AuthorityDiscoveryId,
		) {
			(
				self.sr.into(),
				self.sr.into(),
				self.sr.unchecked_into(),
				self.ed.unchecked_into(),
				self.sr.unchecked_into(),
				self.sr.unchecked_into(),
			)
		}
	}

	// 5FGWcEpsd5TbDh14UGJEzRQENwrPXUt7e2ufzFzfcCEMesAQ
	let multi_sign: AccountId = fixed_hex_bytes_unchecked!(
		"0x8db5c746c14cf05e182b10576a9ee765265366c3b7fd53c41d43640c97f4a8b8",
		32
	)
	.into();

	let root_key: AccountId = fixed_hex_bytes_unchecked!(
		"0x0a66532a23c418cca12183fee5f6afece770a0bb8725f459d7d1b1b598f91c49",
		32
	)
	.into();
	let stakers = [
		// AlexChien
		Staker {
			sr: fixed_hex_bytes_unchecked!(
				"0x80a5d9612f5504f3e04a31ca19f1d6108ca77252bd05940031eb446953409c1a",
				32
			),
			ed: fixed_hex_bytes_unchecked!(
				"0x1b861031d9a6edea47c6478cb3765d7cd4881b36bfb1c665f6b6deb5e0d9c253",
				32
			),
		},
		// AurevoirXavier
		Staker {
			// 5G9z8Ttoo7892VqBHiSWCbnd2aEdH8noJLqZ4HFMzMVNhvgP
			sr: fixed_hex_bytes_unchecked!(
				"0xb4f7f03bebc56ebe96bc52ea5ed3159d45a0ce3a8d7f082983c33ef133274747",
				32
			),
			// 5ETtsEtnsGQZc5jcAJazedgmiePShJ43VyrY88aCvdQmkvj8
			ed: fixed_hex_bytes_unchecked!(
				"0x6a282c7674945c039a9289b702376ae168e8b67c9ed320054e2a019015f236fd",
				32
			),
		},
		// clearloop
		Staker {
			sr: fixed_hex_bytes_unchecked!(
				"0x6e6844ba5c73db6c4c6b67ea59c2787dd6bd2f9b8139a69c33e14a722d1e801d",
				32
			),
			ed: fixed_hex_bytes_unchecked!(
				"0x13c0b78d9573e99a74c313ddcf30f8fc3d3bc0503f8864427ad34654804e1bc5",
				32
			),
		},
		// freehere107
		Staker {
			sr: fixed_hex_bytes_unchecked!(
				"0xc4429847f3598f40008d0cbab53476a2f19165696aa41002778524b3ecf82938",
				32
			),
			ed: fixed_hex_bytes_unchecked!(
				"0x2c8cb4d2de3192df18c60551038a506033cb2a85fbe0a3ff8cff413dac11f50a",
				32
			),
		},
		// HackFisher
		Staker {
			sr: fixed_hex_bytes_unchecked!(
				"0xb62d88e3f439fe9b5ea799b27bf7c6db5e795de1784f27b1bc051553499e420f",
				32
			),
			ed: fixed_hex_bytes_unchecked!(
				"0x398f7935e0ea85cc2d1af71dab00d93f53b2cbf35e2afb1e6087f7554d2fdf96",
				32
			),
		},
		// WoeOm
		Staker {
			// 5C8thCAFsaTHuJFMJZz2CrT47XDWebP72Vwr9d1sL4eSJ4UM
			sr: fixed_hex_bytes_unchecked!(
				"0x0331760198d850b159844f3bfa620f6e704167973213154aca27675f7ddd987e",
				32
			),
			// 5D2ocj7mvu5oemVwK2TXUz7tmNumtPSYdjs4fmFmNKQ9PJ3A
			ed: fixed_hex_bytes_unchecked!(
				"0x2ac9219ace40f5846ed675dded4e25a1997da7eabdea2f78597a71d6f3803148",
				32
			),
		},
		// yanganto
		Staker {
			sr: fixed_hex_bytes_unchecked!(
				"0xc45f075b5b1aa0145c469f57bd741c02272c1c0c41e9518d5a32426030d98232",
				32
			),
			ed: fixed_hex_bytes_unchecked!(
				"0xaf78c408272f929225861c8276c6e8700c8f45c195b9ba82a0b246aade0937ec",
				32
			),
		},
	];
	// local tester
	let local_tester = Staker {
		// Secret phrase `pulse upset spoil fatigue agent credit dirt language forest aware boat broom` is account:
		// Network ID/version: substrate
		// Secret seed:        0x76c87263b2a385fcb7faed857d0fe105b5e40cdc8cb5f1b2a188d7f57488e595
		// Public key (hex):   0x584ea8f083c3a9038d57acc5229ab4d790ab6132921d5edc5fae1be4ed89ec1f
		// Account ID:         0x584ea8f083c3a9038d57acc5229ab4d790ab6132921d5edc5fae1be4ed89ec1f
		// SS58 Address:       5E4VSMKXm9VFaLMu4Jjbny3Uy7NnPizoGkf92A15XjS45C4A
		sr: fixed_hex_bytes_unchecked!(
			"0x584ea8f083c3a9038d57acc5229ab4d790ab6132921d5edc5fae1be4ed89ec1f",
			32
		),
		// Secret phrase `ecology admit arrest canal cage believe satoshi anger napkin sign decorate use` is account:
		// Network ID/version: substrate
		// Secret seed:        0x7b37f9bd46a368748e0e28992e2cd2bc77060cd8267784aef625fb812908fb7f
		// Public key (hex):   0x70fa82107e81f20bb4e5b059f4ac800d55aafcff9e918e000899569b4f207976
		// Account ID:         0x70fa82107e81f20bb4e5b059f4ac800d55aafcff9e918e000899569b4f207976
		// SS58 Address:       5Ecqdt4nxP76MdwNfBwwYBi4mxWq7MYLDN1GXMtDFUSaerjG
		ed: fixed_hex_bytes_unchecked!(
			"0x70fa82107e81f20bb4e5b059f4ac800d55aafcff9e918e000899569b4f207976",
			32
		),
	};
	let endowed_accounts = stakers
		.iter()
		.map(|staker| staker.sr.into())
		.collect::<Vec<_>>();
	let initial_authorities = [stakers[1].build_init_auth(), local_tester.build_init_auth()];

	CrabGenesisConfig {
		frame_system: Some(crab_runtime::SystemConfig {
			code: crab_runtime::wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_babe: Some(Default::default()),
		pallet_indices: Some(Default::default()),
		darwinia_balances_Instance0: Some(crab_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, RING_ENDOWMENT))
				.chain(
					vec![
						(root_key.clone(), 25_000_000 * CRING),
						(multi_sign, 700_000_000 * CRING),
						(local_tester.sr.into(), CRING),
					]
					.into_iter(),
				)
				.collect(),
		}),
		darwinia_balances_Instance1: Some(crab_runtime::KtonConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, KTON_ENDOWMENT))
				.collect(),
		}),
		darwinia_staking: Some(crab_runtime::StakingConfig {
			minimum_validator_count: 2,
			validator_count: 7,
			stakers: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0, x.1, CRING, crab_runtime::StakerStatus::Validator))
				.collect(),
			force_era: crab_runtime::Forcing::NotForcing,
			slash_reward_fraction: Perbill::from_percent(10),
			payout_fraction: Perquintill::from_percent(50),
			..Default::default()
		}),
		pallet_session: Some(crab_runtime::SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0.clone(), x.0, crab_session_keys(x.2, x.3, x.4, x.5)))
				.collect(),
		}),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(Default::default()),
		pallet_collective_Instance0: Some(Default::default()),
		pallet_collective_Instance1: Some(Default::default()),
		darwinia_elections_phragmen: Some(Default::default()),
		pallet_membership_Instance0: Some(Default::default()),
		darwinia_claims: Some({
			crab_runtime::ClaimsConfig {
				claims_list: crab_runtime::ClaimsList::from_file(
					".node/service/res/crab_claims_list.json",
					"CLAIMS_LIST_PATH",
				),
			}
		}),
		pallet_sudo: Some(crab_runtime::SudoConfig { key: root_key }),
		darwinia_ethereum_backing: Some(crab_runtime::EthereumBackingConfig {
			token_redeem_address: fixed_hex_bytes_unchecked!(
				"0x49262B932E439271d05634c32978294C7Ea15d0C",
				20
			)
			.into(),
			deposit_redeem_address: fixed_hex_bytes_unchecked!(
				"0x6EF538314829EfA8386Fc43386cB13B4e0A67D1e",
				20
			)
			.into(),
			ring_token_address: fixed_hex_bytes_unchecked!(
				"0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0",
				20
			)
			.into(),
			kton_token_address: fixed_hex_bytes_unchecked!(
				"0x1994100c58753793D52c6f457f189aa3ce9cEe94",
				20
			)
			.into(),
			ring_locked: 7_569_833 * CRING,
			kton_locked: 30_000 * CRING,
			..Default::default()
		}),
		darwinia_ethereum_relay: Some(crab_runtime::EthereumRelayConfig {
			genesis_header_info: (
				0,
				b"A\x94\x10#h\t#\xe0\xfeMt\xa3K\xda\xc8\x14\x1f%@\xe3\xae\x90b7\x18\xe4}f\xd1\xcaJ-".into(),
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: crab_runtime::DagsMerkleRootsLoader::from_file(
				".node/service/res/dags_merkle_roots_loader.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		}),
	}
}

/// Crab config.
pub fn crab_build_spec_config() -> CrabChainSpec {
	let boot_nodes = vec![];
	CrabChainSpec::from_genesis(
		"Darwinia Crab",
		"crab",
		ChainType::Live,
		crab_build_spec_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(CRAB_TELEMETRY_URL.to_string(), 0)])
				.expect("Crab telemetry url is valid; qed"),
		),
		Some(DEFAULT_PROTOCOL_ID),
		Some(crab_properties()),
		Default::default(),
	)
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Generate an account ID from seed.
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
	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(testnet_accounts);

	CrabGenesisConfig {
		frame_system: Some(crab_runtime::SystemConfig {
			code: crab_runtime::wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_babe: Some(Default::default()),
		pallet_indices: Some(Default::default()),
		darwinia_balances_Instance0: Some(crab_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		}),
		darwinia_balances_Instance1: Some(crab_runtime::KtonConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		}),
		darwinia_staking: Some(crab_runtime::StakingConfig {
			minimum_validator_count: 1,
			validator_count: 2,
			stakers: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0, x.1, 1 << 60, crab_runtime::StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().cloned().map(|x| x.0).collect(),
			force_era: crab_runtime::Forcing::ForceAlways,
			slash_reward_fraction: Perbill::from_percent(10),
			payout_fraction: Perquintill::from_percent(50),
			..Default::default()
		}),
		pallet_session: Some(crab_runtime::SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0.clone(), x.0, crab_session_keys(x.2, x.3, x.4, x.5)))
				.collect(),
		}),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(Default::default()),
		pallet_collective_Instance0: Some(Default::default()),
		pallet_collective_Instance1: Some(Default::default()),
		darwinia_elections_phragmen: Some(Default::default()),
		pallet_membership_Instance0: Some(Default::default()),
		darwinia_claims: Some({
			crab_runtime::ClaimsConfig {
				claims_list: crab_runtime::ClaimsList::from_file(
					"./service/res/crab_claims_list.json",
					"CLAIMS_LIST_PATH",
				),
			}
		}),
		pallet_sudo: Some(crab_runtime::SudoConfig { key: root_key }),
		darwinia_ethereum_backing: Some(crab_runtime::EthereumBackingConfig {
			token_redeem_address: fixed_hex_bytes_unchecked!(
				"0x49262B932E439271d05634c32978294C7Ea15d0C",
				20
			)
			.into(),
			deposit_redeem_address: fixed_hex_bytes_unchecked!(
				"0x6EF538314829EfA8386Fc43386cB13B4e0A67D1e",
				20
			)
			.into(),
			ring_token_address: fixed_hex_bytes_unchecked!(
				"0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0",
				20
			)
			.into(),
			kton_token_address: fixed_hex_bytes_unchecked!(
				"0x1994100c58753793D52c6f457f189aa3ce9cEe94",
				20
			)
			.into(),
			ring_locked: 1 << 60,
			kton_locked: 1 << 60,
			..Default::default()
		}),
		darwinia_ethereum_relay: Some(crab_runtime::EthereumRelayConfig {
			genesis_header_info: (
				0,
				b"A\x94\x10#h\t#\xe0\xfeMt\xa3K\xda\xc8\x14\x1f%@\xe3\xae\x90b7\x18\xe4}f\xd1\xcaJ-".into(),
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: crab_runtime::DagsMerkleRootsLoader::from_file(
				"./service/res/dags_merkle_roots_loader.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		}),
	}
}

/// Crab development config (single validator Alice)
pub fn crab_development_config() -> CrabChainSpec {
	fn crab_development_genesis() -> CrabGenesisConfig {
		crab_testnet_genesis(
			vec![get_authority_keys_from_seed("Alice")],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			Some(vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			]),
		)
	}

	CrabChainSpec::from_genesis(
		"Development",
		"crab_dev",
		ChainType::Development,
		crab_development_genesis,
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		Some(crab_properties()),
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
			Some(vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			]),
		)
	}

	CrabChainSpec::from_genesis(
		"Crab Local Testnet",
		"crab_local_testnet",
		ChainType::Local,
		crab_local_testnet_genesis,
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		Some(crab_properties()),
		Default::default(),
	)
}
