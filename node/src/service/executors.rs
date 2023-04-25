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

/// Darwinia native executor instance.
#[cfg(any(feature = "darwinia-native", feature = "darwinia-native-evm-tracing"))]
pub struct DarwiniaRuntimeExecutor;
#[cfg(any(feature = "darwinia-native", feature = "darwinia-native-evm-tracing"))]
impl NativeExecutionDispatch for DarwiniaRuntimeExecutor {
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	#[cfg(feature = "darwinia-native-evm-tracing")]
	type ExtendHostFunctions = moonbeam_primitives_ext::moonbeam_ext::HostFunctions;
	#[cfg(not(any(feature = "darwinia-native-evm-tracing", feature = "runtime-benchmarks")))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		darwinia_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		darwinia_runtime::native_version()
	}
}

/// Crab native executor instance.
#[cfg(any(feature = "crab-native", feature = "crab-native-evm-tracing"))]
pub struct CrabRuntimeExecutor;
#[cfg(any(feature = "crab-native", feature = "crab-native-evm-tracing"))]
impl NativeExecutionDispatch for CrabRuntimeExecutor {
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	#[cfg(feature = "crab-native-evm-tracing")]
	type ExtendHostFunctions = moonbeam_primitives_ext::moonbeam_ext::HostFunctions;
	#[cfg(not(any(feature = "crab-native-evm-tracing", feature = "runtime-benchmarks")))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		crab_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		crab_runtime::native_version()
	}
}

/// Pangolin native executor instance.
#[cfg(any(feature = "pangolin-native", feature = "pangolin-native-evm-tracing"))]
pub struct PangolinRuntimeExecutor;
#[cfg(any(feature = "pangolin-native", feature = "pangolin-native-evm-tracing"))]
impl NativeExecutionDispatch for PangolinRuntimeExecutor {
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	#[cfg(feature = "pangolin-native-evm-tracing")]
	type ExtendHostFunctions = moonbeam_primitives_ext::moonbeam_ext::HostFunctions;
	#[cfg(not(any(feature = "pangolin-native-evm-tracing", feature = "runtime-benchmarks")))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		pangolin_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		pangolin_runtime::native_version()
	}
}

/// Pangoro native executor instance.
#[cfg(any(feature = "pangoro-native", feature = "pangoro-native-evm-tracing"))]
pub struct PangoroRuntimeExecutor;
#[cfg(any(feature = "pangoro-native", feature = "pangoro-native-evm-tracing"))]
impl NativeExecutionDispatch for PangoroRuntimeExecutor {
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	#[cfg(feature = "pangoro-native-evm-tracing")]
	type ExtendHostFunctions = moonbeam_primitives_ext::moonbeam_ext::HostFunctions;
	#[cfg(not(any(feature = "pangoro-native-evm-tracing", feature = "runtime-benchmarks")))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		pangoro_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		pangoro_runtime::native_version()
	}
}
