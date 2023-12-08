// This file is part of Darwinia.
//
// Copyright (C) 2018-2023 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

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
				reason: ExistenceReason<u128, AccountId>,
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
					reason: ExistenceReason::<u128, AccountId>::Sufficient,
					extra: (),
				};

				assert!(AccountMigration::account_of(&account_id_32).is_none());
				assert!(AccountMigration::kton_account_of(&account_id_32).is_none());

				<pallet_balances::TotalIssuance<Runtime, _>>::put(RING_AMOUNT);
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
			// frontier
			use pallet_evm_precompile_dispatch::DispatchValidateT;
			// substrate
			use frame_support::assert_err;
			use sp_core::{H160, U256};
			use sp_runtime::{DispatchError, ModuleError};

			#[test]
			fn configured_base_extrinsic_weight_is_evm_compatible() {
				let min_ethereum_transaction_weight = WeightPerGas::get() * 21_000;
				let base_extrinsic = <Runtime as frame_system::Config>::BlockWeights::get()
					.get(frame_support::dispatch::DispatchClass::Normal)
					.base_extrinsic;

				assert!(base_extrinsic.ref_time() <= min_ethereum_transaction_weight.ref_time());
			}

			#[test]
			fn evm_constants_are_correctly() {
				assert_eq!(BlockGasLimit::get(), U256::from(20_000_000));
				assert_eq!(WeightPerGas::get().ref_time(), 18750);
			}

			#[test]
			fn pallet_evm_calls_only_callable_by_root() {
				ExtBuilder::default().build().execute_with(|| {
					// https://github.com/darwinia-network/darwinia/blob/5923b2e0526b67fe05cee6e4e592ceca80e8f2ff/runtime/darwinia/src/pallets/evm.rs#L136
					assert_err!(
						EVM::call(
							RuntimeOrigin::signed(H160::default().into()),
							H160::default(),
							H160::default(),
							vec![],
							U256::default(),
							1000000,
							U256::from(1_000_000),
							None,
							None,
							vec![],
						),
						DispatchError::BadOrigin
					);

					if let Err(dispatch_info_with_err) = EVM::call(
						RuntimeOrigin::root(),
						H160::default(),
						H160::default(),
						vec![],
						U256::default(),
						1000000,
						U256::from(1_000_000),
						None,
						None,
						vec![],
					) {
						assert_eq!(
							dispatch_info_with_err.error,
							DispatchError::Module(ModuleError {
								index: 37,
								error: [4, 0, 0, 0],
								message: Some("GasPriceTooLow")
							})
						);
					}
				});
			}

			#[test]
			fn dispatch_validator_filter_runtime_calls() {
				ExtBuilder::default().build().execute_with(|| {
					assert!(DarwiniaDispatchValidator::validate_before_dispatch(
						&H160::default().into(),
						&RuntimeCall::System(frame_system::Call::remark { remark: vec![] })
					)
					.is_none());

					assert!(DarwiniaDispatchValidator::validate_before_dispatch(
						&H160::default().into(),
						// forbidden call
						&RuntimeCall::EVM(pallet_evm::Call::call {
							source: H160::default(),
							target: H160::default(),
							input: vec![],
							value: U256::default(),
							gas_limit: 1000000,
							max_fee_per_gas: U256::from(1_000_000),
							max_priority_fee_per_gas: None,
							nonce: None,
							access_list: vec![],
						})
					)
					.is_some());
				});
			}

			#[test]
			fn dispatch_validator_filter_dispatch_class() {
				ExtBuilder::default().build().execute_with(|| {
					// Default class
					assert!(DarwiniaDispatchValidator::validate_before_dispatch(
						&H160::default().into(),
						&RuntimeCall::System(frame_system::Call::remark { remark: vec![] })
					)
					.is_none());

					// Operational class
					assert!(DarwiniaDispatchValidator::validate_before_dispatch(
						&H160::default().into(),
						&RuntimeCall::System(frame_system::Call::set_heap_pages { pages: 20 })
					)
					.is_none());

					// Mandatory class
					assert!(DarwiniaDispatchValidator::validate_before_dispatch(
						&H160::default().into(),
						&RuntimeCall::Timestamp(pallet_timestamp::Call::set { now: 100 })
					)
					.is_some());
				});
			}
		}
	};
}

#[macro_export]
macro_rules! impl_weight_tests {
	() => {
		mod weight {
			// darwinia
			use super::mock::*;
			// substrate
			use frame_support::{
				dispatch::DispatchClass,
				weights::{Weight, WeightToFee as WeightToFeeT},
			};
			use sp_runtime::traits::Zero;

			// We can fit at least 1000 transfers in a block.
			#[test]
			fn sane_block_weight() {
				// substrate
				use pallet_balances::WeightInfo;

				let block = RuntimeBlockWeights::get().max_block;
				let base = RuntimeBlockWeights::get().get(DispatchClass::Normal).base_extrinsic;
				let transfer =
					base + weights::pallet_balances::WeightInfo::<Runtime>::transfer_allow_death();
				let fit = block.checked_div_per_component(&transfer).unwrap_or_default();

				assert!(fit >= 1000, "{} should be at least 1000", fit);
			}

			// The fee for one transfer is at most 1 UNIT.
			#[test]
			fn sane_transfer_fee() {
				// substrate
				use pallet_balances::WeightInfo;

				let base = RuntimeBlockWeights::get().get(DispatchClass::Normal).base_extrinsic;
				let transfer =
					base + weights::pallet_balances::WeightInfo::<Runtime>::transfer_allow_death();
				let fee = WeightToFee::weight_to_fee(&transfer);

				assert!(fee <= UNIT, "{} MILLIUNIT should be at most 1000", fee / MILLIUNIT);
			}

			// Weight is being charged for both dimensions.
			#[test]
			fn weight_charged_for_both_components() {
				let fee = WeightToFee::weight_to_fee(&Weight::from_parts(10_000, 0));
				assert!(!fee.is_zero(), "Charges for ref time");

				let fee = WeightToFee::weight_to_fee(&Weight::from_parts(0, 10_000));
				assert_eq!(fee, UNIT, "10kb maps to UNIT");
			}

			// Filling up a block by proof size is at most 30 times more expensive than ref time.
			//
			// This is just a sanity check.
			#[test]
			fn full_block_fee_ratio() {
				let block = RuntimeBlockWeights::get().max_block;
				let time_fee = WeightToFee::weight_to_fee(&Weight::from_parts(block.ref_time(), 0));
				let proof_fee =
					WeightToFee::weight_to_fee(&Weight::from_parts(0, block.proof_size()));

				let proof_o_time = proof_fee.checked_div(time_fee).unwrap_or_default();
				assert!(proof_o_time <= 30, "{} should be at most 30", proof_o_time);
				let time_o_proof = time_fee.checked_div(proof_fee).unwrap_or_default();
				assert!(time_o_proof <= 30, "{} should be at most 30", time_o_proof);
			}

			#[test]
			fn eth_transaction_max_allowed_gas_limit() {
				// frontier
				use pallet_evm::GasWeightMapping;

				let max_extrinsic_weight = <Runtime as frame_system::Config>::BlockWeights::get()
					.get(DispatchClass::Normal)
					.max_extrinsic
					.expect("Failed to get max extrinsic weight");

				assert!(!<Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					12_000_000, true
				)
				.any_gt(max_extrinsic_weight));
				assert!(!<Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					14_000_000, true
				)
				.any_gt(max_extrinsic_weight));
				assert!(!<Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					16_000_000, true
				)
				.any_gt(max_extrinsic_weight));
				assert!(!<Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					18_000_000, true
				)
				.any_gt(max_extrinsic_weight));
				assert!(<Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					20_000_000, true
				)
				.any_gt(max_extrinsic_weight));
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
			use sp_runtime::{traits::Convert, BuildStorage, Perbill};

			fn run_with_system_weight<F>(w: Weight, mut assertions: F)
			where
				F: FnMut() -> (),
			{
				let mut t: sp_io::TestExternalities =
					<frame_system::GenesisConfig<Runtime>>::default()
						.build_storage()
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
						U256::from(1_507_065_121_289u128)
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

					assert_eq!(sim(Perbill::from_percent(0), 1), U256::from(1507036864082u128),);
					assert_eq!(sim(Perbill::from_percent(25), 1), U256::from(1507036864082u128),);
					assert_eq!(sim(Perbill::from_percent(50), 1), U256::from(1507065121289u128),);
					assert_eq!(sim(Perbill::from_percent(100), 1), U256::from(1507149896086u128),);

					// 1 "real" hour (at 12-second blocks)
					assert_eq!(sim(Perbill::from_percent(0), 300), U256::from(1498695976859u128));
					assert_eq!(sim(Perbill::from_percent(25), 300), U256::from(1498695976859u128),);
					assert_eq!(sim(Perbill::from_percent(50), 300), U256::from(1507149896086u128),);
					assert_eq!(sim(Perbill::from_percent(100), 300), U256::from(1532798855001u128),);

					// 1 "real" day (at 12-second blocks)
					assert_eq!(sim(Perbill::from_percent(0), 7200), U256::from(1339230749042u128),);
					assert_eq!(sim(Perbill::from_percent(25), 7200), U256::from(1339230749042u128),);
					assert_eq!(sim(Perbill::from_percent(50), 7200), U256::from(1532798855001u128));
					assert_eq!(
						sim(Perbill::from_percent(100), 7200),
						U256::from(2298129154896u128),
					);

					// 7 "real" day (at 12-second blocks)
					assert_eq!(sim(Perbill::from_percent(0), 50400), U256::from(893235853851u128),);
					assert_eq!(sim(Perbill::from_percent(25), 50400), U256::from(893235853851u128),);
					assert_eq!(
						sim(Perbill::from_percent(50), 50400),
						U256::from(2298129154896u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(100), 50400),
						U256::from(39138059391389u128),
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
					frame_support::weights::constants::BlockExecutionWeight::get(),
					frame_support::weights::constants::BlockExecutionWeight::get(),
				);
				assert_eq!(
					frame_support::weights::constants::ExtrinsicBaseWeight::get(),
					frame_support::weights::constants::ExtrinsicBaseWeight::get(),
				);
			}
		}
	};
}

#[macro_export]
macro_rules! impl_maintenance_tests {
	() => {
		mod maintenance_test {
			// darwinia
			use super::mock::*;
			// substrate
			use frame_support::{assert_err, assert_ok};
			use pallet_tx_pause::RuntimeCallNameOf;
			use sp_core::H160;
			use sp_runtime::{traits::Dispatchable, DispatchError};

			pub fn full_name(pallet_name: &[u8], call_name: &[u8]) -> RuntimeCallNameOf<Runtime> {
				<RuntimeCallNameOf<Runtime>>::from((
					pallet_name.to_vec().try_into().unwrap(),
					call_name.to_vec().try_into().unwrap(),
				))
			}

			#[test]
			fn tx_pause_origins_work_correctly() {
				ExtBuilder::default().build().execute_with(|| {
					assert_ok!(TxPause::pause(
						RuntimeOrigin::root(),
						full_name(b"Balances", b"transfer")
					));

					assert_err!(
						TxPause::pause(
							RuntimeOrigin::signed(H160::default().into()),
							full_name(b"Balances", b"transfer")
						),
						DispatchError::BadOrigin
					);
				})
			}

			#[test]
			fn tx_pause_pause_and_unpause_work_correctly() {
				let from = H160::from_low_u64_be(555).into();
				let to = H160::from_low_u64_be(333).into();
				ExtBuilder::default()
					.with_balances(vec![(from, 100), (to, 50)])
					.build()
					.execute_with(|| {
						assert_ok!(TxPause::pause(
							RuntimeOrigin::root(),
							full_name(b"System", b"remark")
						));
						assert_err!(
							RuntimeCall::System(frame_system::Call::remark {
								remark: b"hello world".to_vec()
							})
							.dispatch(RuntimeOrigin::signed(from)),
							frame_system::Error::<Runtime>::CallFiltered
						);

						assert_ok!(TxPause::unpause(
							RuntimeOrigin::root(),
							full_name(b"System", b"remark")
						));
						assert_ok!(RuntimeCall::System(frame_system::Call::remark {
							remark: b"hello world".to_vec(),
						})
						.dispatch(RuntimeOrigin::signed(from)));
					})
			}

			#[test]
			fn tx_pause_pause_calls_except_on_whitelist() {
				let from = H160::from_low_u64_be(555).into();
				ExtBuilder::default().with_balances(vec![(from, 100)]).build().execute_with(|| {
					assert_ok!(RuntimeCall::System(frame_system::Call::remark_with_event {
						remark: b"hello world".to_vec(),
					})
					.dispatch(RuntimeOrigin::signed(from)));

					assert_err!(
						TxPause::pause(
							RuntimeOrigin::root(),
							full_name(b"System", b"remark_with_event")
						),
						pallet_tx_pause::Error::<Runtime>::Unpausable
					);

					assert_ok!(RuntimeCall::System(frame_system::Call::remark_with_event {
						remark: b"hello world".to_vec(),
					})
					.dispatch(RuntimeOrigin::signed(from)));
				})
			}
		}
	};
}
