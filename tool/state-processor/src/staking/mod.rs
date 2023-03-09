// darwinia
use crate::*;

impl<S> Processor<S> {
	pub fn process_staking(&mut self) -> &mut Self {
		// Storage items.
		// https://github.dev/darwinia-network/darwinia-common/blob/darwinia-v0.12.5/frame/staking/src/lib.rs#L560
		let mut ledgers = <Map<StakingLedger>>::default();
		let mut ring_pool_storage = Balance::default();
		let mut kton_pool_storage = Balance::default();
		let mut ring_pool = Balance::default();
		let mut kton_pool = Balance::default();
		let mut elapsed_time = u64::default();
		let mut eras_stakers_clipped = <Map<Exposure>>::default();
		let mut active_era = ActiveEraInfo::default();
		let mut bonded = <Map<AccountId32>>::default();
		let mut era_reward_points = <Map<EraRewardPoints>>::default();
		let mut eras_validator_reward = <Map<Balance>>::default();
		let mut eras_validator_prefs = <Map<ValidatorPrefs>>::default();
		let mut total_staked = Balance::default();
		let mut total_deposit = Balance::default();

		log::info!("take solo `Staking::Ledger`, `Staking::RingPool`, `Staking::KtonPool` and `Staking::LivingTime`");
		self.solo_state
			.take_map(b"Staking", b"Ledger", &mut ledgers, get_last_64_key)
			.take_value(b"Staking", b"RingPool", "", &mut ring_pool_storage)
			.take_value(b"Staking", b"KtonPool", "", &mut kton_pool_storage)
			.take_value(b"Staking", b"LivingTime", "", &mut elapsed_time)
			.take_prefix(
				&item_key(b"Staking", b"ErasStakersClipped"),
				&mut eras_stakers_clipped,
				strip_prefix_key,
			)
			.take_value(b"Staking", b"ActiveEra", "", &mut active_era)
			.take_map(b"Staking", b"Bonded", &mut bonded, get_last_64_key)
			.take_map(
				b"Staking",
				b"ErasValidatorReward",
				&mut eras_validator_reward,
				get_last_8_key,
			)
			.take_map(b"Staking", b"ErasRewardPoints", &mut era_reward_points, get_last_8_key)
			.take_map(
				b"Staking",
				b"ErasValidatorReward",
				&mut eras_validator_reward,
				get_last_8_key,
			)
			.take_map(
				b"Staking",
				b"ErasValidatorPrefs",
				&mut eras_validator_prefs,
				strip_prefix_key,
			);

		log::info!("adjust decimals and block number, convert ledger, adjust unstaking duration then set `AccountMigration::Ledgers` and `AccountMigration::Deposits`");
		{
			let staking_ik = item_key(b"AccountMigration", b"Ledgers");
			let deposit_ik = item_key(b"AccountMigration", b"Deposits");

			for (_, mut v) in ledgers.clone() {
				if v.is_empty() {
					log::trace!(
						"clean empty ledger for Account({})",
						array_bytes::bytes2hex("0x", v.stash)
					);

					continue;
				}

				v.adjust();

				let hash_k = blake2_128_concat_to_string(v.stash);
				let deposit_k = format!("{deposit_ik}{hash_k}");
				let staking_k = format!("{staking_ik}{hash_k}");
				let mut consumers = 1;
				let mut staked_deposits = Vec::default();
				let mut deposit_ring = Balance::default();

				if !v.deposit_items.is_empty() {
					consumers += 1;

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
									start_time: d.start_time as _,
									expired_time: d.expire_time as _,
									in_use: true,
								}
							})
							.collect::<Vec<_>>(),
					);
				}

				ring_pool += v.active;
				kton_pool += v.active_kton;
				total_deposit += deposit_ring;
				total_staked += v.active - deposit_ring;

				self.shell_state.inc_consumers_by(&array_bytes::bytes2hex("", v.stash), consumers);
				self.shell_state.insert_raw_key_value(
					staking_k,
					Ledger {
						// Decoupling staking and deposit.
						staked_ring: v.active - deposit_ring,
						staked_kton: v.active_kton,
						staked_deposits,
						unstaking_ring: v
							.ring_staking_lock
							.unbondings
							.into_iter()
							.filter_map(
								// Clear the expired unbondings.
								//
								// Since we don't add any lock here.
								|u| if u.until == 0 { None } else { Some((u.amount, u.until)) },
							)
							.collect(),
						unstaking_kton: v
							.kton_staking_lock
							.unbondings
							.into_iter()
							.filter_map(
								// Clear the expired unbondings.
								//
								// Since we don't add any lock here.
								|u| if u.until == 0 { None } else { Some((u.amount, u.until)) },
							)
							.collect(),
						unstaking_deposits: Default::default(),
					},
				);
			}
		}

		ring_pool_storage.adjust();
		kton_pool_storage.adjust();

		log::info!("        `ring_pool({ring_pool})`");
		log::info!("`ring_pool_storage({ring_pool_storage})`");
		log::info!("        `kton_pool({kton_pool})`");
		log::info!("`kton_pool_storage({kton_pool_storage})`");
		log::info!("`total_deposit({total_deposit})`");
		log::info!("` total_staked({total_staked})`");

		log::info!("set `Staking::RingPool` and `Staking::KtonPool`");
		self.shell_state.insert_value(b"Staking", b"RingPool", "", ring_pool).insert_value(
			b"Staking",
			b"KtonPool",
			"",
			kton_pool,
		);

		log::info!("set `Staking::ElapsedTime`");
		self.shell_state.insert_value(b"Staking", b"ElapsedTime", "", elapsed_time as Moment);

		log::info!("make payout");

		let eras_stakers_clipped = build_double_map(
			eras_stakers_clipped,
			// twox64_concat: 16 + u32
			16..24,
			// twox64_concat: 16 + [u8; 32]
			40..104,
		);
		let eras_validator_prefs = build_double_map(
			eras_validator_prefs,
			// twox64_concat: 16 + u32
			16..24,
			// twox64_concat: 16 + [u8; 32]
			40..104,
		);

		// subalfred key --type pallet da/staki
		// sub-seed PalletId(da/staki)
		// public-key 0x6d6f646c64612f7374616b690000000000000000000000000000000000000000
		// Substrate 5EYCAe5gKAhKZcQsCCDnRgeVRrnyjZwG7WkBqzQdqFxQ8T9W
		//
		// Truncate to 20 bytes.
		let staking_pallet_account = "0x6d6f646c64612f7374616b690000000000000000";
		let mut payout = Balance::default();

		for (era_raw, stakers) in eras_stakers_clipped {
			let era = u32::from_le_bytes(array_bytes::hex2array_unchecked(&era_raw));

			if era == active_era.index {
				// The inflation for current era doesn't happen yet, skip.
				continue;
			}

			let era_payout = {
				let mut p = *eras_validator_reward.get(&era_raw).unwrap();

				p.adjust();

				p as f64
			};

			for (stash, exposure) in stakers {
				let controller = array_bytes::bytes2hex("0x", bonded.get(&stash).unwrap());
				let ledger = ledgers.get(&controller).unwrap();

				if ledger.claimed_rewards.contains(&era) {
					// Already claimed.
					continue;
				}

				let era_reward_points = era_reward_points.get(&era_raw).unwrap();
				let Some(validator_reward_points) = era_reward_points.individual.get(&ledger.stash).cloned() else {
					// No reward points.
					continue;
				};
				let total_reward_points = era_reward_points.total;
				let validator_total_reward_part =
					validator_reward_points as f64 / total_reward_points as f64;
				let validator_total_payout = era_payout * validator_total_reward_part;
				let validator_prefs =
					eras_validator_prefs.get(&era_raw).unwrap().get(&stash).unwrap();
				// `Perbill` to `f64`.
				let validator_commission = validator_prefs.commission as f64 / 1_000_000_000_f64;
				let validator_commission_payout = validator_total_payout * validator_commission;
				let validator_leftover_payout =
					validator_total_payout - validator_commission_payout;
				let validator_exposure_part =
					exposure.own_power as f64 / exposure.total_power as f64;
				let validator_staking_payout = validator_leftover_payout * validator_exposure_part;
				let validator_payout = validator_commission_payout + validator_staking_payout;

				payout += validator_payout as Balance;

				exposure.others.into_iter().for_each(|nominator| {
					let nominator_exposure_part =
						nominator.power as f64 / exposure.total_power as f64;
					let nominator_payout = validator_leftover_payout * nominator_exposure_part;

					payout += nominator_payout as Balance;

					let nominator = array_bytes::bytes2hex("0x", nominator.who);
					let nominator_payout = nominator_payout as Balance;

					log::trace!("pay `{nominator_payout}` to `nominator({nominator})`");

					self.shell_state.transfer(staking_pallet_account, &nominator, nominator_payout);
				});

				let validator_payout = validator_payout as Balance;

				log::trace!("pay `{validator_payout}` to `validator({stash})`");

				self.shell_state.transfer(staking_pallet_account, &stash, validator_payout);
			}
		}

		{
			let mut a = AccountInfo::default();

			self.shell_state.get_value(
				b"System",
				b"Account",
				&blake2_128_concat_to_string(array_bytes::hex2array_unchecked::<_, 20>(
					staking_pallet_account,
				)),
				&mut a,
			);

			log::info!("`staking_pallet_account_balance({})`", a.data.free);
			log::info!("                        `payout({payout})`");

			assert!(
				a.data.free > payout,
				"`staking_pallet_account_balance` must be greater then `payout`"
			);
		}

		self
	}
}
