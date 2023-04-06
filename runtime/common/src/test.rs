#[macro_export]
macro_rules! impl_account_migration_tests {
	() => {
		mod account_migration {
			// darwinia
			use super::mock::*;
			use darwinia_deposit::Deposit as DepositS;
			use darwinia_staking::Ledger;
			// substrate
			use frame_support::{
				assert_err, assert_ok, migration, Blake2_128Concat, StorageHasher,
			};
			use frame_system::AccountInfo;
			use pallet_assets::ExistenceReason;
			use pallet_balances::AccountData;
			use pallet_identity::{
				Data, IdentityFields, IdentityInfo, RegistrarInfo, Registration,
			};
			use sp_core::{sr25519::Pair, Decode, Encode, Get, Pair as PairT, H160};
			use sp_keyring::sr25519::Keyring;
			use sp_runtime::{
				traits::ValidateUnsigned,
				transaction_validity::{InvalidTransaction, TransactionValidityError},
				AccountId32, DispatchError,
			};
			use sp_version::RuntimeVersion;

			const RING_AMOUNT: u128 = 100;
			const KTON_AMOUNT: u128 = 100;

			#[derive(Debug, PartialEq, Eq)]
			enum E {
				T(TransactionValidityError),
				D(DispatchError),
			}
			use E::*;
			impl From<TransactionValidityError> for E {
				fn from(t: TransactionValidityError) -> Self {
					T(t)
				}
			}
			impl From<DispatchError> for E {
				fn from(d: DispatchError) -> Self {
					D(d)
				}
			}

			// This struct is private in `pallet-assets`.
			#[derive(Encode, Decode)]
			struct AssetAccount {
				balance: u128,
				is_frozen: bool,
				reason: ExistenceReason<u128>,
				extra: (),
			}
			// This struct is private in `pallet-assets`.
			#[derive(PartialEq, Eq, Encode, Decode)]
			struct AssetDetails {
				owner: AccountId,
				issuer: AccountId,
				admin: AccountId,
				freezer: AccountId,
				supply: Balance,
				deposit: Balance,
				min_balance: Balance,
				is_sufficient: bool,
				accounts: u32,
				sufficients: u32,
				approvals: u32,
				status: AssetStatus,
			}
			// This struct is private in `pallet-assets`.
			#[derive(PartialEq, Eq, Encode, Decode)]
			enum AssetStatus {
				Live,
				Frozen,
				Destroying,
			}

			// This struct is private in `pallet-vesting`.
			#[derive(Encode)]
			struct VestingInfo {
				locked: u128,
				per_block: u128,
				starting_block: u32,
			}

			fn alice() -> (Pair, AccountId32) {
				let pair = Keyring::Alice.pair();
				let public_key = AccountId32::new(pair.public().0);

				(pair, public_key)
			}

			fn invalid_transaction(code: u8) -> E {
				T(TransactionValidityError::Invalid(InvalidTransaction::Custom(code)))
			}

			fn preset_state_of(who: &Pair) {
				let account_id_32 = AccountId32::new(who.public().0);
				let asset_account = AssetAccount {
					balance: KTON_AMOUNT,
					is_frozen: false,
					reason: ExistenceReason::<u128>::Sufficient,
					extra: (),
				};

				assert!(AccountMigration::account_of(&account_id_32).is_none());
				assert!(AccountMigration::kton_account_of(&account_id_32).is_none());

				<darwinia_account_migration::Accounts<Runtime>>::insert(
					&account_id_32,
					AccountInfo {
						nonce: 100,
						consumers: 1,
						providers: 1,
						sufficients: 1,
						data: AccountData { free: RING_AMOUNT, ..Default::default() },
					},
				);
				migration::put_storage_value(
					b"AccountMigration",
					b"KtonAccounts",
					&Blake2_128Concat::hash(account_id_32.as_ref()),
					asset_account,
				);
				assert!(AccountMigration::account_of(&account_id_32).is_some());
				assert!(AccountMigration::kton_account_of(&account_id_32).is_some());
			}

			fn migrate(from: Pair, to: AccountId) -> Result<(), E> {
				let message = darwinia_account_migration::signable_message(
					<<Runtime as frame_system::Config>::Version as Get<RuntimeVersion>>::get()
						.spec_name
						.as_bytes(),
					&to,
				);
				let sig = from.sign(&message);
				let from_pk = AccountId32::new(from.public().0);

				AccountMigration::pre_dispatch(&darwinia_account_migration::Call::migrate {
					from: from_pk.clone(),
					to,
					signature: sig.0.clone(),
				})?;
				AccountMigration::migrate(RuntimeOrigin::none(), from_pk, to, sig.0)?;

				Ok(())
			}

			#[test]
			fn validate_substrate_account_not_found() {
				ExtBuilder::default().build().execute_with(|| {
					let (from, _) = alice();
					let to = AccountId::default();

					// Migration source doesn't exist.
					assert_err!(migrate(from, to), invalid_transaction(1));
				});
			}

			#[test]
			fn validate_evm_account_already_exist() {
				let (from, _) = alice();
				let to = H160::from_low_u64_be(33).into();

				ExtBuilder::default().with_balances(vec![(to, RING_AMOUNT)]).build().execute_with(
					|| {
						preset_state_of(&from);

						// Migration target has already been migrated.
						assert_err!(migrate(from, to), invalid_transaction(0));
					},
				);
			}

			#[test]
			fn validate_invalid_sig() {
				let (from, from_pk) = alice();
				let to = H160::from_low_u64_be(33).into();
				let message = darwinia_account_migration::signable_message(b"?", &to);
				let sig = from.sign(&message);

				ExtBuilder::default().build().execute_with(|| {
					preset_state_of(&from);

					assert_err!(
						AccountMigration::pre_dispatch(
							&darwinia_account_migration::Call::migrate {
								from: from_pk,
								to,
								signature: sig.0,
							}
						)
						.map_err(E::from),
						invalid_transaction(2)
					);
				});
			}

			#[test]
			fn migrate_accounts_should_work() {
				let (from, from_pk) = alice();
				let to = H160::from_low_u64_be(255).into();

				ExtBuilder::default().build().execute_with(|| {
					preset_state_of(&from);

					assert_ok!(migrate(from, to));
					assert_eq!(AccountMigration::account_of(from_pk), None);
					assert_eq!(
						System::account(to),
						AccountInfo {
							nonce: 100,
							consumers: 1,
							providers: 1,
							sufficients: 1,
							data: AccountData { free: RING_AMOUNT, ..Default::default() },
						}
					);
				});
			}

			#[test]
			fn migrate_kton_accounts_should_work() {
				let (from, from_pk) = alice();
				let to = H160::from_low_u64_be(255).into();

				let asset_details = || {
					migration::get_storage_value::<AssetDetails>(
						b"Assets",
						b"Asset",
						&Blake2_128Concat::hash(&KTON_ID.encode()),
					)
					.unwrap()
				};

				ExtBuilder::default().build().execute_with(|| {
					preset_state_of(&from);
					let pre_asset_details = asset_details();

					assert_ok!(migrate(from, to));
					let asset_details = asset_details();
					assert_eq!(AccountMigration::kton_account_of(from_pk), None);
					assert_eq!(Assets::maybe_balance(KTON_ID, to).unwrap(), KTON_AMOUNT);
					assert_eq!(pre_asset_details.accounts + 1, asset_details.accounts);
					assert_eq!(pre_asset_details.sufficients + 1, asset_details.sufficients);
					assert_eq!(pre_asset_details.owner, asset_details.owner);
					assert_eq!(pre_asset_details.supply, asset_details.supply);

					let actual_accounts = migration::storage_key_iter_with_suffix::<
						AccountId,
						AssetAccount,
						Blake2_128Concat,
					>(
						b"Assets",
						b"Account",
						&Blake2_128Concat::hash(&(KTON_ID as u64).encode()),
					)
					.count();
					assert_eq!(actual_accounts as u32, asset_details.accounts);
				});
			}

			#[test]
			fn vesting_should_work() {
				let (from, from_pk) = alice();
				let to = H160::from_low_u64_be(255).into();

				ExtBuilder::default().build().execute_with(|| {
					preset_state_of(&from);

					migration::put_storage_value(
						b"AccountMigration",
						b"Vestings",
						&Blake2_128Concat::hash(from_pk.as_ref()),
						vec![
							VestingInfo { locked: 100, per_block: 5, starting_block: 0 },
							VestingInfo { locked: 100, per_block: 5, starting_block: 0 },
						],
					);
					assert!(Vesting::vesting(to).is_none());
					assert!(Balances::locks(to).is_empty());

					assert_ok!(migrate(from, to));
					assert_eq!(Vesting::vesting(to).unwrap().len(), 2);
					assert_eq!(Balances::locks(to).len(), 1);
				});
			}

			#[test]
			fn staking_should_work() {
				let (from, from_pk) = alice();
				let init = H160::from_low_u64_be(254).into();
				let to = H160::from_low_u64_be(255).into();

				ExtBuilder::default()
					.with_assets_accounts(vec![(KTON_ID, init, KTON_AMOUNT)])
					.build()
					.execute_with(|| {
						preset_state_of(&from);

						<darwinia_account_migration::Deposits<Runtime>>::insert(
							&from_pk,
							vec![
								DepositS {
									id: 1,
									value: 10,
									start_time: 1000,
									expired_time: 2000,
									in_use: true,
								},
								DepositS {
									id: 2,
									value: 10,
									start_time: 1000,
									expired_time: 2000,
									in_use: true,
								},
							],
						);
						<darwinia_account_migration::Ledgers<Runtime>>::insert(
							&from_pk,
							Ledger {
								staked_ring: 20,
								staked_kton: 20,
								staked_deposits: Default::default(),
								unstaking_ring: Default::default(),
								unstaking_kton: Default::default(),
								unstaking_deposits: Default::default(),
							},
						);

						assert_ok!(migrate(from, to));
						assert_eq!(Balances::free_balance(to), 60);
						assert_eq!(
							Balances::free_balance(&darwinia_deposit::account_id::<AccountId>()),
							20
						);
						assert_eq!(
							Balances::free_balance(&darwinia_staking::account_id::<AccountId>()),
							20
						);
						assert_eq!(Deposit::deposit_of(to).unwrap().len(), 2);
						assert_eq!(Assets::maybe_balance(KTON_ID, to).unwrap(), 80);
						assert_eq!(
							Assets::maybe_balance(
								KTON_ID,
								darwinia_staking::account_id::<AccountId>()
							)
							.unwrap(),
							20
						);
						assert_eq!(DarwiniaStaking::ledger_of(to).unwrap().staked_ring, 20);
						assert_eq!(DarwiniaStaking::ledger_of(to).unwrap().staked_kton, 20);
					});
			}

			#[test]
			fn identities_should_work() {
				let (from, from_pk) = alice();
				let to = H160::from_low_u64_be(255).into();

				ExtBuilder::default().build().execute_with(|| {
					preset_state_of(&from);

					let info = IdentityInfo {
						additional: Default::default(),
						display: Data::Sha256([1u8; 32]),
						legal: Data::None,
						web: Data::None,
						riot: Data::None,
						email: Data::None,
						pgp_fingerprint: None,
						image: Data::None,
						twitter: Data::None,
					};
					<darwinia_account_migration::Identities<Runtime>>::insert(
						from_pk,
						Registration {
							judgements: Default::default(),
							deposit: RING_AMOUNT,
							info: info.clone(),
						},
					);

					assert_ok!(migrate(from, to));
					assert_eq!(Identity::identity(to).unwrap().info, info);
					assert_eq!(Identity::identity(to).unwrap().deposit, 0);
					assert_eq!(Identity::identity(to).unwrap().judgements.len(), 0);
				});
			}

			#[test]
			fn registrars_should_work() {
				let (from, from_pk) = alice();
				let mut truncated_from = [0_u8; 20];

				truncated_from
					.copy_from_slice(&<AccountId32 as AsRef<[u8; 32]>>::as_ref(&from_pk)[..20]);

				let to = H160::from_low_u64_be(255).into();

				ExtBuilder::default().build().execute_with(|| {
					preset_state_of(&from);

					let info = RegistrarInfo::<Balance, AccountId> {
						account: truncated_from.into(),
						fee: RING_AMOUNT,
						fields: IdentityFields::default(),
					};

					migration::put_storage_value(
						b"Identity",
						b"Registrars",
						&[],
						vec![Some(info.clone()), None],
					);

					assert_ok!(migrate(from, to));
					assert_eq!(Identity::registrars()[0].as_ref().unwrap().account, to);
					assert_eq!(Identity::registrars()[0].as_ref().unwrap().fee, info.fee);
					assert!(Identity::registrars()[1].is_none());
				});
			}
		}
	};
}

#[macro_export]
macro_rules! impl_evm_tests {
	() => {
		mod evm {
			// darwinia
			use super::mock::*;

			#[test]
			fn configured_base_extrinsic_weight_is_evm_compatible() {
				let min_ethereum_transaction_weight = WeightPerGas::get() * 21_000;
				let base_extrinsic = <Runtime as frame_system::Config>::BlockWeights::get()
					.get(frame_support::dispatch::DispatchClass::Normal)
					.base_extrinsic;

				assert!(base_extrinsic.ref_time() <= min_ethereum_transaction_weight.ref_time());
			}
		}
	};
}

#[macro_export]
macro_rules! impl_fee_tests {
	() => {
		mod transaction_fee {
			// darwinia
			use super::mock::*;
			// frontier
			use fp_evm::FeeCalculator;
			// substrate
			use frame_support::{
				dispatch::DispatchClass, pallet_prelude::Weight, traits::OnFinalize,
			};
			use pallet_transaction_payment::Multiplier;
			use polkadot_runtime_common::{
				MinimumMultiplier, SlowAdjustingFeeUpdate, TargetBlockFullness,
			};
			use sp_core::U256;
			use sp_runtime::{traits::Convert, Perbill};

			fn run_with_system_weight<F>(w: Weight, mut assertions: F)
			where
				F: FnMut() -> (),
			{
				let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
					.build_storage::<Runtime>()
					.unwrap()
					.into();
				t.execute_with(|| {
					System::set_block_consumed_resources(w, 0);
					assertions()
				});
			}

			#[test]
			fn multiplier_can_grow_from_zero() {
				let minimum_multiplier = MinimumMultiplier::get();
				let target = TargetBlockFullness::get()
					* RuntimeBlockWeights::get().get(DispatchClass::Normal).max_total.unwrap();
				// if the min is too small, then this will not change, and we are doomed forever.
				// the weight is 1/100th bigger than target.
				run_with_system_weight(target.saturating_mul(101) / 100, || {
					let next = SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
					assert!(next > minimum_multiplier, "{:?} !>= {:?}", next, minimum_multiplier);
				})
			}

			#[test]
			fn initial_evm_gas_fee_is_correct() {
				ExtBuilder::default().build().execute_with(|| {
					assert_eq!(TransactionPayment::next_fee_multiplier(), Multiplier::from(1u128));
					assert_eq!(
						TransactionPaymentGasPrice::min_gas_price().0,
						U256::from(18_780_048_076_923u128)
					);
				})
			}

			#[test]
			fn test_evm_fee_adjustment() {
				ExtBuilder::default().build().execute_with(|| {
					let sim = |fullness: Perbill, num_blocks: u64| -> U256 {
						let block_weight = NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT * fullness;
						for i in 0..num_blocks {
							System::set_block_number(i as u32);
							System::set_block_consumed_resources(block_weight, 0);
							TransactionPayment::on_finalize(i as u32);
						}

						TransactionPaymentGasPrice::min_gas_price().0
					};

					assert_eq!(
						sim(Perbill::from_percent(0), 1),
						U256::from(18_779_695_954_322u128),
					);
					assert_eq!(
						sim(Perbill::from_percent(25), 1),
						U256::from(18_779_695_954_322u128),
					);
					assert_eq!(
						sim(Perbill::from_percent(50), 1),
						U256::from(18_780_048_076_923u128),
					);
					assert_eq!(
						sim(Perbill::from_percent(100), 1),
						U256::from(18_781_104_484_337u128),
					);

					// 1 "real" hour (at 12-second blocks)
					assert_eq!(
						sim(Perbill::from_percent(0), 300),
						U256::from(18_675_757_338_238u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(25), 300),
						U256::from(18_675_757_338_238u128),
					);
					assert_eq!(
						sim(Perbill::from_percent(50), 300),
						U256::from(18_781_104_484_337u128),
					);
					assert_eq!(
						sim(Perbill::from_percent(100), 300),
						U256::from(19_100_724_834_341u128),
					);

					// 1 "real" day (at 12-second blocks)
					assert_eq!(
						sim(Perbill::from_percent(0), 7200),
						U256::from(16_688_607_212_670u128),
					);
					assert_eq!(
						sim(Perbill::from_percent(25), 7200),
						U256::from(16_688_607_212_670u128),
					);
					assert_eq!(
						sim(Perbill::from_percent(50), 7200),
						U256::from(19_100_724_834_341u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(100), 7200),
						U256::from(28_637_764_490_907u128),
					);

					// 7 "real" day (at 12-second blocks)
					assert_eq!(
						sim(Perbill::from_percent(0), 50400),
						U256::from(11_130_914_014_528u128),
					);
					assert_eq!(
						sim(Perbill::from_percent(25), 50400),
						U256::from(11_130_914_014_528u128),
					);
					assert_eq!(
						sim(Perbill::from_percent(50), 50400),
						U256::from(28_637_764_490_907u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(100), 50400),
						U256::from(487_712_592_259_520u128),
					);
				})
			}
		}
	};
}

#[macro_export]
macro_rules! impl_messages_bridge_tests {
	() => {
		mod messages_bridge {
			// crates.io
			use static_assertions::assert_type_eq_all;
			// darwinia
			use super::mock::*;

			#[test]
			fn darwinia_constants_should_match() {
				assert_eq!(
					bp_darwinia_core::MILLISECS_PER_BLOCK,
					dc_primitives::MILLISECS_PER_BLOCK
				);
				assert_eq!(bp_darwinia_core::MINUTES, dc_primitives::MINUTES);
				assert_eq!(bp_darwinia_core::HOURS, dc_primitives::HOURS);
				assert_eq!(bp_darwinia_core::DAYS, dc_primitives::DAYS);

				assert_eq!(
					bp_darwinia_core::AVERAGE_ON_INITIALIZE_RATIO,
					AVERAGE_ON_INITIALIZE_RATIO
				);
				assert_eq!(bp_darwinia_core::NORMAL_DISPATCH_RATIO, NORMAL_DISPATCH_RATIO);
				assert_eq!(
					bp_darwinia_core::WEIGHT_MILLISECS_PER_BLOCK,
					WEIGHT_MILLISECS_PER_BLOCK
				);
				assert_eq!(bp_darwinia_core::MAXIMUM_BLOCK_WEIGHT, MAXIMUM_BLOCK_WEIGHT);

				assert_eq!(
					bp_darwinia_core::RuntimeBlockLength::get().max,
					RuntimeBlockLength::get().max
				);
			}

			#[test]
			fn darwinia_types_should_match() {
				assert_type_eq_all!(bp_darwinia_core::BlockNumber, dc_primitives::BlockNumber);
				assert_type_eq_all!(bp_darwinia_core::Hash, dc_primitives::Hash);
				assert_type_eq_all!(bp_darwinia_core::Nonce, dc_primitives::Nonce);
				assert_type_eq_all!(bp_darwinia_core::Balance, dc_primitives::Balance);
				assert_type_eq_all!(bp_darwinia_core::AccountId, dc_primitives::AccountId);
			}

			#[test]
			fn polkadot_constants_should_match() {
				assert_eq!(
					bp_polkadot_core::NORMAL_DISPATCH_RATIO,
					polkadot_runtime_common::NORMAL_DISPATCH_RATIO
				);
				assert_eq!(
					bp_polkadot_core::MAXIMUM_BLOCK_WEIGHT,
					polkadot_runtime_common::MAXIMUM_BLOCK_WEIGHT
				);
				assert_eq!(
					bp_polkadot_core::AVERAGE_ON_INITIALIZE_RATIO,
					polkadot_runtime_common::AVERAGE_ON_INITIALIZE_RATIO
				);
				assert_eq!(
					bp_polkadot_core::BlockLength::get().max,
					polkadot_runtime_common::BlockLength::get().max
				);
			}

			#[test]
			fn polkadot_types_should_match() {
				assert_type_eq_all!(
					bp_polkadot_core::BlockNumber,
					polkadot_primitives::BlockNumber
				);
				assert_type_eq_all!(bp_polkadot_core::Balance, polkadot_primitives::Balance);
				assert_type_eq_all!(bp_polkadot_core::Hash, polkadot_primitives::Hash);
				assert_type_eq_all!(bp_polkadot_core::Index, polkadot_primitives::AccountIndex);
				assert_type_eq_all!(bp_polkadot_core::Nonce, polkadot_primitives::Nonce);
				assert_type_eq_all!(bp_polkadot_core::Signature, polkadot_primitives::Signature);
				assert_type_eq_all!(bp_polkadot_core::AccountId, polkadot_primitives::AccountId);
				assert_type_eq_all!(bp_polkadot_core::Header, polkadot_primitives::Header);
			}

			#[test]
			fn block_execution_and_extrinsic_base_weight_should_match() {
				assert_eq!(
					weights::BlockExecutionWeight::get(),
					frame_support::weights::constants::BlockExecutionWeight::get(),
				);
				assert_eq!(
					weights::ExtrinsicBaseWeight::get(),
					frame_support::weights::constants::ExtrinsicBaseWeight::get(),
				);
			}
		}
	};
}
