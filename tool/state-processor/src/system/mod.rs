// crates.io
use array_bytes::bytes2hex;
// darwinia
use crate::*;
// parity
use frame_support::{Blake2_128Concat, StorageHasher};
use sp_core::H160;

#[derive(Debug)]
pub struct AccountAll {
	pub key: String,
	pub nonce: u32,
	pub consumers: u32,
	pub providers: u32,
	pub sufficients: u32,
	pub ring: u128,
	pub ring_reserved: u128,
	pub ring_locks: Vec<BalanceLock>,
	pub kton: u128,
	pub kton_reserved: u128,
	pub kton_locks: Vec<BalanceLock>,
}

impl Processor {
	// System storage items.
	// https://github.com/paritytech/substrate/blob/polkadot-v0.9.16/frame/system/src/lib.rs#L545-L639
	// Balances storage items.
	// https://github.com/paritytech/substrate/blob/polkadot-v0.9.16/frame/balances/src/lib.rs#L486-L535
	pub fn process_system(&mut self) -> &mut Self {
		let solo_account_infos = self.process_solo_account_infos();
		let para_account_infos = self.process_para_account_infos();
		let (ring_total_issuance_storage, kton_total_issuance_storage) = self.process_balances();
		let mut accounts = Map::default();
		let mut ring_total_issuance = u128::default();
		let mut kton_total_issuance = u128::default();

		log::info!("build accounts");
		log::info!("calculate total issuance");
		solo_account_infos.into_iter().for_each(|(k, v)| {
			accounts.insert(
				k.clone(),
				AccountAll {
					key: k,
					nonce: v.nonce,
					// ---
					// TODO: check if we could ignore para's.
					consumers: v.consumers,
					providers: v.providers,
					sufficients: v.sufficients,
					// ---
					ring: v.data.free,
					ring_reserved: v.data.reserved,
					ring_locks: Default::default(),
					kton: v.data.free_kton_or_misc_frozen,
					kton_reserved: v.data.reserved_kton_or_fee_frozen,
					kton_locks: Default::default(),
				},
			);

			ring_total_issuance += v.data.free;
			ring_total_issuance += v.data.reserved;
			kton_total_issuance += v.data.free_kton_or_misc_frozen;
			kton_total_issuance += v.data.reserved_kton_or_fee_frozen;
		});
		para_account_infos.into_iter().for_each(|(k, v)| {
			accounts
				.entry(k.clone())
				.and_modify(|a| {
					a.nonce = v.nonce.max(a.nonce);
					a.ring += v.data.free;
					a.ring_reserved += v.data.reserved;
				})
				.or_insert(AccountAll {
					key: k,
					nonce: v.nonce,
					consumers: v.consumers,
					providers: v.providers,
					sufficients: v.sufficients,
					ring: v.data.free,
					ring_reserved: v.data.reserved,
					ring_locks: Default::default(),
					kton: 0,
					kton_reserved: 0,
					kton_locks: Default::default(),
				});

			ring_total_issuance += v.data.free;
			ring_total_issuance += v.data.reserved;
		});

		log::info!("set `Balances::TotalIssuance`");
		log::info!("ring_total_issuance({ring_total_issuance})");
		log::info!("ring_total_issuance_storage({ring_total_issuance_storage})");
		self.shell_state
			.0
			.insert(item_key(b"Balances", b"TotalIssuance"), encode_value(ring_total_issuance));

		log::info!("kton_total_issuance({kton_total_issuance})");
		log::info!("kton_total_issuance_storage({kton_total_issuance_storage})");
		// TODO: set KTON total issuance

		log::info!("update ring misc frozen and fee frozen");
		log::info!("set `System::Account`");
		log::info!("set `Balances::Locks`");
		accounts.into_iter().for_each(|(k, v)| {
			let key = get_last_64(&k);
			let mut a = AccountInfo {
				nonce: v.nonce,
				consumers: v.consumers,
				providers: v.providers,
				sufficients: v.sufficients,
				data: AccountData {
					free: v.ring,
					reserved: v.ring_reserved,
					free_kton_or_misc_frozen: Default::default(),
					reserved_kton_or_fee_frozen: Default::default(),
				},
			};

			match is_evm_addr(&key) {
				(true, addr) => {
					self.shell_state.0.insert(
						full_key(
							b"System",
							b"Account",
							&bytes2hex("", &Blake2_128Concat::hash(&addr.encode())),
						),
						encode_value(a),
					);
					// TODO: migrate kton balances.
				},
				(false, None) => {
					a.nonce = 0;

					self.shell_state
						.0
						.insert(full_key(b"AccountMigration", b"Accounts", &key), encode_value(a));
				},
				_ => unreachable!(),
			}
		});

		self
	}

	fn process_solo_account_infos(&mut self) -> Map<AccountInfo> {
		let mut account_infos = <Map<AccountInfo>>::default();
		let mut remaining_ring = <Map<u128>>::default();
		let mut remaining_kton = <Map<u128>>::default();

		log::info!("take solo `System::Account`, `Ethereum::RemainingRingBalance` and `Ethereum::RemainingKtonBalance`");
		self.solo_state
			.take_map(b"System", b"Account", &mut account_infos, get_hashed_key)
			.take_map(b"Ethereum", b"RemainingRingBalance", &mut remaining_ring, get_hashed_key)
			.take_map(b"Ethereum", b"RemainingKtonBalance", &mut remaining_kton, get_hashed_key);

		log::info!("adjust solo balance decimals");
		account_infos.iter_mut().for_each(|(_, v)| {
			v.data.free *= GWEI;
			v.data.reserved *= GWEI;
			v.data.free_kton_or_misc_frozen *= GWEI;
			v.data.reserved_kton_or_fee_frozen *= GWEI;
		});

		log::info!("merge solo remaining balances");
		remaining_ring.into_iter().for_each(|(k, v)| {
			if let Some(a) = account_infos.get_mut(&k) {
				a.data.free += v;
			} else {
				log::error!(
					"`Account({})` not found while merging `RemainingRingBalance`",
					get_last_64(&k)
				);
			}
		});
		remaining_kton.into_iter().for_each(|(k, v)| {
			if let Some(a) = account_infos.get_mut(&k) {
				a.data.free_kton_or_misc_frozen += v;
			} else {
				log::error!(
					"`Account({})` not found while merging `RemainingKtonBalance`",
					get_last_64(&k)
				);
			}
		});

		account_infos
	}

	fn process_para_account_infos(&mut self) -> Map<AccountInfo> {
		let mut account_infos = <Map<AccountInfo>>::default();

		log::info!("take para `System::Account`");
		self.para_state.take_map(b"System", b"Account", &mut account_infos, get_hashed_key);

		account_infos
	}
}

// Returns true if the key is an EVM account key and the associated address, otherwise, returns None
fn is_evm_addr(key: &str) -> (bool, Option<H160>) {
	let k = array_bytes::hex2bytes_unchecked(key);

	if k.starts_with(b"dvm:") && k[1..31].iter().fold(k[0], |checksum, &b| checksum ^ b) == k[31] {
		return (true, Some(H160::from_slice(&k[11..31])));
	}
	(false, None)
}

#[test]
fn verify_evm_address_checksum_should_work() {
	// std
	use std::str::FromStr;

	// subalfred key 5ELRpquT7C3mWtjerpPfdmaGoSh12BL2gFCv2WczEcv6E1zL
	// sub-seed
	// public-key 0x64766d3a00000000000000b7de7f8c52ac75e036d05fda53a75cf12714a76973
	// Substrate 5ELRpquT7C3mWtjerpPfdmaGoSh12BL2gFCv2WczEcv6E1zL
	assert!(is_evm_addr("0x64766d3a00000000000000b7de7f8c52ac75e036d05fda53a75cf12714a76973").0);
	assert_eq!(
		is_evm_addr("0x64766d3a00000000000000b7de7f8c52ac75e036d05fda53a75cf12714a76973").1,
		H160::from_str("b7de7f8c52ac75e036d05fda53a75cf12714a769").ok(),
	);
}