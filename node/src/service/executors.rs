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
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

// substrate
use sc_executor::{NativeExecutionDispatch, NativeVersion};

#[cfg(all(feature = "runtime-benchmarks", feature = "evm-tracing"))]
pub type HostFunctions = (
	frame_benchmarking::benchmarking::HostFunctions,
	moonbeam_primitives_ext::moonbeam_ext::HostFunctions,
);
#[cfg(all(feature = "runtime-benchmarks", not(feature = "evm-tracing")))]
pub type HostFunctions = frame_benchmarking::benchmarking::HostFunctions;
#[cfg(all(not(feature = "runtime-benchmarks"), feature = "evm-tracing"))]
pub type HostFunctions = moonbeam_primitives_ext::moonbeam_ext::HostFunctions;
#[cfg(not(any(feature = "evm-tracing", feature = "runtime-benchmarks")))]
pub type HostFunctions = ();

/// Darwinia native executor instance.
#[cfg(feature = "darwinia-native")]
pub struct DarwiniaRuntimeExecutor;
#[cfg(feature = "darwinia-native")]
impl NativeExecutionDispatch for DarwiniaRuntimeExecutor {
	type ExtendHostFunctions = HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		darwinia_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		darwinia_runtime::native_version()
	}
}

/// Crab native executor instance.
#[cfg(feature = "crab-native")]
pub struct CrabRuntimeExecutor;
#[cfg(feature = "crab-native")]
impl NativeExecutionDispatch for CrabRuntimeExecutor {
	type ExtendHostFunctions = HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		crab_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		crab_runtime::native_version()
	}
}

/// Pangolin native executor instance.
#[cfg(feature = "pangolin-native")]
pub struct PangolinRuntimeExecutor;
#[cfg(feature = "pangolin-native")]
impl NativeExecutionDispatch for PangolinRuntimeExecutor {
	type ExtendHostFunctions = HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		pangolin_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		pangolin_runtime::native_version()
	}
}
