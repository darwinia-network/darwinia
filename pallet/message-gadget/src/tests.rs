// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
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

// TODO: move to mock later.
use crate as dp_message_gadget;

// std
use std::str::FromStr;
// darwinia
use crate::*;
// frontier
use pallet_evm::{FeeCalculator, Runner};
// substrate
use sp_core::U256;
use sp_runtime::BuildStorage;

impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<u128>;
	type AccountId = sp_core::H160;
	type BaseCallFilter = frame_support::traits::Everything;
	type Block = frame_system::mocking::MockBlock<Self>;
	type BlockHashCount = ();
	type BlockLength = ();
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type Nonce = u64;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

impl pallet_timestamp::Config for Runtime {
	type MinimumPeriod = ();
	type Moment = u128;
	type OnTimestampSet = ();
	type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	type Balance = u128;
	type DustRemoval = ();
	type ExistentialDeposit = frame_support::traits::ConstU128<0>;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type MaxHolds = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = ();
	type WeightInfo = ();
}

impl pallet_evm::Config for Runtime {
	type AddressMapping = pallet_evm::IdentityAddressMapping;
	type BlockGasLimit = ();
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<sp_core::H160>;
	type ChainId = ();
	type Currency = Balances;
	type FeeCalculator = ();
	type FindAuthor = ();
	type GasLimitPovSizeRatio = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type RuntimeEvent = RuntimeEvent;
	type Timestamp = Timestamp;
	type WeightInfo = ();
	type WeightPerGas = ();
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<sp_core::H160>;
}

impl dp_message_gadget::Config for Runtime {}

frame_support::construct_runtime! {
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		EVM: pallet_evm,
		MessageGadget: dp_message_gadget,
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	<frame_system::GenesisConfig<Runtime>>::default().build_storage().unwrap().into()
}

#[test]
fn message_root_getter_should_work() {
	new_test_ext().execute_with(|| assert_eq!(<MessageRootGetter<Runtime>>::get(), None));
	new_test_ext().execute_with(|| {
		// pragma solidity ^0.8.0;
		//
		// contract MessageRootGetter {
		//     function commitment() public returns (bool) {
		//         return true;
		//     }
		// }
		const CONTRACT_CODE: &str = "0x608060405234801561001057600080fd5b5060b88061001f6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c80631303a48414602d575b600080fd5b60336047565b604051603e9190605d565b60405180910390f35b60006001905090565b6057816076565b82525050565b6000602082019050607060008301846050565b92915050565b6000811515905091905056fea26469706673582212205edcbb73cc70f096b015d00b65ed893df280a01c9e90e964e8bb39957d6d3c9d64736f6c63430008070033";

		let res = <Runtime as pallet_evm::Config>::Runner::create(
			H160::from_str("1000000000000000000000000000000000000001").unwrap(),
			array_bytes::hex2bytes_unchecked(CONTRACT_CODE),
			U256::zero(),
			U256::from(300_000_000).low_u64(),
			Some(<Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price().0),
			None,
			Some(U256::from(1)),
			vec![],
			true,
			false,
			None,
			None,
			<Runtime as pallet_evm::Config>::config(),
		);
		let contract_address = res.unwrap().value;

		<CommitmentContract<Runtime>>::put(contract_address);

		assert_eq!(MessageGadget::commitment_contract(), contract_address);
		assert_eq!(
			<MessageRootGetter<Runtime>>::get(),
			Some(H256::from_slice(&[
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 1
			]))
		);
	});
}
