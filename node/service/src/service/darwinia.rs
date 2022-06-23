// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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

//! Darwinia service. Specialized wrapper over substrate service.

// --- std ---
use std::sync::Arc;
// --- darwinia-network ---
use super::*;
use darwinia_primitives::OpaqueBlock as Block;

pub struct Executor;
impl sc_executor::NativeExecutionDispatch for Executor {
	type ExtendHostFunctions =
		(frame_benchmarking::benchmarking::HostFunctions, dp_evm_trace_ext::dvm_ext::HostFunctions);

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		darwinia_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		darwinia_runtime::native_version()
	}
}

/// Create a new Darwinia service for a full node.
#[cfg(feature = "full-node")]
pub fn darwinia_new_full(
	config: sc_service::Configuration,
	authority_discovery_disabled: bool,
	eth_rpc_config: darwinia_rpc::EthRpcConfig,
) -> ServiceResult<(
	sc_service::TaskManager,
	Arc<impl crate::client::DarwiniaClient<Block, FullBackend, darwinia_runtime::RuntimeApi>>,
	sc_service::RpcHandlers,
)> {
	let (components, client, rpc_handlers) = new_full::<darwinia_runtime::RuntimeApi, Executor>(
		config,
		authority_discovery_disabled,
		eth_rpc_config,
	)?;

	Ok((components, client, rpc_handlers))
}
