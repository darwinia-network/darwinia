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

// --- std ---
use std::{path::PathBuf, sync::Arc};
// --- darwinia-network ---
use darwinia_client_rpc::{CacheTask, DebugTask};
use darwinia_common_primitives::{OpaqueBlock as Block, *};
use fc_db::Backend as DvmBackend;
use fc_mapping_sync::{MappingSyncWorker, SyncStrategy};
use fc_rpc::{EthTask, OverrideHandle};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use sc_client_api::BlockchainEvents;
use sp_blockchain::Error as BlockChainError;

pub struct DvmTaskParams<'a, B, C, BE>
where
	B: sp_runtime::traits::Block,
{
	pub task_manager: &'a sc_service::TaskManager,
	pub client: Arc<C>,
	pub substrate_backend: Arc<BE>,
	pub dvm_backend: Arc<DvmBackend<B>>,
	pub filter_pool: Option<fc_rpc_core::types::FilterPool>,
	pub is_archive: bool,
	pub rpc_config: darwinia_rpc::EthRpcConfig,
	pub fee_history_cache: FeeHistoryCache,
	pub overrides: Arc<OverrideHandle<B>>,
}
impl<'a, B, C, BE> DvmTaskParams<'a, B, C, BE>
where
	B: sp_runtime::traits::Block,
{
	pub fn spawn_task(self) -> darwinia_rpc::EthRpcRequesters
	where
		C: 'static
			+ sp_api::ProvideRuntimeApi<B>
			+ sp_blockchain::HeaderBackend<B>
			+ sp_blockchain::HeaderMetadata<B, Error = sp_blockchain::Error>
			+ sc_client_api::BlockOf
			+ sc_client_api::BlockchainEvents<B>,
		C::Api: sp_block_builder::BlockBuilder<B>
			+ fp_rpc::EthereumRuntimeRPCApi<B>
			+ moonbeam_rpc_primitives_debug::DebugRuntimeApi<B>,
		B: 'static + Send + Sync + sp_runtime::traits::Block<Hash = Hash>,
		B::Header: sp_api::HeaderT<Number = BlockNumber>,
		BE: 'static + sc_client_api::backend::Backend<B>,
	{
		// --- std ---
		use std::time::Duration;
		// --- crates.io ---
		use futures::StreamExt;
		use tokio::sync::Semaphore;
		// --- paritytech ---
		use darwinia_client_rpc::EthTask;
		use dc_mapping_sync::{MappingSyncWorker, SyncStrategy};
		// --- darwinia-network ---
		use darwinia_client_rpc::{CacheTask, DebugTask};
		use darwinia_rpc::{EthRpcConfig, EthRpcRequesters};
		use moonbeam_rpc_debug::DebugHandler;
		use moonbeam_rpc_trace::CacheTask;

		let DvmTaskParams {
			task_manager,
			client,
			substrate_backend,
			dvm_backend,
			filter_pool,
			is_archive,
			rpc_config:
				EthRpcConfig {
					ethapi_debug_targets,
					ethapi_max_permits,
					ethapi_trace_cache_duration,
					fee_history_limit,
					overrides,
				},
		} = self;

		if is_archive {
			// Spawn schema cache maintenance task.
			task_manager.spawn_essential_handle().spawn(
				"frontier-schema-cache-task",
				EthTask::ethereum_schema_cache_task(Arc::clone(&client), Arc::clone(&dvm_backend)),
			);
			// Spawn Frontier FeeHistory cache maintenance task.
			task_manager.spawn_essential_handle().spawn(
				"frontier-fee-history",
				EthTask::fee_history_task(
					Arc::clone(&client),
					Arc::clone(&overrides),
					fee_history_cache,
					rpc_config.fee_history_limit,
				),
			);
			// Spawn mapping sync worker task.
			task_manager.spawn_essential_handle().spawn(
				"frontier-mapping-sync-worker",
				MappingSyncWorker::new(
					client.import_notification_stream(),
					Duration::new(6, 0),
					client.clone(),
					substrate_backend.clone(),
					dvm_backend.clone(),
					SyncStrategy::Normal,
				)
				.for_each(|()| futures::future::ready(())),
			);
			// Spawn EthFilterApi maintenance task.
			if let Some(filter_pool) = filter_pool {
				// Each filter is allowed to stay in the pool for 100 blocks.
				const FILTER_RETAIN_THRESHOLD: u64 = 100;
				task_manager.spawn_essential_handle().spawn(
					"frontier-filter-pool",
					EthTask::filter_pool_task(
						Arc::clone(&client),
						filter_pool,
						FILTER_RETAIN_THRESHOLD,
					),
				);
			}
		}

		if ethapi_debug_targets
			.iter()
			.any(|cmd| matches!(cmd.as_str(), "debug" | "trace"))
		{
			let permit_pool = Arc::new(Semaphore::new(ethapi_max_permits as usize));
			let (trace_filter_task, trace_filter_requester) = if ethapi_debug_targets
				.iter()
				.any(|target| target.as_str() == "trace")
			{
				let (trace_filter_task, trace_filter_requester) = CacheTask::create(
					Arc::clone(&client),
					Arc::clone(&substrate_backend),
					Duration::from_secs(ethapi_trace_cache_duration),
					Arc::clone(&permit_pool),
					Arc::clone(&overrides),
				);
				(Some(trace_filter_task), Some(trace_filter_requester))
			} else {
				(None, None)
			};

			let (debug_task, debug_requester) = if ethapi_debug_targets
				.iter()
				.any(|target| target.as_str() == "debug")
			{
				let (debug_task, debug_requester) = DebugHandler::task(
					Arc::clone(&client),
					Arc::clone(&substrate_backend),
					Arc::clone(&dvm_backend),
					Arc::clone(&permit_pool),
					Arc::clone(&overrides),
				);
				(Some(debug_task), Some(debug_requester))
			} else {
				(None, None)
			};

			// `trace_filter` cache task. Essential.
			// Proxies rpc requests to it's handler.
			if let Some(trace_filter_task) = trace_filter_task {
				task_manager
					.spawn_essential_handle()
					.spawn("trace-filter-cache", trace_filter_task);
			}

			// `debug` task if enabled. Essential.
			// Proxies rpc requests to it's handler.
			if let Some(debug_task) = debug_task {
				task_manager
					.spawn_essential_handle()
					.spawn("ethapi_debug_targets-debug", debug_task);
			}

			return EthRpcRequesters {
				debug: debug_requester,
				trace: trace_filter_requester,
			};
		}

		EthRpcRequesters {
			debug: None,
			trace: None,
		}
	}
}

pub fn db_path(config: &sc_service::Configuration) -> PathBuf {
	let config_dir = config
		.base_path
		.as_ref()
		.map(|base_path| base_path.config_dir(config.chain_spec.id()))
		.expect("Config dir must be set.");

	config_dir.join("dvm").join("db")
}

pub fn open_backend(
	config: &sc_service::Configuration,
) -> Result<Arc<dc_db::Backend<Block>>, String> {
	// --- darwinia-network ---
	use dc_db::{Backend, DatabaseSettings, DatabaseSettingsSrc};

	Ok(Arc::new(Backend::<Block>::new(&DatabaseSettings {
		source: DatabaseSettingsSrc::RocksDb {
			path: db_path(&config),
			cache_size: 0,
		},
	})?))
}
