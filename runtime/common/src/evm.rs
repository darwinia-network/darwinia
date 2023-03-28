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

/// Frontier SelfContainedCall implementation
#[macro_export]
macro_rules! impl_self_contained_call {
	() => {
		impl fp_self_contained::SelfContainedCall for RuntimeCall {
			type SignedInfo = sp_core::H160;

			fn is_self_contained(&self) -> bool {
				match self {
					RuntimeCall::Ethereum(call) => call.is_self_contained(),
					_ => false,
				}
			}

			fn check_self_contained(
				&self,
			) -> Option<
				Result<
					Self::SignedInfo,
					sp_runtime::transaction_validity::TransactionValidityError,
				>,
			> {
				match self {
					RuntimeCall::Ethereum(call) => call.check_self_contained(),
					_ => None,
				}
			}

			fn validate_self_contained(
				&self,
				info: &Self::SignedInfo,
				dispatch_info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
				len: usize,
			) -> Option<sp_runtime::transaction_validity::TransactionValidity> {
				match self {
					RuntimeCall::Ethereum(call) =>
						call.validate_self_contained(info, dispatch_info, len),
					_ => None,
				}
			}

			fn pre_dispatch_self_contained(
				&self,
				info: &Self::SignedInfo,
				dispatch_info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
				len: usize,
			) -> Option<Result<(), sp_runtime::transaction_validity::TransactionValidityError>> {
				match self {
					RuntimeCall::Ethereum(call) =>
						call.pre_dispatch_self_contained(info, dispatch_info, len),
					_ => None,
				}
			}

			fn apply_self_contained(
				self,
				info: Self::SignedInfo,
			) -> Option<
				sp_runtime::DispatchResultWithInfo<sp_runtime::traits::PostDispatchInfoOf<Self>>,
			> {
				// substrate
				use sp_runtime::traits::Dispatchable;

				match self {
					call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) =>
						Some(call.dispatch(RuntimeOrigin::from(
							pallet_ethereum::RawOrigin::EthereumTransaction(info),
						))),
					_ => None,
				}
			}
		}
	};
}

/// The author finder for the EVM module
pub struct DarwiniaFindAuthor<Inner>(sp_std::marker::PhantomData<Inner>);
impl<Inner> frame_support::traits::FindAuthor<sp_core::H160> for DarwiniaFindAuthor<Inner>
where
	Inner: frame_support::traits::FindAuthor<dc_primitives::AccountId>,
{
	fn find_author<'a, I>(digests: I) -> Option<sp_core::H160>
	where
		I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
	{
		Inner::find_author(digests).map(Into::into)
	}
}

/// Fixed EVM gas price
pub struct FixedGasPrice;
impl pallet_evm::FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> (sp_core::U256, frame_support::weights::Weight) {
		(sp_core::U256::from(dc_types::GWEI), frame_support::weights::Weight::zero())
	}
}

/// Derive an asset id from an account id
pub struct AssetIdConverter;
impl darwinia_precompile_assets::AccountToAssetId<dc_primitives::AccountId, dc_types::AssetId>
	for AssetIdConverter
{
	fn account_to_asset_id(account_id: dc_primitives::AccountId) -> dc_types::AssetId {
		let addr: sp_core::H160 = account_id.into();
		addr.to_low_u64_be()
	}
}
