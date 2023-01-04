// darwinia
use crate::*;

impl<S> Processor<S> {
	pub fn process_staking(&mut self) -> &mut Self {
		// Storage items.
		// https://github.dev/darwinia-network/darwinia-common/blob/darwinia-v0.12.5/frame/staking/src/lib.rs#L560
		let mut ledgers = <Map<StakingLedger>>::default();
		let mut ring_pool_storage = u128::default();
		let mut kton_pool_storage = u128::default();
		let mut ring_pool = u128::default();
		let mut kton_pool = u128::default();
		let mut elapsed_time = u64::default();

		log::info!("take solo `Staking::Ledger`, `Staking::RingPool`, `Staking::KtonPool` and `Staking::LivingTime`");
		self.solo_state
			.take_map(b"Staking", b"Ledger", &mut ledgers, get_identity_key)
			.take_value(b"Staking", b"RingPool", "", &mut ring_pool_storage)
			.take_value(b"Staking", b"KtonPool", "", &mut kton_pool_storage)
			.take_value(b"Staking", b"LivingTime", "", &mut elapsed_time);

		log::info!("adjust decimals and block number, convert ledger, adjust unstaking duration then set `AccountMigration::Ledgers` and `AccountMigration::Deposits`");
		{
			let staking_ik = item_key(b"AccountMigration", b"Ledgers");
			let deposit_ik = item_key(b"AccountMigration", b"Deposits");

			ledgers.into_iter().for_each(|(_, mut v)| {
				v.adjust();

				let hash_k = blake2_128_concat_to_string(v.stash);
				let deposit_k = format!("{deposit_ik}{hash_k}");
				let staking_k = format!("{staking_ik}{hash_k}");
				let mut staked_deposits = Vec::default();

				if !v.deposit_items.is_empty() {
					let mut deposit_ring = u128::default();

					self.shell_state.insert_raw_key_value(
						deposit_k,
						v.deposit_items
							.into_iter()
							.enumerate()
							.map(|(i, d)| {
								let id = i as _;

								staked_deposits.push(id);
								deposit_ring += d.value;

								Deposit {
									id,
									value: d.value,
									expired_time: d.expire_time as _,
									in_use: true,
								}
							})
							.collect::<Vec<_>>(),
					);
				}

				ring_pool += v.active;
				kton_pool += v.active_kton;

				self.shell_state.insert_raw_key_value(
					staking_k,
					Ledger {
						staked_ring: v.active,
						staked_kton: v.active_kton,
						staked_deposits,
						unstaking_ring: v
							.ring_staking_lock
							.unbondings
							.into_iter()
							.map(|u| (u.amount, u.until))
							.collect(),
						unstaking_kton: v
							.kton_staking_lock
							.unbondings
							.into_iter()
							.map(|u| (u.amount, u.until))
							.collect(),
						unstaking_deposits: Default::default(),
					},
				);
			});
		}

		ring_pool_storage.adjust();
		kton_pool_storage.adjust();

		log::info!("`ring_pool({ring_pool})`");
		log::info!("`ring_pool_storage({ring_pool_storage})`");
		log::info!("`kton_pool({kton_pool})`");
		log::info!("`kton_pool_storage({kton_pool_storage})`");

		log::info!("set `Staking::RingPool` and `Staking::KtonPool`");
		self.shell_state.insert_value(b"Staking", b"RingPool", "", ring_pool).insert_value(
			b"Staking",
			b"KtonPool",
			"",
			kton_pool,
		);

		log::info!("set `Staking::ElapsedTime`");
		self.shell_state.insert_value(b"Staking", b"ElapsedTime", "", elapsed_time as u128);

		self
	}
}
