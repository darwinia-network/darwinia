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
				assert_err, assert_ok, migration, traits::Get, Blake2_128Concat, StorageHasher,
			};
			use frame_system::AccountInfo;
			use pallet_assets::ExistenceReason;
			use pallet_balances::AccountData;
			use pallet_identity::{
				Data, IdentityFields, IdentityInfo, RegistrarInfo, Registration,
			};
			use sp_core::{sr25519::Pair, Decode, Encode, Pair as PairT, H160};
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
				let message = darwinia_account_migration::sr25519_signable_message(
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
					signature: sig.clone(),
				})?;
				AccountMigration::migrate(RuntimeOrigin::none(), from_pk, to, sig)?;

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
				let message = darwinia_account_migration::sr25519_signable_message(b"?", &to);
				let sig = from.sign(&message);

				ExtBuilder::default().build().execute_with(|| {
					preset_state_of(&from);

					assert_err!(
						AccountMigration::pre_dispatch(
							&darwinia_account_migration::Call::migrate {
								from: from_pk.clone(),
								to,
								signature: sig.clone(),
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

					assert_ok!(migrate(from, to,));
					assert_eq!(Identity::identity(to).unwrap().info, info);
					assert_eq!(Identity::identity(to).unwrap().deposit, RING_AMOUNT);
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
			use crate::mock::{Runtime, WeightPerGas};

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
