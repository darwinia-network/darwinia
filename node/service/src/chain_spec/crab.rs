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
use array_bytes::fixed_hex_bytes_unchecked;
use crab_runtime::{constants::currency::COIN, *};
use darwinia_primitives::{AccountId, Balance};

/// The `ChainSpec parametrised for Crab runtime`.
pub type CrabChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

const CRAB_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

pub fn crab_config() -> Result<CrabChainSpec, String> {
	CrabChainSpec::from_json_bytes(&include_bytes!("../../res/crab/crab.json")[..])
}

/// Session keys for Crab.
pub fn crab_session_keys(
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

pub fn crab_build_spec_genesis() -> GenesisConfig {
	const C_RING_ENDOWMENT: Balance = 1_000_000 * COIN;
	const C_KTON_ENDOWMENT: Balance = 10_000 * COIN;

	const ROOT: &'static str = "0x0a66532a23c418cca12183fee5f6afece770a0bb8725f459d7d1b1b598f91c49";
	const MULTI_SIG: &'static str =
		"0x8db5c746c14cf05e182b10576a9ee765265366c3b7fd53c41d43640c97f4a8b8";
	const GENESIS_VALIDATOR_SR: &'static str =
		"0xb4f7f03bebc56ebe96bc52ea5ed3159d45a0ce3a8d7f082983c33ef133274747";
	const GENESIS_VALIDATOR_ED: &'static str =
		"0x6a282c7674945c039a9289b702376ae168e8b67c9ed320054e2a019015f236fd";

	const TOKEN_REDEEM_ADDRESS: &'static str = "0x49262B932E439271d05634c32978294C7Ea15d0C";
	const DEPOSIT_REDEEM_ADDRESS: &'static str = "0x6EF538314829EfA8386Fc43386cB13B4e0A67D1e";
	const RING_TOKEN_ADDRESS: &'static str = "0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0";
	const KTON_TOKEN_ADDRESS: &'static str = "0x1994100c58753793D52c6f457f189aa3ce9cEe94";

	let root: AccountId = fixed_hex_bytes_unchecked!(ROOT, 32).into();
	let multi_sig: AccountId = fixed_hex_bytes_unchecked!(MULTI_SIG, 32).into();
	let genesis_validator: (
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		AuthorityDiscoveryId,
	) = {
		let stash = fixed_hex_bytes_unchecked!(GENESIS_VALIDATOR_SR, 32);
		let controller = fixed_hex_bytes_unchecked!(GENESIS_VALIDATOR_SR, 32);
		let session = fixed_hex_bytes_unchecked!(GENESIS_VALIDATOR_SR, 32);
		let grandpa = fixed_hex_bytes_unchecked!(GENESIS_VALIDATOR_ED, 32);

		(
			stash.into(),
			controller.into(),
			session.unchecked_into(),
			grandpa.unchecked_into(),
			session.unchecked_into(),
			session.unchecked_into(),
		)
	};
	let endowed_accounts = [
		// AlexChien
		"0x80a5d9612f5504f3e04a31ca19f1d6108ca77252bd05940031eb446953409c1a",
		// clearloop
		"0x6e6844ba5c73db6c4c6b67ea59c2787dd6bd2f9b8139a69c33e14a722d1e801d",
		// freehere107
		"0xc4429847f3598f40008d0cbab53476a2f19165696aa41002778524b3ecf82938",
		// HackFisher
		"0xb62d88e3f439fe9b5ea799b27bf7c6db5e795de1784f27b1bc051553499e420f",
		// WoeOm
		"0x0331760198d850b159844f3bfa620f6e704167973213154aca27675f7ddd987e",
		// yanganto
		"0xc45f075b5b1aa0145c469f57bd741c02272c1c0c41e9518d5a32426030d98232",
	]
	.iter()
	.map(|s| fixed_hex_bytes_unchecked!(s, 32).into())
	.collect::<Vec<_>>();

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_babe: Some(Default::default()),
		pallet_indices: Some(Default::default()),
		darwinia_balances_Instance0: Some(BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, C_RING_ENDOWMENT))
				.chain(
					vec![
						(root.clone(), 25_000_000 * COIN),
						(multi_sig, 700_000_000 * COIN),
						(genesis_validator.0.clone(), C_RING_ENDOWMENT),
					]
					.into_iter(),
				)
				.collect(),
		}),
		darwinia_balances_Instance1: Some(KtonConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, C_KTON_ENDOWMENT))
				.collect(),
		}),
		darwinia_staking: Some(StakingConfig {
			minimum_validator_count: 1,
			validator_count: 15,
			stakers: vec![(
				genesis_validator.0.clone(),
				genesis_validator.1.clone(),
				COIN,
				StakerStatus::Validator
			)],
			force_era: Forcing::ForceNew,
			slash_reward_fraction: Perbill::from_percent(10),
			payout_fraction: Perbill::from_percent(50),
			..Default::default()
		}),
		pallet_session: Some(SessionConfig {
			keys: vec![(
				genesis_validator.0.clone(),
				genesis_validator.0,
				crab_session_keys(
					genesis_validator.2,
					genesis_validator.3,
					genesis_validator.4,
					genesis_validator.5
				)
			)]
		}),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(Default::default()),
		darwinia_democracy: Some(Default::default()),
		pallet_collective_Instance0: Some(Default::default()),
		pallet_collective_Instance1: Some(Default::default()),
		darwinia_elections_phragmen: Some(Default::default()),
		pallet_membership_Instance0: Some(Default::default()),
		darwinia_claims: Some({
			ClaimsConfig {
				claims_list: ClaimsList::from_file(
					"node/service/res/crab/claims-list.json",
					"CLAIMS_LIST_PATH",
				),
			}
		}),
		pallet_sudo: Some(SudoConfig { key: root }),
		darwinia_ethereum_backing: Some(EthereumBackingConfig {
			token_redeem_address: fixed_hex_bytes_unchecked!(TOKEN_REDEEM_ADDRESS, 20).into(),
			deposit_redeem_address: fixed_hex_bytes_unchecked!(DEPOSIT_REDEEM_ADDRESS, 20).into(),
			ring_token_address: fixed_hex_bytes_unchecked!(RING_TOKEN_ADDRESS, 20).into(),
			kton_token_address: fixed_hex_bytes_unchecked!(KTON_TOKEN_ADDRESS, 20).into(),
			ring_locked: 7_569_833 * COIN,
			kton_locked: 30_000 * COIN,
			..Default::default()
		}),
		darwinia_ethereum_relay: Some(EthereumRelayConfig {
			genesis_header_info: (
				vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71, 128, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 33, 123, 11, 188, 251, 114, 226, 213, 126, 40, 243, 60, 179, 97, 185, 152, 53, 19, 23, 119, 85, 220, 63, 51, 206, 62, 112, 34, 237, 98, 183, 123, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 132, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 136, 0, 0, 0, 0, 0, 0, 0, 66, 1, 65, 148, 16, 35, 104, 9, 35, 224, 254, 77, 116, 163, 75, 218, 200, 20, 31, 37, 64, 227, 174, 144, 98, 55, 24, 228, 125, 102, 209, 202, 74, 45],
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: DagsMerkleRootsLoader::from_file(
				"node/service/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		}),
		darwinia_crab_issuing: Some(CrabIssuingConfig {
			total_mapped_ring: 40_000_000 * COIN,
		}),
		pallet_evm: Some(Default::default()),
		dvm_ethereum: Some(Default::default()),
	}
}

/// Crab config.
pub fn crab_build_spec_config() -> CrabChainSpec {
	let boot_nodes = vec![
		"/dns/g1.p2p.crab.darwinia.network/tcp/30333/p2p/12D3KooWFqHZkyv6iabxxqiHdNjWb4c7EfmBqMNCyqLCCVZm8yyQ".parse().unwrap(),
		"/dns/g2.p2p.crab.darwinia.network/tcp/30333/p2p/12D3KooWPiza2NAD6CjdBGtfUd3pfDnZXysYKzumejGHafW3Y8xP".parse().unwrap()
	];

	CrabChainSpec::from_genesis(
		"Crab",
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

/// Helper function to create Crab GenesisConfig for testing
pub fn crab_testnet_genesis(
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
	const TOKEN_REDEEM_ADDRESS: &'static str = "0x49262B932E439271d05634c32978294C7Ea15d0C";
	const DEPOSIT_REDEEM_ADDRESS: &'static str = "0x6EF538314829EfA8386Fc43386cB13B4e0A67D1e";
	const RING_TOKEN_ADDRESS: &'static str = "0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0";
	const KTON_TOKEN_ADDRESS: &'static str = "0x1994100c58753793D52c6f457f189aa3ce9cEe94";

	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(testnet_accounts);

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_babe: Some(Default::default()),
		pallet_indices: Some(Default::default()),
		darwinia_balances_Instance0: Some(BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 56))
				.collect(),
		}),
		darwinia_balances_Instance1: Some(KtonConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 56))
				.collect(),
		}),
		darwinia_staking: Some(StakingConfig {
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
		}),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0.clone(), x.0, crab_session_keys(x.2, x.3, x.4, x.5)))
				.collect(),
		}),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(Default::default()),
		darwinia_democracy: Some(Default::default()),
		pallet_collective_Instance0: Some(Default::default()),
		pallet_collective_Instance1: Some(Default::default()),
		darwinia_elections_phragmen: Some(Default::default()),
		pallet_membership_Instance0: Some(Default::default()),
		darwinia_claims: Some({
			ClaimsConfig {
				claims_list: ClaimsList::from_file(
					"node/service/res/crab/claims-list.json",
					"CLAIMS_LIST_PATH",
				),
			}
		}),
		pallet_sudo: Some(SudoConfig { key: root }),
		darwinia_ethereum_backing: Some(EthereumBackingConfig {
			token_redeem_address: fixed_hex_bytes_unchecked!(TOKEN_REDEEM_ADDRESS, 20).into(),
			deposit_redeem_address: fixed_hex_bytes_unchecked!(DEPOSIT_REDEEM_ADDRESS, 20).into(),
			ring_token_address: fixed_hex_bytes_unchecked!(RING_TOKEN_ADDRESS, 20).into(),
			kton_token_address: fixed_hex_bytes_unchecked!(KTON_TOKEN_ADDRESS, 20).into(),
			ring_locked: 1 << 56,
			kton_locked: 1 << 56,
			..Default::default()
		}),
		darwinia_ethereum_relay: Some(EthereumRelayConfig {
			genesis_header_info: (
				vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71, 128, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 33, 123, 11, 188, 251, 114, 226, 213, 126, 40, 243, 60, 179, 97, 185, 152, 53, 19, 23, 119, 85, 220, 63, 51, 206, 62, 112, 34, 237, 98, 183, 123, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 132, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 136, 0, 0, 0, 0, 0, 0, 0, 66, 1, 65, 148, 16, 35, 104, 9, 35, 224, 254, 77, 116, 163, 75, 218, 200, 20, 31, 37, 64, 227, 174, 144, 98, 55, 24, 228, 125, 102, 209, 202, 74, 45],
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: DagsMerkleRootsLoader::from_file(
				"node/service/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		}),
		darwinia_crab_issuing: Some(CrabIssuingConfig {
			total_mapped_ring: 1 << 56
		}),
		pallet_evm: Some(Default::default()),
		dvm_ethereum: Some(Default::default()),
	}
}

/// Crab development config (single validator Alice)
pub fn crab_development_config() -> CrabChainSpec {
	fn crab_development_genesis() -> GenesisConfig {
		crab_testnet_genesis(
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
