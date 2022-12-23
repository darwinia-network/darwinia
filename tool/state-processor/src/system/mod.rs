// darwinia
use crate::*;
use subhasher::blake2_128_concat;

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
	pub fn process_system(&mut self) -> &mut Self {
		// System storage items.
		// https://github.dev/darwinia-network/substrate/blob/darwinia-v0.12.5/frame/system/src/lib.rs#L545
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

		log::info!("`ring_total_issuance({ring_total_issuance})`");
		log::info!("`ring_total_issuance_storage({ring_total_issuance_storage})`");

		log::info!("set `Balances::TotalIssuance`");
		self.shell_state.insert_value(b"Balances", b"TotalIssuance", "", ring_total_issuance);

		let mut kton_details = AssetDetails {
			owner: ROOT,
			issuer: ROOT,
			admin: ROOT,
			freezer: ROOT,
			supply: kton_total_issuance,
			deposit: 0,
			min_balance: 1,      // The same as the value in the runtime.
			is_sufficient: true, // The same as the value in the runtime.
			sufficients: 0,
			accounts: 0,
			approvals: 0,
			is_frozen: false,
		};

		log::info!("set `System::Account`");
		log::info!("set `Balances::Locks`");
		log::info!("set `Assets::Account` and `Assets::Approvals`");
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

			if let Some(k) = try_get_evm_address(&key) {
				// If the evm account is a contract contract with sufficients, then we should
				// increase the sufficients by one.
				if self.solo_state.contains_key(&full_key(
					b"EVM",
					b"AccountCodes",
					&blake2_128_concat_to_string(k),
				)) && a.sufficients == 0
				{
					a.sufficients += 1;
				}

				if v.kton != 0 || v.kton_reserved != 0 {
					let aa = AssetAccount {
						balance: v.kton,
						is_frozen: false,
						reason: ExistenceReason::Sufficient,
						extra: (),
					};

					a.sufficients += 1;
					kton_details.accounts += 1;
					kton_details.sufficients += 1;
					// Note: this is double map structure in the pallet-assets.
					self.shell_state.insert_value(
						b"Assets",
						b"Account",
						&format!(
							"{}{}",
							array_bytes::bytes2hex("", blake2_128_concat(&KTON_ID.encode())),
							array_bytes::bytes2hex("", blake2_128_concat(&k.encode())),
						),
						&aa,
					);

					// https://github.dev/darwinia-network/darwinia-common/blob/6a9392cfb9fe2c99b1c2b47d0c36125d61991bb7/frame/dvm/evm/precompiles/kton/src/lib.rs#L72
					let mut approves = Map::<primitive_types::U256>::default();
					self.solo_state.take_map(
						b"KtonERC20",
						b"Approves",
						&mut approves,
						get_hashed_key,
					);
					approves.iter().for_each(|(k, v)| {
						kton_details.approvals += 1;
						self.shell_state.insert_value(
							b"Assets",
							b"Approvals",
							&k,
							Approval { amount: v.as_u128(), deposit: 0 },
						);
					});
				}

				self.shell_state.insert_value(
					b"System",
					b"Account",
					&blake2_128_concat_to_string(k),
					a,
				);
			} else {
				a.nonce = 0;

				if v.kton != 0 || v.kton_reserved != 0 {
					let aa = AssetAccount {
						balance: v.kton,
						is_frozen: false,
						reason: ExistenceReason::Sufficient,
						extra: (),
					};

					self.shell_state.insert_value(b"AccountMigration", b"KtonAccounts", &k, &aa);
				}

				self.shell_state.insert_value(b"AccountMigration", b"Accounts", &k, a);
			}
		});

		log::info!("set `Assets::Asset`");
		log::info!("kton_total_issuance({kton_total_issuance})");
		log::info!("kton_total_issuance_storage({kton_total_issuance_storage})");
		self.shell_state.insert_value(
			b"Assets",
			b"Asset",
			&array_bytes::bytes2hex("", blake2_128_concat(&KTON_ID.encode())),
			kton_details,
		);

		log::info!("set `Assets::Metadata`");
		self.shell_state.insert_value(
			b"Assets",
			b"Metadata",
			&array_bytes::bytes2hex("", blake2_128_concat(&KTON_ID.encode())),
			AssetMetadata {
				deposit: 0,
				name: b"Darwinia Commitment Token".to_vec(),
				symbol: b"KTON".to_vec(),
				decimals: 18,
				is_frozen: false,
			},
		);

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

		log::info!("adjust solo `AccountData`s");
		account_infos.iter_mut().for_each(|(_, v)| v.data.adjust());

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

fn try_get_evm_address(key: &str) -> Option<[u8; 20]> {
	let k = array_bytes::hex2bytes_unchecked(key);

	if k.starts_with(b"dvm:") && k[1..31].iter().fold(k[0], |checksum, &b| checksum ^ b) == k[31] {
		Some(array_bytes::slice2array_unchecked(&k[11..31]))
	} else {
		None
	}
}

#[test]
fn verify_evm_address_checksum_should_work() {
	// subalfred key 5ELRpquT7C3mWtjerpPfdmaGoSh12BL2gFCv2WczEcv6E1zL
	// sub-seed
	// public-key 0x64766d3a00000000000000b7de7f8c52ac75e036d05fda53a75cf12714a76973
	// Substrate 5ELRpquT7C3mWtjerpPfdmaGoSh12BL2gFCv2WczEcv6E1zL
	assert_eq!(
		try_get_evm_address("0x64766d3a00000000000000b7de7f8c52ac75e036d05fda53a75cf12714a76973")
			.unwrap(),
		array_bytes::hex2array_unchecked::<_, 20>("0xb7de7f8c52ac75e036d05fda53a75cf12714a769")
	);
}
