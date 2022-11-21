// darwinia
use crate::*;

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
		let mut accounts = Map::default();
		let mut solo_account_infos = Map::default();
		let mut solo_ring_locks = Map::default();
		let mut solo_kton_locks = Map::default();
		let mut para_account_infos = Map::default();
		let mut remaining_ring = Map::default();
		let mut remaining_kton = Map::default();
		let mut ring_total_issuance = 0;

		log::info!("take solo and remaining balances");
		self.solo_state
			.take::<AccountInfo, _>(
				b"System",
				b"Account",
				&mut solo_account_infos,
				get_blake2_128_concat_suffix,
			)
			.take::<u128, _>(
				b"Ethereum",
				b"RemainingRingBalance",
				&mut remaining_ring,
				get_blake2_128_concat_suffix,
			)
			.take::<u128, _>(
				b"Ethereum",
				b"RemainingKtonBalance",
				&mut remaining_kton,
				get_blake2_128_concat_suffix,
			);

		log::info!("take solo and para balance locks");
		self.process_balances(&mut solo_ring_locks, &mut solo_kton_locks);

		log::info!("take para balances");
		self.para_state.take::<AccountInfo, _>(
			b"System",
			b"Account",
			&mut para_account_infos,
			get_blake2_128_concat_suffix,
		);

		log::info!("adjust solo balance decimals");
		solo_account_infos.iter_mut().for_each(|(_, v)| {
			v.data.free *= GWEI;
			v.data.reserved *= GWEI;
			v.data.free_kton_or_misc_frozen *= GWEI;
			v.data.reserved_kton_or_fee_frozen *= GWEI;
		});

		log::info!("merge solo and remaining balances");
		remaining_ring.into_iter().for_each(|(k, v)| {
			if let Some(a) = solo_account_infos.get_mut(&k) {
				a.data.free += v;
			} else {
				log::warn!("`RemainingRingBalance({k})` not found");
			}
		});
		remaining_kton.into_iter().for_each(|(k, v)| {
			if let Some(a) = solo_account_infos.get_mut(&k) {
				a.data.free_kton_or_misc_frozen += v;
			} else {
				log::warn!("`RemainingKtonBalance({k})` not found");
			}
		});

		log::info!("build accounts");
		log::info!("calculate ring total issuance");
		solo_account_infos.into_iter().for_each(|(k, v)| {
			let ring_locks = solo_ring_locks.remove(&k).unwrap_or_default();
			let kton_locks = solo_kton_locks.remove(&k).unwrap_or_default();

			ring_total_issuance += v.data.free;
			ring_total_issuance += v.data.reserved;

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
					ring_locks,
					kton: v.data.free_kton_or_misc_frozen,
					kton_reserved: v.data.reserved_kton_or_fee_frozen,
					kton_locks,
				},
			);
		});
		para_account_infos.into_iter().for_each(|(k, v)| {
			ring_total_issuance += v.data.free;
			ring_total_issuance += v.data.reserved;

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
					ring_locks: Vec::new(),
					kton: 0,
					kton_reserved: 0,
					kton_locks: Vec::new(),
				});
		});

		log::info!("check solo remaining locks");
		solo_ring_locks.into_iter().for_each(|(k, _)| log::warn!("ring_locks' owner({k}) dropped"));
		solo_kton_locks.into_iter().for_each(|(k, _)| log::warn!("kton_locks' owner({k}) dropped"));

		let state = &mut self.shell_chain_spec.genesis.raw.top;

		log::info!("set `Balances::TotalIssuance`");
		state.insert(item_key(b"Balances", b"TotalIssuance"), encode_value(ring_total_issuance));

		log::info!("update ring misc frozen and fee frozen");
		log::info!("set `System::Account`");
		log::info!("set `Balances::Locks`");
		accounts.into_iter().for_each(|(k, v)| {
			let mut a = AccountInfo {
				nonce: v.nonce,
				consumers: v.consumers,
				providers: v.providers,
				sufficients: v.sufficients,
				data: AccountData {
					free: v.ring,
					reserved: v.ring_reserved,
					free_kton_or_misc_frozen: 0,
					reserved_kton_or_fee_frozen: 0,
				},
			};

			// https://github.com/paritytech/substrate/blob/polkadot-v0.9.16/frame/balances/src/lib.rs#L945-L952
			// Update ring misc frozen and fee frozen.
			for l in v.ring_locks.iter() {
				if l.reasons == Reasons::All || l.reasons == Reasons::Misc {
					a.data.free_kton_or_misc_frozen = a.data.free_kton_or_misc_frozen.max(l.amount);
				}
				if l.reasons == Reasons::All || l.reasons == Reasons::Fee {
					a.data.reserved_kton_or_fee_frozen =
						a.data.reserved_kton_or_fee_frozen.max(l.amount);
				}
			}
			// ---
			// TODO: migrate kton locks.
			// ---

			// Set `System::Account`.
			state.insert(format!("{}{k}", item_key(b"System", b"Account")), encode_value(a));
			// Set `Balances::Locks`.
			// Skip empty locks.
			if !v.ring_locks.is_empty() {
				state.insert(
					format!("{}{k}", item_key(b"Balances", b"Locks")),
					encode_value(v.ring_locks),
				);
			}
		});

		self
	}
}
