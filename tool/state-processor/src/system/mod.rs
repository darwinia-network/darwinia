// crates.io
use parity_scale_codec::Encode;
// darwinia
use crate::*;

#[derive(Debug)]
pub struct AccountAll {
	pub nonce: u32,
	pub consumers: RefCount,
	pub providers: RefCount,
	pub sufficients: RefCount,
	pub ring: Balance,
	pub ring_locks: Vec<BalanceLock>,
	pub kton: Balance,
	pub kton_locks: Vec<BalanceLock>,
}

impl<S> Processor<S>
where
	S: Configurable,
{
	pub fn process_system(&mut self) -> &mut Self {
		// System storage items.
		// https://github.dev/darwinia-network/substrate/blob/darwinia-v0.12.5/frame/system/src/lib.rs#L545
		let mut solo_account_infos = self.process_solo_account_infos();
		let mut para_account_infos = self.process_para_account_infos();
		let (ring_total_issuance_storage, kton_total_issuance_storage) = self.process_balances();
		let (solo_validators, para_collators) = self.process_session();
		let mut accounts = Map::default();
		let mut ring_total_issuance = Balance::default();
		let mut kton_total_issuance = Balance::default();

		// Skip for testnets, due to https://github.com/paritytech/substrate/issues/13172.
		// log::info!("decrease solo pallet-session account references");
		// solo_validators.into_iter().for_each(|k| {
		// 	if let Some(a) = solo_account_infos.get_mut(&k) {
		// 		a.consumers -= 1;
		// 	}
		// });

		// Skip, due to https://github.com/paritytech/substrate/issues/13172.
		// log::info!("decrease para pallet-session account references");
		// para_collators.into_iter().for_each(|k| {
		// 	if let Some(a) = para_account_infos.get_mut(&k) {
		// 		dbg!(get_last_64(&k));
		// 		a.consumers -= 1;
		// 	}
		// });

		log::info!("build accounts");
		log::info!("calculate total issuance");
		solo_account_infos.into_iter().for_each(|(k, v)| {
			let ring = v.data.free + v.data.reserved;
			let kton = v.data.free_kton_or_misc_frozen + v.data.reserved_kton_or_fee_frozen;

			accounts.insert(
				k,
				AccountAll {
					nonce: v.nonce,
					// ---
					// TODO: check if we could ignore para's.
					consumers: v.consumers,
					providers: v.providers,
					sufficients: v.sufficients,
					// ---
					ring,
					ring_locks: Default::default(),
					kton,
					kton_locks: Default::default(),
				},
			);

			ring_total_issuance += ring;
			kton_total_issuance += kton;
		});
		para_account_infos.into_iter().for_each(|(k, v)| {
			let ring = v.data.free + v.data.reserved;

			accounts
				.entry(k)
				.and_modify(|a| {
					a.nonce = v.nonce.max(a.nonce);
					a.ring += ring;
				})
				.or_insert(AccountAll {
					nonce: v.nonce,
					consumers: v.consumers,
					providers: v.providers,
					sufficients: v.sufficients,
					ring,
					ring_locks: Default::default(),
					kton: Default::default(),
					kton_locks: Default::default(),
				});

			ring_total_issuance += ring;
		});

		log::info!("burn parachain backing ring");
		if let Some(a) = accounts.get_mut(&blake2_128_concat_to_string(
			array_bytes::hex2array_unchecked::<_, 32>(S::PARACHAIN_BACKING),
		)) {
			ring_total_issuance -= a.ring;
			a.ring = 0;
		}

		log::info!("`ring_total_issuance({ring_total_issuance})`");
		log::info!("`ring_total_issuance_storage({ring_total_issuance_storage})`");

		log::info!("set `Balances::TotalIssuance`");
		self.shell_state.insert_value(b"Balances", b"TotalIssuance", "", ring_total_issuance);

		log::info!("`kton_total_issuance({kton_total_issuance})`");
		log::info!("`kton_total_issuance_storage({kton_total_issuance_storage})`");

		let mut kton_details = AssetDetails {
			owner: ROOT,
			issuer: ROOT,
			admin: ROOT,
			freezer: ROOT,
			supply: kton_total_issuance,
			deposit: Default::default(),
			min_balance: 1,      // The same as the value in the runtime.
			is_sufficient: true, // The same as the value in the runtime.
			sufficients: Default::default(),
			accounts: Default::default(),
			approvals: Default::default(),
			is_frozen: false,
		};

		log::info!("fix `EVM::AccountCodes`'s `sufficients` and set `Assets::Account`, `System::Account`, `AccountMigration::KtonAccounts` and `AccountMigration::Accounts`");
		accounts.into_iter().for_each(|(k, v)| {
			let key = get_last_64(&k);
			let mut a = AccountInfo {
				nonce: v.nonce,
				consumers: v.consumers,
				providers: v.providers,
				sufficients: v.sufficients,
				data: AccountData {
					free: v.ring,
					reserved: Default::default(),
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

				if v.kton != 0 {
					self.shell_state.insert_value(
						b"Assets",
						b"Account",
						&format!(
							"{}{}",
							blake2_128_concat_to_string(KTON_ID.encode()),
							blake2_128_concat_to_string(k.encode()),
						),
						new_kton_account(&mut a, &mut kton_details, v.kton),
					);
				}

				self.shell_state.insert_value(
					b"System",
					b"Account",
					&blake2_128_concat_to_string(k),
					a,
				);
			} else {
				a.nonce = 0;

				if v.kton != 0 {
					self.shell_state.insert_value(
						b"AccountMigration",
						b"KtonAccounts",
						&k,
						new_kton_account(&mut a, &mut kton_details, v.kton),
					);
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
			&blake2_128_concat_to_string(KTON_ID.encode()),
			kton_details,
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
		let (mut total_remaining_ring, mut total_remaining_kton) =
			(u128::default(), u128::default());

		remaining_ring.into_iter().for_each(|(k, v)| {
			if let Some(a) = account_infos.get_mut(&k) {
				total_remaining_ring += v;
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
				total_remaining_kton += v;
				a.data.free_kton_or_misc_frozen += v;
			} else {
				log::error!(
					"`Account({})` not found while merging `RemainingKtonBalance`",
					get_last_64(&k),
				);
			}
		});

		log::info!("total_remaining_ring({total_remaining_ring})");
		log::info!("total_remaining_kton({total_remaining_kton})");

		account_infos
	}

	fn process_para_account_infos(&mut self) -> Map<AccountInfo> {
		let mut account_infos = <Map<AccountInfo>>::default();

		log::info!("take para `System::Account`");
		self.para_state.take_map(b"System", b"Account", &mut account_infos, get_hashed_key);

		account_infos
	}
}

fn try_get_evm_address(key: &str) -> Option<AccountId20> {
	let k = array_bytes::hex2bytes_unchecked(key);

	if is_evm_address(&k) {
		Some(array_bytes::slice2array_unchecked(&k[11..31]))
	} else {
		None
	}
}

fn new_kton_account(
	account_info: &mut AccountInfo,
	asset_details: &mut AssetDetails,
	balance: Balance,
) -> AssetAccount {
	account_info.sufficients += 1;
	asset_details.accounts += 1;
	asset_details.sufficients += 1;

	AssetAccount { balance, is_frozen: false, reason: ExistenceReason::Sufficient, extra: () }
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
