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
use array_bytes::fixed_hex_bytes_unchecked;
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
	properties.insert("tokenDecimals".into(), 9.into());
	properties.insert("tokenSymbol".into(), "RING".into());
	properties.insert("ktonTokenDecimals".into(), 9.into());
	properties.insert("ktonTokenSymbol".into(), "KTON".into());

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
	const GENESIS_VALIDATOR_STASH: &'static str =
		"0xb4f7f03bebc56ebe96bc52ea5ed3159d45a0ce3a8d7f082983c33ef133274747";
	const GENESIS_VALIDATOR_CONTROLLER: &'static str =
		"0x7e450358b1768b8cc1df515292a97ac9f14f3f2ec9705a7352ec70b380c7fa60";
	const GENESIS_VALIDATOR_SESSION: &'static str =
		"0x0ae0f956e21c3f0ca9ea9121b41a1c1fc567f6ba6ce8abfed000073bb3352511";
	const GENESIS_VALIDATOR_GRANDPA: &'static str =
		"0x14342647be14beb21000d518a326be1e9b01d96ef1415148043e4ae2c726d463";

	let mut rings = BTreeMap::new();
	let mut ktons = BTreeMap::new();
	let mut swapped_ring_for_crab = 0;
	let mut da_crabk_endowed = false;
	let mut root_endowed = false;
	let mut genesis_validator_stash_endowed = false;

	// Initialize Crab genesis swap
	for (address, ring) in
		genesis_loader::load_genesis_swap_from_file("node/service/res/darwinia/swapped-cring.json")
			.unwrap()
	{
		match format!("0x{}", address).as_ref() {
			// MULTI_SIGN => multi_sign_endowed = true,
			ROOT => root_endowed = true,
			GENESIS_VALIDATOR_STASH => genesis_validator_stash_endowed = true,
			_ => (),
		}

		rings
			.entry(fixed_hex_bytes_unchecked!(address, 32).into())
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
			.entry(fixed_hex_bytes_unchecked!(address, 32).into())
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
			.entry(fixed_hex_bytes_unchecked!(address, 32).into())
			.and_modify(|kton_| *kton_ += kton)
			.or_insert(kton);
	}

	// Important account MUST be initialized
	assert!(da_crabk_endowed);
	assert!(root_endowed);
	assert!(genesis_validator_stash_endowed);

	let root: AccountId = fixed_hex_bytes_unchecked!(ROOT, 32).into();
	let da_crabk: AccountId = fixed_hex_bytes_unchecked!(DA_CRABK, 32).into();
	let team_vesting: AccountId = fixed_hex_bytes_unchecked!(TEAM_VESTING, 32).into();
	let foundation_vesting: AccountId = fixed_hex_bytes_unchecked!(FOUNDATION_VESTING, 32).into();
	let genesis_validator: (
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		AuthorityDiscoveryId,
	) = {
		let stash = fixed_hex_bytes_unchecked!(GENESIS_VALIDATOR_STASH, 32);
		let controller = fixed_hex_bytes_unchecked!(GENESIS_VALIDATOR_CONTROLLER, 32);
		let session = fixed_hex_bytes_unchecked!(GENESIS_VALIDATOR_SESSION, 32);
		let grandpa = fixed_hex_bytes_unchecked!(GENESIS_VALIDATOR_GRANDPA, 32);

		(
			stash.into(),
			controller.into(),
			session.unchecked_into(),
			grandpa.unchecked_into(),
			session.unchecked_into(),
			session.unchecked_into(),
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
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_babe: Some(Default::default()),
		darwinia_balances_Instance0: Some(BalancesConfig { balances: rings.into_iter().collect() }),
		darwinia_balances_Instance1: Some(KtonConfig { balances: ktons.into_iter().collect() }),
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
				darwinia_session_keys(
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
		pallet_collective_Instance0: Some(Default::default()),
		pallet_collective_Instance1: Some(Default::default()),
		darwinia_elections_phragmen: Some(Default::default()),
		pallet_membership_Instance0: Some(Default::default()),
		darwinia_vesting: Some(VestingConfig {
			vesting: vec![
				// Team vesting: 1 year period start after 1 year since mainnet lanuch
				(team_vesting, 365 * DAYS, 365 * DAYS, 0),
				// Foundation vesting: 5 years period start when mainnet launch
				(foundation_vesting, 0, (5.00_f64 * 365.25_f64) as BlockNumber * DAYS, 0)
			]
		}),
		pallet_sudo: Some(SudoConfig { key: root }),
		darwinia_ethereum_backing: Some(EthereumBackingConfig {
			// Los Angeles: 9/24/2020, 7:42:52 PM
			// Berlin :     9/25/2020, 10:42:52 AM
			// Beijing:     9/25/2020, 9:42:52 AM
			// New York :   9/24/2020, 9:42:52 PM
			ring_locked: 1_141_998_248_692_824_029_753_349_753_u128 / COIN + 1,
			kton_locked: 55_760_225_171_204_355_332_737_128 / COIN + 1,
			..Default::default()
		}),
		darwinia_ethereum_relay: Some(EthereumRelayConfig {
			genesis_header_info: (
				vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71, 128, 17, 187, 232, 219, 78, 52, 123, 78, 140, 147, 124, 28, 131, 112, 228, 181, 237, 51, 173, 179, 219, 105, 203, 219, 122, 56, 225, 229, 11, 27, 130, 250, 215, 248, 151, 79, 181, 172, 120, 217, 172, 9, 155, 154, 213, 1, 139, 237, 194, 206, 10, 114, 218, 209, 130, 122, 23, 9, 218, 48, 88, 15, 5, 68, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 136, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 132, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 136, 0, 0, 0, 0, 0, 0, 0, 66, 1, 212, 229, 103, 64, 248, 118, 174, 248, 192, 16, 184, 106, 64, 213, 245, 103, 69, 161, 24, 208, 144, 106, 52, 230, 154, 236, 140, 13, 177, 203, 143, 163],
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: DagsMerkleRootsLoader::from_file(
				"node/service/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		}),
		darwinia_tron_backing: Some(TronBackingConfig {
			// Los Angeles: 9/24/2020, 7:42:52 PM
			// Berlin :     9/25/2020, 10:42:52 AM
			// Beijing:     9/25/2020, 9:42:52 AM
			// New York :   9/24/2020, 9:42:52 PM
			backed_ring: 90_403_994_952_547_849_178_882_078_u128 / COIN + 1,
			backed_kton: 1_357_120_581_926_771_954_238_u128 / COIN + 1,
		}),
	}
}

/// Darwinia config.
pub fn darwinia_build_spec_config() -> DarwiniaChainSpec {
	let boot_nodes = vec![
		"/dns4/g1.p2p.cc1.darwinia.network/tcp/30333/p2p/12D3KooWANEQE69Td86QUy68Lim3rZR5mxsMviGYdi14ErzCfdht".parse().unwrap(),
		"/dns4/g2.p2p.cc1.darwinia.network/tcp/30333/p2p/12D3KooWBxWFD4zdSd2HQTxXNysJ7s248PsKjKKW4DnyiS47i49D".parse().unwrap()
	];

	DarwiniaChainSpec::from_genesis(
		"Darwinia Devnet",
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
	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(testnet_accounts);

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_babe: Some(Default::default()),
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
				.map(|x| (x.0.clone(), x.0, darwinia_session_keys(x.2, x.3, x.4, x.5)))
				.collect(),
		}),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(Default::default()),
		pallet_collective_Instance0: Some(Default::default()),
		pallet_collective_Instance1: Some(Default::default()),
		darwinia_elections_phragmen: Some(Default::default()),
		pallet_membership_Instance0: Some(Default::default()),
		darwinia_vesting: Some(Default::default()),
		pallet_sudo: Some(SudoConfig { key: root }),
		darwinia_ethereum_backing: Some(EthereumBackingConfig {
			ring_locked: 1 << 56,
			kton_locked: 1 << 56,
			..Default::default()
		}),
		darwinia_ethereum_relay: Some(EthereumRelayConfig {
			genesis_header_info: (
				vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71, 128, 17, 187, 232, 219, 78, 52, 123, 78, 140, 147, 124, 28, 131, 112, 228, 181, 237, 51, 173, 179, 219, 105, 203, 219, 122, 56, 225, 229, 11, 27, 130, 250, 215, 248, 151, 79, 181, 172, 120, 217, 172, 9, 155, 154, 213, 1, 139, 237, 194, 206, 10, 114, 218, 209, 130, 122, 23, 9, 218, 48, 88, 15, 5, 68, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 136, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 132, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 136, 0, 0, 0, 0, 0, 0, 0, 66, 1, 212, 229, 103, 64, 248, 118, 174, 248, 192, 16, 184, 106, 64, 213, 245, 103, 69, 161, 24, 208, 144, 106, 52, 230, 154, 236, 140, 13, 177, 203, 143, 163],
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: DagsMerkleRootsLoader::from_file(
				"node/service/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		}),
		darwinia_tron_backing: Some(TronBackingConfig {
			backed_ring: 1 << 56,
			backed_kton: 1 << 56,
		}),
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
