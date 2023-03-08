// crates.io
use once_cell::sync::Lazy;
use parity_scale_codec::Encode;
use primitive_types::H256;
// darwinia
use crate::*;

static T: Lazy<Tester> = Lazy::new(|| Tester::new());

struct Tester {
	// solo chain
	solo_accounts: Map<AccountInfo>,
	solo_remaining_ring: Map<Balance>,
	solo_remaining_kton: Map<Balance>,
	solo_evm_codes: Map<Vec<u8>>,
	// para chain
	para_accounts: Map<AccountInfo>,
	// processed
	migration_accounts: Map<AccountInfo>,
	migration_kton_accounts: Map<AssetAccount>,
	shell_system_accounts: Map<AccountInfo>,
	shell_evm_codes: Map<Vec<u8>>,

	solo_state: State<Crab>,
	para_state: State<()>,
	shell_state: State<()>,
}
impl Tester {
	fn new() -> Self {
		// This test is only used to ensure the correctness of the state processor and is only
		// applicable to Crab, Crab Parachain.
		<Processor<Crab>>::new().unwrap().test().process().save().unwrap();

		let mut solo_state = State::from_file("data/crab-solo.json").unwrap();
		let mut para_state = State::from_file("data/crab-para.json").unwrap();
		let mut shell_state = State::from_file("data/crab-processed.json").unwrap();

		// solo chain
		let mut solo_accounts = <Map<AccountInfo>>::default();
		let mut solo_remaining_ring = <Map<u128>>::default();
		let mut solo_remaining_kton = <Map<u128>>::default();
		let mut solo_evm_codes = <Map<Vec<u8>>>::default();
		solo_state
			.take_map(b"System", b"Account", &mut solo_accounts, get_last_64_key)
			.take_map(
				b"Ethereum",
				b"RemainingRingBalance",
				&mut solo_remaining_ring,
				get_last_64_key,
			)
			.take_map(
				b"Ethereum",
				b"RemainingKtonBalance",
				&mut solo_remaining_kton,
				get_last_64_key,
			)
			.take_map(b"EVM", b"AccountCodes", &mut solo_evm_codes, get_last_40);

		// para chain
		let mut para_accounts = <Map<AccountInfo>>::default();
		para_state.take_map(b"System", b"Account", &mut para_accounts, get_last_64_key);

		// processed
		let mut shell_system_accounts = <Map<AccountInfo>>::default();
		let mut shell_evm_codes = <Map<Vec<u8>>>::default();
		let mut migration_accounts = <Map<AccountInfo>>::default();
		let mut migration_kton_accounts = <Map<AssetAccount>>::default();
		shell_state
			.take_map(b"System", b"Account", &mut shell_system_accounts, get_last_40)
			.take_map(
				b"AccountMigration",
				b"KtonAccounts",
				&mut migration_kton_accounts,
				get_last_64_key,
			)
			.take_map(b"AccountMigration", b"Accounts", &mut migration_accounts, get_last_64_key)
			.take_map(b"EVM", b"AccountCodes", &mut shell_evm_codes, get_last_40);

		Self {
			solo_accounts,
			solo_remaining_ring,
			solo_remaining_kton,
			solo_evm_codes,

			para_accounts,

			migration_accounts,
			migration_kton_accounts,
			shell_system_accounts,
			shell_evm_codes,

			solo_state,
			para_state,
			shell_state,
		}
	}
}

fn two_x64_concat_to_string<D>(data: D) -> String
where
	D: AsRef<[u8]>,
{
	array_bytes::bytes2hex("", subhasher::twox64_concat(data))
}

fn get_last_40(key: &str, _: &str) -> String {
	get_last_n(key, 40)
}

fn run_test<T>(test: T)
where
	T: FnOnce(&Tester),
{
	test(&T);
}

// --- System & Balances & Assets ---

#[test]
fn solo_chain_substrate_account() {
	run_test(|tester| {
		// the purest account
		{
			let addr = "0xcefc2d75238c60c67b4b02867f061a7c36f9b37f625854d4e5daac2cbf58f42d";
			let solo_account = tester.solo_accounts.get(addr).unwrap();
			assert_eq!(solo_account.nonce, 0);
			assert_eq!(solo_account.consumers, 0);
			assert_eq!(solo_account.providers, 1);
			assert_eq!(solo_account.sufficients, 0);
			assert_ne!(solo_account.data.free, 0);
			assert_eq!(solo_account.data.reserved, 0);

			// after migrate

			let m_account = tester.migration_accounts.get(addr).unwrap();
			assert_eq!(m_account.nonce, 0);
			assert_eq!(m_account.consumers, 0);
			assert_eq!(m_account.providers, 1);
			assert_eq!(m_account.sufficients, 0);
			assert_eq!(m_account.data.free, solo_account.data.free * GWEI);
			assert_eq!(m_account.data.reserved, 0);
		}

		// account nonce reset
		{
			let addr = "0x82e54b190ef8dbe7864b7cdca3de6b3e8b5483e0a3e9419ba2a41a16531aaa0b";
			let solo_account = tester.solo_accounts.get(addr).unwrap();
			assert_ne!(solo_account.nonce, 0);

			// after migrate

			let m_account = tester.migration_accounts.get(addr).unwrap();
			assert_eq!(m_account.nonce, 0);
		}

		// account staking without deposit items
		{
			let addr = "0x82e54b190ef8dbe7864b7cdca3de6b3e8b5483e0a3e9419ba2a41a16531aaa0b";
			let solo_account = tester.solo_accounts.get(addr).unwrap();
			assert_eq!(solo_account.consumers, 2);

			// after migrate

			let m_account = tester.migration_accounts.get(addr).unwrap();
			assert_eq!(m_account.consumers, 1);
		}

		// account has kton with ledger and deposit items
		{
			let addr = "0xf4171e1b64c96cc17f601f28d002cb5fcd27eab8b6585e296f4652be5bf05550";
			let solo_account = tester.solo_accounts.get(addr).unwrap();
			assert_eq!(solo_account.consumers, 3);
			assert_eq!(solo_account.providers, 1);
			assert_eq!(solo_account.sufficients, 0);
			assert_ne!(solo_account.data.free, 0);
			assert_ne!(solo_account.data.free_kton_or_misc_frozen, 0);

			// after migrate

			let m_account = tester.migration_accounts.get(addr).unwrap();
			assert_eq!(m_account.consumers, 2);
			assert_eq!(m_account.providers, 1);
			assert_eq!(m_account.sufficients, 1);
			// 30_976_316_716_418_406_400 is the unclaimed staking reward
			assert_eq!(m_account.data.free, solo_account.data.free * GWEI + 30_976_316_716_418_406_400);
			assert_eq!(m_account.data.free_kton_or_misc_frozen, 0);
			//  the kton part moved to the asset pallet
			let asset_account = tester.migration_kton_accounts.get(addr).unwrap();
			assert_eq!(asset_account.balance, solo_account.data.free_kton_or_misc_frozen * GWEI);
			assert!(!asset_account.is_frozen);
		}
	});
}

#[test]
fn solo_chain_substrate_account_with_remaining_balance() {
	run_test(|tester| {
		let addr = "0xfe129f56cc498227acacc4231f70ae15a2f4e8f9ccfa51f4de268c75516fa350";

		let solo_account = tester.solo_accounts.get(addr).unwrap();
		let remaining_balance = tester.solo_remaining_ring.get(addr).unwrap();
		assert_ne!(*remaining_balance, 0);

		// after migrate

		let m_account = tester.migration_accounts.get(addr).unwrap();
		assert_eq!(
			m_account.data.free + m_account.data.reserved,
			(solo_account.data.free + solo_account.data.reserved) * GWEI + remaining_balance
		);
	});
}

#[test]
fn combine_solo_and_para_account() {
	run_test(|tester| {
		let addr = "0x2a997fbf3423723ab73fae76567b320de6979664cb3287c0e6ce24099d0eff68";

		// solo
		let solo_account = tester.solo_accounts.get(addr).unwrap();
		let remaining_balance = tester.solo_remaining_ring.get(addr).unwrap();
		assert_ne!(solo_account.nonce, 0);
		// para
		let para_account = tester.para_accounts.get(addr).unwrap();
		assert_ne!(para_account.nonce, 0);

		// after migrate

		let m_account = tester.migration_accounts.get(addr).unwrap();
		assert_eq!(
			m_account.data.free,
			solo_account.data.free * GWEI + remaining_balance + para_account.data.free
		);
		// reset the nonce
		assert_eq!(m_account.nonce, 0);
	});
}

#[test]
fn evm_account() {
	run_test(|tester| {
		let addr = "0x64766d3a00000000000000aef71b03670f1c52cd3d8efc2ced3ad68ad91e33f3";

		let solo_account = tester.solo_accounts.get(addr).unwrap();
		assert_ne!(solo_account.nonce, 0);
		assert_ne!(solo_account.data.free, 0);
		assert_ne!(solo_account.data.free_kton_or_misc_frozen, 0);
		let solo_remaining_kton = tester.solo_remaining_kton.get(addr).unwrap();
		assert_ne!(*solo_remaining_kton, 0);

		// after migrate

		let m_addr = "0xaef71b03670f1c52cd3d8efc2ced3ad68ad91e33";
		let m_account = tester.shell_system_accounts.get(m_addr).unwrap();
		// nonce doesn't changed.
		assert_eq!(m_account.nonce, solo_account.nonce);
		assert_eq!(m_account.consumers, solo_account.consumers);
		assert_eq!(m_account.providers, solo_account.providers);
		// sufficient increase by one because of the asset pallet.
		assert_eq!(m_account.sufficients, solo_account.sufficients + 1);
		assert_eq!(m_account.data.free, solo_account.data.free * GWEI);
		assert_eq!(m_account.data.free_kton_or_misc_frozen, 0);

		//  the kton part moved to the asset pallet
		let mut asset_account = AssetAccount::default();
		let m_addr =
			array_bytes::hex2array_unchecked::<_, 20>("0xaef71b03670f1c52cd3d8efc2ced3ad68ad91e33");
		tester.shell_state.get_value(
			b"Assets",
			b"Account",
			&format!(
				"{}{}",
				blake2_128_concat_to_string(KTON_ID.encode()),
				blake2_128_concat_to_string(m_addr.encode()),
			),
			&mut asset_account,
		);
		assert_eq!(
			asset_account.balance,
			solo_account.data.free_kton_or_misc_frozen * GWEI + solo_remaining_kton
		);
		assert!(!asset_account.is_frozen);
	});
}

#[test]
fn evm_contract_account_sufficients() {
	run_test(|tester| {
		let addr = "0x64766d3a000000000000000050f880c35c31c13bfd9cbb7d28aafaeca3abd2d0";
		let solo_account = tester.solo_accounts.get(addr).unwrap();
		assert_eq!(solo_account.sufficients, 0);

		// after migrate

		let m_addr = "0x0050f880c35c31c13bfd9cbb7d28aafaeca3abd2";
		let m_account = tester.shell_system_accounts.get(m_addr).unwrap();
		assert_eq!(m_account.sufficients, 1);
	});
}

#[test]
fn ring_total_issuance() {
	run_test(|tester| {
		let mut solo_issuance = u128::default();
		let mut para_issuance = u128::default();

		tester.solo_state.get_value(b"Balances", b"TotalIssuance", "", &mut solo_issuance);
		assert_ne!(solo_issuance, 0);
		tester.para_state.get_value(b"Balances", b"TotalIssuance", "", &mut para_issuance);
		assert_ne!(para_issuance, 0);

		// after migrate

		let mut migrated_total_issuance = u128::default();
		tester.shell_state.get_value(
			b"Balances",
			b"TotalIssuance",
			"",
			&mut migrated_total_issuance,
		);

		assert_eq!(
			migrated_total_issuance - 161_223_151_710_u128,
			solo_issuance * GWEI + para_issuance
		);
	});
}

#[test]
fn kton_total_issuance() {
	run_test(|tester| {
		let mut total_issuance = u128::default();
		tester.solo_state.get_value(b"Kton", b"TotalIssuance", "", &mut total_issuance);
		assert_ne!(total_issuance, 0);

		// after migrate

		let mut migrated_total_issuance = u128::default();
		tester.shell_state.get_value(
			b"Balances",
			b"TotalIssuance",
			"",
			&mut migrated_total_issuance,
		);

		let mut details = AssetDetails::default();
		tester.shell_state.get_value(
			b"Assets",
			b"Asset",
			&blake2_128_concat_to_string(KTON_ID.encode()),
			&mut details,
		);
		assert_eq!(details.supply - 5_999_999_999_u128, total_issuance * GWEI);
	});
}

#[test]
fn asset_creation() {
	run_test(|tester| {
		let mut details = AssetDetails::default();
		tester.shell_state.get_value(
			b"Assets",
			b"Asset",
			&blake2_128_concat_to_string(KTON_ID.encode()),
			&mut details,
		);
		assert_eq!(details.owner, ROOT);
		assert!(details.supply != 0);
		assert_eq!(details.deposit, 0);
		assert_eq!(details.min_balance, 1);
		assert_eq!(details.is_sufficient, true);
		assert_eq!(details.sufficients, details.accounts);
		assert!(details.accounts > 0);
		assert_eq!(details.approvals, 0);
		assert_eq!(details.status, AssetStatus::Live);

		let total_kton_accounts = tester
			.solo_accounts
			.iter()
			.filter(|(k, a)| {
				a.data.free_kton_or_misc_frozen != 0
					|| a.data.reserved_kton_or_fee_frozen != 0
					|| tester.solo_remaining_kton.get(k.as_str()).map(|i| *i).unwrap_or_default()
						!= 0
			})
			.count();
		assert_eq!(
			total_kton_accounts as u32,
			details.sufficients + tester.migration_kton_accounts.len() as u32
		);
	});
}

#[test]
fn asset_metadata() {
	run_test(|tester| {
		let mut metadata = AssetMetadata::default();
		tester.shell_state.get_value(
			b"Assets",
			b"Metadata",
			&blake2_128_concat_to_string(KTON_ID.encode()),
			&mut metadata,
		);
		assert_eq!(metadata.decimals, 18);
		assert_eq!(metadata.symbol, b"CKTON".to_vec());
		assert_eq!(metadata.name, b"Crab Commitment Token".to_vec());
	});
}

#[test]
fn identities_reservation() {
	run_test(|tester| {
		{
			// https://crab.subscan.io/account/5CXHjmXetspzSTWci8UKXnPjBeJGpibitrWX7fDDMqbggyap
			let addr = "0x14466f29bc873ce014367d897940e3a4d4a22c1c70d83469bcd7647e921d1557";

			let solo_account = tester.solo_accounts.get(addr).unwrap();
			let remaining = tester.solo_remaining_ring.get(addr).unwrap();
			assert_eq!(solo_account.data.reserved, 10000000000);
			let total = (solo_account.data.free + solo_account.data.reserved) * GWEI + remaining;

			// after migrate

			let m_account = tester.migration_accounts.get(addr).unwrap();
			assert_eq!(m_account.data.reserved, 100025800000000000000);
			assert_eq!(m_account.data.reserved + m_account.data.free, total);
		}

		// can not afford the latest reservation amount
		{
			// https://crab.subscan.io/account/5HTysESF4MCRABBJ2Pmm8Sx3JrJToQgz1nwiBctGXGUKZLeP
			let addr = "0xeeedb4805e781b16db87edc6fc2bb0982bf70a435e6a5acac37ede09131d8b8b";

			let solo_account = tester.solo_accounts.get(addr).unwrap();
			assert_ne!(solo_account.data.free, 0);
			assert_eq!(solo_account.data.reserved, 10000000000);
			let total = (solo_account.data.free + solo_account.data.reserved) * GWEI;

			// after migrate

			let m_account = tester.migration_accounts.get(addr).unwrap();
			assert_eq!(m_account.data.free, 0);
			assert_eq!(m_account.data.reserved, 10800000000000000000);
			assert_eq!(m_account.data.reserved + m_account.data.free, total);
		}
	});
}

#[test]
fn special_accounts() {
	run_test(|tester| {
		{
			// sibling:2023
			// https://crab-parachain.subscan.io/account/5Eg2fntNdR73Xc3d8jr1954TYnT173qFgSZmDhMTn2K4ewsT
			let addr_1 = "0x7369626ce7070000000000000000000000000000000000000000000000000000";
			let para_account_1 = tester.para_accounts.get(addr_1).unwrap();
			assert_ne!(para_account_1.data.free, 0);

			// sibling:2000
			// https://crab-parachain.subscan.io/account/5Eg2fntJ27qsari4FGrGhrMqKFDRnkNSR6UshkZYBGXmSuC8
			let addr_2 = "0x7369626cd0070000000000000000000000000000000000000000000000000000";
			let para_account_2 = tester.para_accounts.get(addr_2).unwrap();
			assert_ne!(para_account_2.data.free, 0);

			// sibling:2004
			// https://crab-parachain.subscan.io/account/5Eg2fntJpc4bfLRHXnmBUav7hms6NC6vowNAEBDnYSfeMPCw
			let addr_3 = "0x7369626cd4070000000000000000000000000000000000000000000000000000";
			let para_account_3 = tester.para_accounts.get(addr_3).unwrap();
			assert_ne!(para_account_3.data.free, 0);

			// after migrate

			let m_account_1 = tester.shell_system_accounts.get(&addr_1[..42]).unwrap();
			assert_eq!(m_account_1.data.free, para_account_1.data.free);
			let m_account_2 = tester.shell_system_accounts.get(&addr_2[..42]).unwrap();
			assert_eq!(m_account_2.data.free, para_account_2.data.free);
			let m_account_3 = tester.shell_system_accounts.get(&addr_3[..42]).unwrap();
			assert_eq!(m_account_3.data.free, para_account_3.data.free);
		}

		{
			// PalletId(da/paais)
			let addr_1 = "0x6d6f646c64612f70616169730000000000000000000000000000000000000000";
			let para_account_1 = tester.para_accounts.get(addr_1).unwrap();
			assert!(para_account_1.providers > 1);

			// PalletId(PotStake)
			let addr_2 = "0x6d6f646c506f745374616b650000000000000000000000000000000000000000";
			let para_account_2 = tester.para_accounts.get(addr_2).unwrap();
			assert_eq!(para_account_2.data.free, 1);

			// PalletId(da/ethrl)
			let addr_3 = "0x6d6f646c64612f657468726c0000000000000000000000000000000000000000";
			let solo_account_3 = tester.solo_accounts.get(addr_3).unwrap();
			assert_ne!(solo_account_3.data.free, 0);

			// PalletId(da/trsry)
			let addr_4 = "0x6d6f646c64612f74727372790000000000000000000000000000000000000000";
			let solo_account_4 = tester.solo_accounts.get(addr_4).unwrap();
			let para_account_4 = tester.para_accounts.get(addr_4).unwrap();
			assert_ne!(solo_account_4.data.free, 0);
			assert_ne!(para_account_4.data.free, 0);

			// PalletId(py/trsry)
			let addr_5 = "0x6d6f646c70792f74727372790000000000000000000000000000000000000000";
			let solo_account_5 = tester.solo_accounts.get(addr_5).unwrap();
			assert_ne!(solo_account_5.data.free, 0);
			assert_ne!(solo_account_5.data.free_kton_or_misc_frozen, 0);

			// after migrate

			let m_account_1 = tester.shell_system_accounts.get(&addr_1[..42]).unwrap();
			assert_eq!(m_account_1.providers, 1);

			let m_account_2 = tester.shell_system_accounts.get(&addr_2[..42]).unwrap();
			assert_eq!(m_account_2.data.free, para_account_2.data.free);

			let m_account_3 = tester.shell_system_accounts.get(&addr_3[..42]).unwrap();
			assert_eq!(m_account_3.data.free, solo_account_3.data.free * GWEI);

			let m_account_4 = tester.shell_system_accounts.get(&addr_4[..42]).unwrap();
			assert_eq!(
				m_account_4.data.free,
				solo_account_4.data.free * GWEI + para_account_4.data.free
			);

			let m_account_5 = tester.shell_system_accounts.get(&addr_5[..42]).unwrap();
			assert_eq!(m_account_5.data.free, solo_account_5.data.free * GWEI);
			assert_eq!(m_account_5.data.free_kton_or_misc_frozen, 0);
			let m_addr = array_bytes::hex2array_unchecked::<_, 20>(&addr_5[..42]);
			let mut asset_account = AssetAccount::default();
			tester.shell_state.get_value(
				b"Assets",
				b"Account",
				&format!(
					"{}{}",
					blake2_128_concat_to_string(KTON_ID.encode()),
					blake2_128_concat_to_string(m_addr.encode()),
				),
				&mut asset_account,
			);
			assert_eq!(asset_account.balance, solo_account_5.data.free_kton_or_misc_frozen * GWEI);
		}

		{
			let addr = "0x0000000000000000000000000000000000000000000000000000000000000000";
			let solo_account = tester.solo_accounts.get(addr).unwrap();
			assert_ne!(solo_account.data.free, 0);
			assert_ne!(solo_account.data.reserved, 0);

			// after migrate

			let m_account = tester.shell_system_accounts.get(&addr[..42]).unwrap();
			assert_eq!(
				m_account.data.free,
				(solo_account.data.free + solo_account.data.reserved) * GWEI
			);
		}
	});
}

// --- EVM & Ethereum ---

#[test]
fn evm_code() {
	run_test(|tester| {
		{
			let addr = "0x0050f880c35c31c13bfd9cbb7d28aafaeca3abd2";

			let code = tester.solo_evm_codes.get(addr).unwrap();
			assert_ne!(code.len(), 0);

			// after migrate

			let migrated_code = tester.shell_evm_codes.get(addr).unwrap();
			assert_eq!(*code, *migrated_code);
		}

		{
			tester.solo_evm_codes.iter().for_each(|(k, v)| {
				assert_eq!(tester.shell_evm_codes.get(k), Some(v));
			});
		}
	});
}

#[test]
fn precompiles_code() {
	run_test(|tester| {
		let addrs = ["001", "009", "400", "402", "600", "601"];

		for i in addrs {
			let addr = format!("{}{i}", "0x0000000000000000000000000000000000000");
			assert_eq!(tester.shell_evm_codes.get(&addr), Some(&[96, 0, 96, 0, 253].to_vec()));
		}
	});
}

#[test]
fn evm_contract_account_storage() {
	run_test(|tester| {
		// https://crab.subscan.io/account/0x0050f880c35c31c13bfd9cbb7d28aafaeca3abd2
		let addr =
			array_bytes::hex2array_unchecked::<_, 20>("0x0050f880c35c31c13bfd9cbb7d28aafaeca3abd2");

		let storage_item_len = tester.solo_state.map.iter().fold(0u32, |sum, (k, _)| {
			if k.starts_with(&full_key(
				b"EVM",
				b"AccountStorages",
				&blake2_128_concat_to_string(addr.encode()),
			)) {
				sum + 1
			} else {
				sum
			}
		});
		assert_ne!(storage_item_len, 0);

		let storage_key = array_bytes::hex2array_unchecked::<_, 32>(
			"0x2093bcd1218dc1519493ee712ddfee3f4ced2d74096331d39d4247147baf17e2",
		);
		let mut storage_value = H256::zero();
		tester.solo_state.get_value(
			b"EVM",
			b"AccountStorages",
			&format!(
				"{}{}",
				&blake2_128_concat_to_string(addr.encode()),
				&blake2_128_concat_to_string(storage_key),
			),
			&mut storage_value,
		);
		assert_ne!(storage_value, H256::zero());

		// after migrate

		let migrated_storage_item_len = tester.shell_state.map.iter().fold(0u32, |sum, (k, _)| {
			if k.starts_with(&full_key(
				b"EVM",
				b"AccountStorages",
				&blake2_128_concat_to_string(addr.encode()),
			)) {
				sum + 1
			} else {
				sum
			}
		});
		assert_eq!(storage_item_len, migrated_storage_item_len);

		let mut migrated_storage_value = H256::zero();
		tester.shell_state.get_value(
			b"EVM",
			b"AccountStorages",
			&format!(
				"{}{}",
				&blake2_128_concat_to_string(addr.encode()),
				&blake2_128_concat_to_string(storage_key),
			),
			&mut migrated_storage_value,
		);
		assert_eq!(storage_value, migrated_storage_value);
	});
}

// --- Staking ---

#[test]
fn stake_deposit_items() {
	run_test(|tester| {
		// https://crab.subscan.io/account/5Dfh9agy74KFmdYqxNGEWae9fE9pdzYnyCUJKqK47Ac64zqM
		let addr = array_bytes::hex2array_unchecked::<_, 32>(
			"0x46eb701bdc7f74ffda9c4335d82b3ae8d4e52c5ac630e50d68ab99822e29b3f6",
		);

		let mut ledger = StakingLedger::default();
		tester.solo_state.get_value(
			b"Staking",
			b"Ledger",
			&blake2_128_concat_to_string(addr.encode()),
			&mut ledger,
		);
		assert_ne!(ledger.deposit_items.len(), 0);
		let deposits_sum: u128 = ledger.deposit_items.iter().map(|i| i.value).sum();

		// after migrate

		let mut m_deposits = Vec::<Deposit>::new();
		tester.shell_state.get_value(
			b"AccountMigration",
			b"Deposits",
			&blake2_128_concat_to_string(addr.encode()),
			&mut m_deposits,
		);
		assert_eq!(m_deposits.len(), ledger.deposit_items.len());
		ledger.deposit_items.iter().zip(m_deposits.iter()).for_each(|(old, new)| {
			assert_eq!(new.value, old.value * GWEI);
			assert_eq!(new.start_time, old.start_time as u128);
			assert_eq!(new.expired_time, old.expire_time as u128);
			assert!(new.in_use);
		});
		let migrated_deposits_sum: u128 = m_deposits.iter().map(|i| i.value).sum();
		assert_eq!(migrated_deposits_sum, deposits_sum * GWEI);
	});
}

#[test]
fn stake_ledgers_values() {
	run_test(|tester| {
		// https://crab.subscan.io/account/5Dfh9agy74KFmdYqxNGEWae9fE9pdzYnyCUJKqK47Ac64zqM
		let addr = array_bytes::hex2array_unchecked::<_, 32>(
			"0x46eb701bdc7f74ffda9c4335d82b3ae8d4e52c5ac630e50d68ab99822e29b3f6",
		);

		let mut ledger = StakingLedger::default();
		tester.solo_state.get_value(
			b"Staking",
			b"Ledger",
			&blake2_128_concat_to_string(addr.encode()),
			&mut ledger,
		);
		assert_ne!(ledger.active, 0);
		assert_ne!(ledger.active_kton, 0);

		// after migrate

		let mut m_ledger = Ledger::default();
		tester.shell_state.get_value(
			b"AccountMigration",
			b"Ledgers",
			&blake2_128_concat_to_string(addr.encode()),
			&mut m_ledger,
		);

		let mut m_deposits: Vec<Deposit> = Vec::new();
		tester.shell_state.get_value(
			b"AccountMigration",
			b"Deposits",
			&blake2_128_concat_to_string(addr.encode()),
			&mut m_deposits,
		);

		assert_eq!(ledger.active * GWEI, m_deposits.iter().map(|d| d.value).sum());
		assert_eq!(m_ledger.staked_ring, 0);
		assert_eq!(m_ledger.staked_kton, ledger.active_kton * GWEI);
	});
}

#[test]
fn stake_ledgers_unbonding() {
	run_test(|tester| {
		// https://crab.subscan.io/account/5FGL7pMZFZK4zWX2y3CRABeqMpMjBq77LhfYipWoBAT9gJsa
		let addr = array_bytes::hex2array_unchecked::<_, 32>(
			"0x8d92774046fd3dc60d41825023506ad5ad91bd0d66e9c1df325fc3cf89c2d317",
		);

		let mut ledger = StakingLedger::default();
		tester.solo_state.get_value(
			b"Staking",
			b"Ledger",
			&blake2_128_concat_to_string(addr.encode()),
			&mut ledger,
		);
		assert_ne!(ledger.ring_staking_lock.unbondings.len(), 0);

		// after migrate

		let mut m_ledger = Ledger::default();
		tester.shell_state.get_value(
			b"AccountMigration",
			b"Ledgers",
			&blake2_128_concat_to_string(addr.encode()),
			&mut m_ledger,
		);
		ledger.ring_staking_lock.unbondings.iter().zip(m_ledger.unstaking_ring.iter()).for_each(
			|(old, (amount, util))| {
				assert_eq!(*amount, old.amount * GWEI);
				assert!(*util < old.until);
			},
		);
	});
}

#[test]
fn stake_ring_pool() {
	run_test(|tester| {
		let mut ring_pool = u128::default();
		tester.solo_state.get_value(b"Staking", b"RingPool", "", &mut ring_pool);
		assert_ne!(ring_pool, 0);

		// after migrate
		let mut m_ring_pool = u128::default();
		tester.shell_state.get_value(b"Staking", b"RingPool", "", &mut m_ring_pool);
		assert_eq!(m_ring_pool, ring_pool * GWEI);
	});
}

#[test]
fn stake_kton_pool() {
	run_test(|tester| {
		let mut kton_pool = u128::default();
		tester.solo_state.get_value(b"Staking", b"KtonPool", "", &mut kton_pool);
		assert_ne!(kton_pool, 0);

		// after migrate

		let mut m_kton_pool = u128::default();
		tester.shell_state.get_value(b"Staking", b"KtonPool", "", &mut m_kton_pool);
		assert_eq!(m_kton_pool, kton_pool * GWEI);
	});
}

#[test]
fn stake_elapsed_time() {
	run_test(|tester| {
		let mut elapsed_time = u64::default();
		tester.solo_state.get_value(b"Staking", b"LivingTime", "", &mut elapsed_time);
		assert_ne!(elapsed_time, 0);

		// after migrate

		let mut m_elapsed_time = u128::default();
		tester.shell_state.get_value(b"Staking", b"ElapsedTime", "", &mut m_elapsed_time);
		assert_eq!(m_elapsed_time, elapsed_time as u128);
	});
}

// --- Vesting ---

#[test]
fn vesting_info() {
	run_test(|tester| {
		// https://crab.subscan.io/account/5EFJA3K6uRfkLxqjhHyrkJoQjfhmhyVyVEG5XtPPBM6yCCxM
		let addr = array_bytes::hex2array_unchecked::<_, 32>(
			"0x608c62275934b164899ca6270c4b89c5d84b2390d4316fda980cd1b3acfad525",
		);

		let mut vesting_info = VestingInfo::default();
		tester.solo_state.get_value(
			b"Vesting",
			b"Vesting",
			&blake2_128_concat_to_string(addr.encode()),
			&mut vesting_info,
		);
		assert_ne!(vesting_info.locked, 0);
		assert_ne!(vesting_info.starting_block, 0);

		// after migrate

		let mut m_vesting_info = VestingInfo::default();
		tester.shell_state.get_value(
			b"AccountMigration",
			b"Vestings",
			&blake2_128_concat_to_string(addr.encode()),
			&mut m_vesting_info,
		);

		assert_eq!(m_vesting_info.locked, vesting_info.locked * GWEI);
		assert_eq!(m_vesting_info.per_block, vesting_info.per_block * GWEI * 2);
		assert!(m_vesting_info.starting_block < vesting_info.starting_block);
	});
}

// --- Identity ---

#[test]
fn identities() {
	run_test(|tester| {
		// https://crab.subscan.io/account/5Ct3V8cbYgJiUoQQhYMyyWChL5YwJnZ4yak7MKegNkpPptAP
		let addr = array_bytes::hex2array_unchecked::<_, 32>(
			"0x241a9c2aa8a83e1c5f02fc2b7112bd1873249a8e55a4f919c7d42cf1164be35c",
		);

		let mut registration = Registration::default();
		tester.solo_state.get_value(
			b"Identity",
			b"IdentityOf",
			&two_x64_concat_to_string(addr.encode()),
			&mut registration,
		);
		assert_ne!(registration.deposit, 0);
		assert_eq!(registration.info.display, Data::Raw(b"iskulbukolPH".to_vec()));
		assert_eq!(registration.info.email, Data::Raw(b"pjdamondamon@gmail.com".to_vec()));
		assert_eq!(registration.info.twitter, Data::Raw(b"@DPedroJuan".to_vec()));

		// after migrate

		let mut m_registration = Registration::default();
		tester.shell_state.get_value(
			b"AccountMigration",
			b"Identities",
			&two_x64_concat_to_string(addr.encode()),
			&mut m_registration,
		);
		assert_eq!(m_registration.deposit, registration.deposit * GWEI);
		registration.judgements.iter().zip(m_registration.judgements.iter()).for_each(
			|((_, r), (_, m_r))| match (r, m_r) {
				(Judgement::FeePaid(a), Judgement::FeePaid(m_a)) => assert_eq!(a * GWEI, *m_a),
				_ => assert_eq!(*r, *m_r),
			},
		);
		assert_eq!(m_registration.info.display, registration.info.display);
		assert_eq!(m_registration.info.email, registration.info.email);
		assert_eq!(m_registration.info.twitter, registration.info.twitter);
	});
}

#[test]
fn registrars() {
	run_test(|tester| {
		let mut rs: Vec<Option<RegistrarInfo<[u8; 32]>>> = Vec::new();
		tester.solo_state.get_value(b"Identity", b"Registrars", "", &mut rs);
		assert!(!rs.is_empty());

		// after migrate

		let mut m_rs: Vec<Option<RegistrarInfo<[u8; 20]>>> = Vec::new();
		tester.shell_state.get_value(b"Identity", b"Registrars", "", &mut m_rs);

		rs.iter().zip(m_rs.iter()).for_each(|(r, m_r)| match (r, m_r) {
			(Some(r), Some(m_r)) => {
				assert_eq!(r.account[..20], m_r.account);
				assert_eq!(r.fee * GWEI, m_r.fee);
				assert_eq!(r.fields, m_r.fields);
			},
			(None, None) => (),
			_ => panic!("this should never happen!"),
		});
	});
}
