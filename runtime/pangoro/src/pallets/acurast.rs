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

// darwinia
use crate::*;

// substrate
use frame_support::pallet_prelude::*;
use frame_support::sp_runtime::traits::StaticLookup;
use sp_core::*;
use frame_support::sp_io::hashing;
use sp_std::prelude::*;
use pallet_acurast_fulfillment_receiver::Fulfillment;

#[derive(RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq)]
pub enum MethodSignatureHash {
    Default,
    Custom(BoundedVec<u8, ConstU32<4>>),
}

impl MethodSignatureHash {
    fn to_bytes(&self) -> [u8; 4] {
        match self {
            Self::Default => hashing::keccak_256(b"fulfill(address,bytes)")[0..4].try_into().unwrap(),
            Self::Custom(bytes) => bytes.to_vec().try_into().unwrap(),
        }
    }
}

#[derive(RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq)]
pub struct AcurastRegistrationExtra {
    pub destination_contract: H160,
    pub method_signature_hash: MethodSignatureHash,
}

pub struct AcurastRouter;
impl FulfillmentRouter<Runtime> for AcurastRouter {
    fn received_fulfillment(
        origin: frame_system::pallet_prelude::OriginFor<Runtime>,
        from: <Runtime as frame_system::Config>::AccountId,
        fulfillment: Fulfillment,
        registration: pallet_acurast::JobRegistrationFor<Runtime>,
        requester: <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Target,
    ) -> DispatchResultWithPostInfo {
        let from_bytes: [u8; 32] = from.try_into().unwrap();
        let eth_source = H160::from_slice(&from_bytes[0..20]);
        let requester_bytes: [u8; 32] = requester.try_into().unwrap();
        let eth_requester = H160::from_slice(&requester_bytes[0..20]);
        let gas_limit = 4294967;
        EVM::call(
            origin,
            eth_source,
            registration.extra.destination_contract,
            create_eth_call(
                registration.extra.method_signature_hash,
                eth_requester,
                fulfillment.payload,
            ),
            U256::zero(),
            gas_limit,
            // TODO update this @jiguantong
            U256::from(1_000_000),
            None,
            None,
            vec![],
        )
    }
}

fn create_eth_call(method: MethodSignatureHash, requester: H160, payload: Vec<u8>) -> Vec<u8> {
    let mut requester_bytes: [u8; 32] = [0; 32];
    requester_bytes[(32 - requester.0.len())..].copy_from_slice(&requester.0);
    let mut offset_bytes: [u8; 32] = [0; 32];
    let payload_offset = requester_bytes.len().to_be_bytes();
    offset_bytes[(32 - payload_offset.len())..].copy_from_slice(&payload_offset);
    let mut payload_len_bytes: [u8; 32] = [0; 32];
    let payload_len = payload.len().to_be_bytes();
    payload_len_bytes[(32 - payload_len.len())..].copy_from_slice(&payload_len);
    [
        method.to_bytes().as_slice(),
        requester_bytes.as_slice(),
        offset_bytes.as_slice(),
        payload_len_bytes.as_slice(),
        &payload,
    ]
    .concat()
}

impl pallet_acurast::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RegistrationExtra = AcurastRegistrationExtra;
    // type FulfillmentRouter = AcurastRouter;
}