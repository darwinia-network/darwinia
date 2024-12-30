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

//! Service and service factory implementation. Specialized wrapper over substrate service.

pub mod frontier;

mod instant_finalize;

#[cfg(feature = "crab-runtime")]
pub use crab_runtime::RuntimeApi as CrabRuntimeApi;
#[cfg(feature = "darwinia-runtime")]
pub use darwinia_runtime::RuntimeApi as DarwiniaRuntimeApi;
#[cfg(feature = "koi-runtime")]
pub use koi_runtime::RuntimeApi as KoiRuntimeApi;

// std
use std::{
	collections::BTreeMap,
	sync::{Arc, Mutex},
	time::Duration,
};
// crates.io
use futures::FutureExt;
// darwinia
use dc_primitives::*;
// polkadot-sdk
use sc_client_api::{Backend, HeaderBackend};
use sc_consensus::ImportQueue;
use sc_network::NetworkBlock;
use sp_consensus_aura::{Slot, SlotDuration};
use sp_core::Encode;

#[cfg(all(feature = "runtime-benchmarks", feature = "evm-tracing"))]
type HostFunctions = (
	cumulus_client_service::ParachainHostFunctions,
	frame_benchmarking::benchmarking::HostFunctions,
	moonbeam_primitives_ext::moonbeam_ext::HostFunctions,
);
#[cfg(all(feature = "runtime-benchmarks", not(feature = "evm-tracing")))]
type HostFunctions = (
	cumulus_client_service::ParachainHostFunctions,
	frame_benchmarking::benchmarking::HostFunctions,
);
#[cfg(all(not(feature = "runtime-benchmarks"), feature = "evm-tracing"))]
type HostFunctions = (
	cumulus_client_service::ParachainHostFunctions,
	moonbeam_primitives_ext::moonbeam_ext::HostFunctions,
);
#[cfg(not(any(feature = "evm-tracing", feature = "runtime-benchmarks")))]
type HostFunctions = cumulus_client_service::ParachainHostFunctions;

/// Full client backend type.
type FullBackend = sc_service::TFullBackend<Block>;
/// Full client type.
type FullClient<RuntimeApi> =
	sc_service::TFullClient<Block, RuntimeApi, sc_executor::WasmExecutor<HostFunctions>>;
/// Parachain specific block import.
type ParachainBlockImport<RuntimeApi> = cumulus_client_consensus_common::ParachainBlockImport<
	Block,
	Arc<FullClient<RuntimeApi>>,
	FullBackend,
>;
type Service<RuntimeApi> = sc_service::PartialComponents<
	FullClient<RuntimeApi>,
	FullBackend,
	sc_consensus::LongestChain<FullBackend, Block>,
	sc_consensus::DefaultImportQueue<Block>,
	sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi>>,
	(
		Arc<dyn fc_storage::StorageOverride<Block>>,
		fc_db::Backend<Block, FullClient<RuntimeApi>>,
		Option<fc_rpc_core::types::FilterPool>,
		fc_rpc_core::types::FeeHistoryCache,
		fc_rpc_core::types::FeeHistoryCacheLimit,
		ParachainBlockImport<RuntimeApi>,
		Option<sc_telemetry::Telemetry>,
		Option<sc_telemetry::TelemetryWorkerHandle>,
	),
>;

/// Can be called for a `Configuration` to check if it is the specific network.
pub trait IdentifyVariant {
	/// Get spec id.
	fn id(&self) -> &str;

	/// Returns if this is a configuration for the `Crab` network.
	fn is_crab(&self) -> bool {
		self.id().starts_with("crab")
	}

	/// Returns if this is a configuration for the `Darwinia` network.
	fn is_darwinia(&self) -> bool {
		self.id().starts_with("darwinia")
	}

	/// Returns if this is a configuration for the `Koi` network.
	fn is_koi(&self) -> bool {
		self.id().starts_with("darwinia-koi")
	}

	/// Returns true if this configuration is for a development network.
	fn is_dev(&self) -> bool {
		// Fulfill Polkadot.JS metadata upgrade requirements.
		self.id().ends_with("-d")
	}
}
impl IdentifyVariant for Box<dyn sc_service::ChainSpec> {
	fn id(&self) -> &str {
		sc_service::ChainSpec::id(&**self)
	}
}

/// A set of APIs that darwinia-like runtimes must implement.
pub trait RuntimeApiCollection:
	cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>
	+ cumulus_primitives_core::CollectCollationInfo<Block>
	+ fp_rpc::ConvertTransactionRuntimeApi<Block>
	+ fp_rpc::EthereumRuntimeRPCApi<Block>
	+ moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block>
	+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
	+ sp_api::ApiExt<Block>
	+ sp_api::Metadata<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ sp_consensus_aura::AuraApi<Block, <<sp_consensus_aura::sr25519::AuthorityId as sp_runtime::app_crypto::AppCrypto>::Pair as sp_core::Pair>::Public>
	+ sp_offchain::OffchainWorkerApi<Block>
	+ sp_session::SessionKeys<Block>
	+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
	+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
{
}
impl<Api> RuntimeApiCollection for Api where
	Api: cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ fp_rpc::ConvertTransactionRuntimeApi<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
		+ moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ sp_api::ApiExt<Block>
		+ sp_api::Metadata<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_consensus_aura::AuraApi<Block, <<sp_consensus_aura::sr25519::AuthorityId as sp_runtime::app_crypto::AppCrypto>::Pair as sp_core::Pair>::Public>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
{
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial<RuntimeApi>(
	config: &sc_service::Configuration,
	eth_rpc_config: &crate::cli::EthRpcConfig,
) -> Result<Service<RuntimeApi>, sc_service::Error>
where
	RuntimeApi: 'static + Send + Sync + sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = sc_telemetry::TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;
	let executor = sc_service::new_wasm_executor::<HostFunctions>(config);
	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts_record_import::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
			true,
		)?;
	let client = Arc::new(client);
	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});
	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);
	let block_import = ParachainBlockImport::new(client.clone(), backend.clone());
	let import_queue = build_import_queue(
		client.clone(),
		block_import.clone(),
		config,
		telemetry.as_ref().map(|telemetry| telemetry.handle()),
		&task_manager,
	)?;
	// Frontier stuffs.
	let (storage_override, frontier_backend) =
		frontier::backend(client.clone(), config, eth_rpc_config.clone())?;
	let filter_pool = Some(Arc::new(Mutex::new(BTreeMap::new())));
	let fee_history_cache = Arc::new(Mutex::new(BTreeMap::new()));
	let fee_history_cache_limit = eth_rpc_config.fee_history_limit;

	Ok(sc_service::PartialComponents {
		backend: backend.clone(),
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain: sc_consensus::LongestChain::new(backend),
		other: (
			storage_override,
			frontier_backend,
			filter_pool,
			fee_history_cache,
			fee_history_cache_limit,
			block_import,
			telemetry,
			telemetry_worker_handle,
		),
	})
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[allow(clippy::too_many_arguments)]
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<Net, RuntimeApi, SC>(
	parachain_config: sc_service::Configuration,
	polkadot_config: sc_service::Configuration,
	collator_options: cumulus_client_cli::CollatorOptions,
	sybil_resistance_level: cumulus_client_service::CollatorSybilResistance,
	para_id: cumulus_primitives_core::ParaId,
	start_consensus: SC,
	no_hardware_benchmarks: bool,
	storage_monitor: sc_storage_monitor::StorageMonitorParams,
	eth_rpc_config: &crate::cli::EthRpcConfig,
) -> sc_service::error::Result<(sc_service::TaskManager, Arc<FullClient<RuntimeApi>>)>
where
	Net: sc_network::NetworkBackend<Block, Hash>,
	RuntimeApi: 'static + Send + Sync + sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	SC: FnOnce(
		Arc<FullClient<RuntimeApi>>,
		Arc<FullBackend>,
		ParachainBlockImport<RuntimeApi>,
		Option<&substrate_prometheus_endpoint::Registry>,
		Option<sc_telemetry::TelemetryHandle>,
		&sc_service::TaskManager,
		Arc<dyn cumulus_relay_chain_interface::RelayChainInterface>,
		Arc<sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi>>>,
		sp_keystore::KeystorePtr,
		Duration,
		cumulus_primitives_core::ParaId,
		polkadot_primitives::CollatorPair,
		cumulus_relay_chain_interface::OverseerHandle,
		Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
	) -> Result<(), sc_service::Error>,
{
	let mut parachain_config = cumulus_client_service::prepare_node_config(parachain_config);
	let sc_service::PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		mut task_manager,
		transaction_pool,
		select_chain: _,
		other:
			(
				storage_override,
				frontier_backend,
				filter_pool,
				fee_history_cache,
				fee_history_cache_limit,
				block_import,
				mut telemetry,
				telemetry_worker_handle,
			),
	} = new_partial::<RuntimeApi>(&parachain_config, eth_rpc_config)?;
	let database_path = parachain_config.database.path().map(|p| p.to_path_buf());
	let hwbench = (!no_hardware_benchmarks)
		.then_some(database_path.as_ref().map(|p| {
			let _ = std::fs::create_dir_all(p);

			sc_sysinfo::gather_hwbench(Some(p))
		}))
		.flatten();
	let (relay_chain_interface, collator_key) =
		cumulus_client_service::build_relay_chain_interface(
			polkadot_config,
			&parachain_config,
			telemetry_worker_handle,
			&mut task_manager,
			collator_options.clone(),
			hwbench.clone(),
		)
		.await
		.map_err(|e| sc_service::Error::Application(Box::new(e) as Box<_>))?;
	let frontier_backend = Arc::new(frontier_backend);
	let collator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let import_queue_service = import_queue.service();
	let net_config =
		<sc_network::config::FullNetworkConfiguration<_, _, Net>>::new(&parachain_config.network);
	let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
		cumulus_client_service::build_network(cumulus_client_service::BuildNetworkParams {
			parachain_config: &parachain_config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			para_id,
			spawn_handle: task_manager.spawn_handle(),
			relay_chain_interface: relay_chain_interface.clone(),
			import_queue,
			sybil_resistance_level,
		})
		.await?;

	if parachain_config.offchain_worker.enabled {
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-work",
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				keystore: Some(keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(
					sc_transaction_pool_api::OffchainTransactionPoolFactory::new(
						transaction_pool.clone(),
					),
				),
				network_provider: Arc::new(network.clone()),
				is_validator: parachain_config.role.is_authority(),
				enable_http_requests: false,
				custom_extensions: move |_| Vec::new(),
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
		task_manager.spawn_handle(),
		storage_override.clone(),
		eth_rpc_config.eth_log_block_cache,
		eth_rpc_config.eth_statuses_cache,
		prometheus_registry.clone(),
	));
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<Block>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);
	// for ethereum-compatibility rpc.
	parachain_config.rpc_id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider));
	let tracing_requesters = frontier::spawn_tasks(
		&task_manager,
		client.clone(),
		backend.clone(),
		frontier_backend.clone(),
		filter_pool.clone(),
		storage_override.clone(),
		fee_history_cache.clone(),
		fee_history_cache_limit,
		sync_service.clone(),
		pubsub_notification_sinks.clone(),
		eth_rpc_config.clone(),
		prometheus_registry.clone(),
	);
	let rpc_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let filter_pool = filter_pool.clone();
		let storage_override = storage_override;
		let fee_history_cache = fee_history_cache.clone();
		let max_past_logs = eth_rpc_config.max_past_logs;
		let collator = parachain_config.role.is_authority();
		let eth_rpc_config = eth_rpc_config.clone();
		let sync_service = sync_service.clone();
		let pending_create_inherent_data_providers = move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			let relay_chain_slot = Slot::from_timestamp(
				timestamp.timestamp(),
				SlotDuration::from_millis(RELAY_CHAIN_SLOT_DURATION_MILLIS as u64),
			);
			// Create a mocked parachain inherent data provider to pass all validations in the
			// parachain system.
			// Without this, the pending functionality will fail.
			let state_proof_builder = cumulus_test_relay_sproof_builder::RelayStateSproofBuilder {
				para_id,
				current_slot: relay_chain_slot,
				included_para_head: Some(polkadot_primitives::HeadData(vec![])),
				..Default::default()
			};
			let (relay_parent_storage_root, relay_chain_state) =
				state_proof_builder.into_state_root_and_proof();
			let parachain_inherent_data =
				cumulus_primitives_parachain_inherent::ParachainInherentData {
					validation_data: cumulus_primitives_core::PersistedValidationData {
						relay_parent_number: u32::MAX,
						relay_parent_storage_root,
						..Default::default()
					},
					relay_chain_state,
					downward_messages: Default::default(),
					horizontal_messages: Default::default(),
				};

			Ok((timestamp, parachain_inherent_data))
		};

		Box::new(move |deny_unsafe, subscription_task_executor| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				deny_unsafe,
				is_authority: collator,
				network: network.clone(),
				sync: sync_service.clone(),
				filter_pool: filter_pool.clone(),
				frontier_backend: match &*frontier_backend {
					fc_db::Backend::KeyValue(bd) => bd.clone(),
					fc_db::Backend::Sql(bd) => bd.clone(),
				},
				max_past_logs,
				fee_history_cache: fee_history_cache.clone(),
				fee_history_cache_limit,
				storage_override: storage_override.clone(),
				block_data_cache: block_data_cache.clone(),
				forced_parent_hashes: None,
				pending_create_inherent_data_providers,
			};

			if eth_rpc_config.tracing_api.contains(&crate::cli::TracingApi::Debug)
				|| eth_rpc_config.tracing_api.contains(&crate::cli::TracingApi::Trace)
			{
				crate::rpc::create_full::<_, _, _, _, crate::rpc::DefaultEthConfig<_, _>, _>(
					deps,
					subscription_task_executor,
					pubsub_notification_sinks.clone(),
					Some(crate::rpc::TracingConfig {
						tracing_requesters: tracing_requesters.clone(),
						trace_filter_max_count: eth_rpc_config.tracing_max_count,
					}),
				)
				.map_err(Into::into)
			} else {
				crate::rpc::create_full::<_, _, _, _, crate::rpc::DefaultEthConfig<_, _>, _>(
					deps,
					subscription_task_executor,
					pubsub_notification_sinks.clone(),
					None,
				)
				.map_err(Into::into)
			}
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: keystore_container.keystore(),
		backend: backend.clone(),
		network,
		sync_service: sync_service.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);

		// Here you can check whether the hardware meets your chains' requirements. Putting a link
		// in there and swapping out the requirements for your own are probably a good idea. The
		// requirements for a para-chain are dictated by its relay-chain.
		if let Err(e) =
			frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench)
		{
			log::warn!(
				"⚠️  The hardware does not meet the minimal requirements {e} for role 'Authority'.",
			);
		}

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}
	if let Some(database_path) = database_path {
		sc_storage_monitor::StorageMonitorService::try_spawn(
			storage_monitor,
			database_path,
			&task_manager.spawn_essential_handle(),
		)
		.map_err(|e| sc_service::Error::Application(e.into()))?;
	}

	let announce_block = {
		let sync_service = sync_service.clone();
		Arc::new(move |hash, data| sync_service.announce_block(hash, data))
	};
	let relay_chain_slot_duration = Duration::from_secs(6);
	let overseer_handle = relay_chain_interface
		.overseer_handle()
		.map_err(|e| sc_service::Error::Application(Box::new(e)))?;

	cumulus_client_service::start_relay_chain_tasks(
		cumulus_client_service::StartRelayChainTasksParams {
			client: client.clone(),
			announce_block: announce_block.clone(),
			para_id,
			relay_chain_interface: relay_chain_interface.clone(),
			task_manager: &mut task_manager,
			da_recovery_profile: if collator {
				cumulus_client_service::DARecoveryProfile::Collator
			} else {
				cumulus_client_service::DARecoveryProfile::FullNode
			},
			import_queue: import_queue_service,
			relay_chain_slot_duration,
			recovery_handle: Box::new(overseer_handle.clone()),
			sync_service: sync_service.clone(),
		},
	)?;

	if collator {
		start_consensus(
			client.clone(),
			backend.clone(),
			block_import,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface.clone(),
			transaction_pool,
			keystore_container.keystore(),
			relay_chain_slot_duration,
			para_id,
			collator_key.expect("Command line arguments do not allow this. qed"),
			overseer_handle,
			announce_block,
		)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}

/// Build the import queue for the parachain runtime.
fn build_import_queue<RuntimeApi>(
	client: Arc<FullClient<RuntimeApi>>,
	block_import: ParachainBlockImport<RuntimeApi>,
	config: &sc_service::Configuration,
	telemetry: Option<sc_telemetry::TelemetryHandle>,
	task_manager: &sc_service::TaskManager,
) -> Result<sc_consensus::DefaultImportQueue<Block>, sc_service::Error>
where
	RuntimeApi: 'static + Send + Sync + sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
{
	Ok(cumulus_client_consensus_aura::equivocation_import_queue::fully_verifying_import_queue::<
		sp_consensus_aura::sr25519::AuthorityPair,
		_,
		_,
		_,
		_,
	>(
		client,
		block_import,
		move |_, _| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

			Ok(timestamp)
		},
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
		telemetry,
	))
}

/// Start a parachain node.
pub async fn start_parachain_node<Net, RuntimeApi>(
	parachain_config: sc_service::Configuration,
	polkadot_config: sc_service::Configuration,
	collator_options: cumulus_client_cli::CollatorOptions,
	para_id: cumulus_primitives_core::ParaId,
	no_hardware_benchmarks: bool,
	storage_monitor: sc_storage_monitor::StorageMonitorParams,
	eth_rpc_config: &crate::cli::EthRpcConfig,
) -> sc_service::error::Result<(sc_service::TaskManager, Arc<FullClient<RuntimeApi>>)>
where
	Net: sc_network::NetworkBackend<Block, Hash>,
	RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	RuntimeApi::RuntimeApi:
		sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
{
	start_node_impl::<Net, RuntimeApi, _>(
		parachain_config,
		polkadot_config,
		collator_options,
		cumulus_client_service::CollatorSybilResistance::Resistant, // Aura
		para_id,
		|client,
		 backend,
		 block_import,
		 prometheus_registry,
		 telemetry,
		 task_manager,
		 relay_chain_interface,
		 transaction_pool,
		 keystore,
		 relay_chain_slot_duration,
		 para_id,
		 collator_key,
		 overseer_handle,
		 announce_block| {
			let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
				task_manager.spawn_handle(),
				client.clone(),
				transaction_pool,
				prometheus_registry,
				telemetry,
			);
			let proposer = cumulus_client_consensus_proposer::Proposer::new(proposer_factory);
			let collator_service = cumulus_client_collator::service::CollatorService::new(
				client.clone(),
				Arc::new(task_manager.spawn_handle()),
				announce_block,
				client.clone(),
			);
			let params = cumulus_client_consensus_aura::collators::lookahead::Params {
				create_inherent_data_providers: move |_, ()| async move { Ok(()) },
				block_import,
				para_client: client.clone(),
				para_backend: backend.clone(),
				relay_client: relay_chain_interface,
				code_hash_provider: move |block_hash| {
					client.code_at(block_hash).ok().map(|c| {
						cumulus_primitives_core::relay_chain::ValidationCode::from(c).hash()
					})
				},
				keystore,
				collator_key,
				para_id,
				overseer_handle,
				relay_chain_slot_duration,
				proposer,
				collator_service,
				// Very limited proposal time.
				authoring_duration: Duration::from_millis(1_500),
				reinitialize: false,
			};
			let fut = cumulus_client_consensus_aura::collators::lookahead::run::<
				Block,
				sp_consensus_aura::sr25519::AuthorityPair,
				_,
				_,
				_,
				_,
				_,
				_,
				_,
				_,
			>(params);

			task_manager.spawn_essential_handle().spawn("aura", None, fut);

			Ok(())
		},
		no_hardware_benchmarks,
		storage_monitor,
		eth_rpc_config,
	)
	.await
}

/// Start a dev node which can seal instantly.
/// !!! WARNING: DO NOT USE ELSEWHERE
pub fn start_dev_node<Net, RuntimeApi>(
	mut config: sc_service::Configuration,
	para_id: cumulus_primitives_core::ParaId,
	eth_rpc_config: &crate::cli::EthRpcConfig,
) -> Result<sc_service::TaskManager, sc_service::error::Error>
where
	Net: sc_network::NetworkBackend<Block, Hash>,
	RuntimeApi: 'static + Send + Sync + sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	RuntimeApi::RuntimeApi:
		sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
{
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other:
			(
				storage_override,
				frontier_backend,
				filter_pool,
				fee_history_cache,
				fee_history_cache_limit,
				_block_import,
				_telemetry,
				_telemetry_worker_handle,
			),
	} = new_partial::<RuntimeApi>(&config, eth_rpc_config)?;
	let net_config =
		<sc_network::config::FullNetworkConfiguration<_, _, Net>>::new(&config.network);
	let metrics = Net::register_notification_metrics(None);
	let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: None,
			block_relay: None,
			metrics,
		})?;

	if config.offchain_worker.enabled {
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-work",
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				keystore: None,
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(
					sc_transaction_pool_api::OffchainTransactionPoolFactory::new(
						transaction_pool.clone(),
					),
				),
				network_provider: Arc::new(network.clone()),
				is_validator: config.role.is_authority(),
				enable_http_requests: false,
				custom_extensions: move |_| Vec::new(),
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	let frontier_backend = Arc::new(frontier_backend);
	let force_authoring = config.force_authoring;
	let backoff_authoring_blocks = None::<()>;
	let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
	let proposer_factory = sc_basic_authorship::ProposerFactory::new(
		task_manager.spawn_handle(),
		client.clone(),
		transaction_pool.clone(),
		None,
		None,
	);
	let client_for_cidp = client.clone();
	if config.role.is_authority() {
		let aura = sc_consensus_aura::start_aura::<
			sp_consensus_aura::sr25519::AuthorityPair,
			_,
			_,
			_,
			_,
			_,
			_,
			_,
			_,
			_,
			_,
		>(sc_consensus_aura::StartAuraParams {
			slot_duration,
			client: client.clone(),
			select_chain,
			block_import: instant_finalize::InstantFinalizeBlockImport::new(client.clone()),
			proposer_factory,
			create_inherent_data_providers: move |block, ()| {
				let maybe_current_para_block = client_for_cidp.number(block);
				let maybe_current_block_head = client_for_cidp.expect_header(block);
				let client_for_xcm = client_for_cidp.clone();
				// TODO: hack for now.
				let additional_key_values = Some(vec![(
					array_bytes::hex2bytes_unchecked(
						"1cb6f36e027abb2091cfb5110ab5087f06155b3cd9a8c9e5e9a23fd5dc13a5ed",
					),
					cumulus_primitives_aura::Slot::from_timestamp(
						sp_timestamp::Timestamp::current(),
						slot_duration,
					)
					.encode(),
				)]);
				async move {
					let current_para_block = maybe_current_para_block?
						.ok_or(sp_blockchain::Error::UnknownBlock(block.to_string()))?;
					let current_para_block_head =
						Some(polkadot_primitives::HeadData(maybe_current_block_head?.encode()));
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
					let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
						*timestamp,
						slot_duration,
					);
					let mocked_parachain =
						cumulus_client_parachain_inherent::MockValidationDataInherentDataProvider {
							current_para_block,
							para_id,
							current_para_block_head,
							relay_offset: 1000,
							relay_blocks_per_para_block: 2,
							para_blocks_per_relay_epoch: 0,
							relay_randomness_config: (),
							xcm_config: cumulus_client_parachain_inherent::MockXcmConfig::new(
								&*client_for_xcm,
								block,
								Default::default(),
							),
							raw_downward_messages: Vec::new(),
							raw_horizontal_messages: Vec::new(),
							additional_key_values,
						};

					Ok((slot, timestamp, mocked_parachain))
				}
			},
			force_authoring,
			backoff_authoring_blocks,
			keystore: keystore_container.keystore(),
			sync_oracle: sync_service.clone(),
			justification_sync_link: sync_service.clone(),
			// We got around 500ms for proposing
			block_proposal_slot_portion: cumulus_client_consensus_aura::SlotProportion::new(
				1f32 / 24f32,
			),
			// And a maximum of 750ms if slots are skipped
			max_block_proposal_slot_portion: Some(
				cumulus_client_consensus_aura::SlotProportion::new(1f32 / 16f32),
			),
			telemetry: None,
			compatibility_mode: Default::default(),
		})?;

		// the AURA authoring task is considered essential, i.e. if it
		// fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking("aura", Some("block-authoring"), aura);
	} else {
		log::warn!("You could add --alice or --bob to make dev chain seal instantly.");
	}

	let prometheus_registry = config.prometheus_registry().cloned();
	let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
		task_manager.spawn_handle(),
		storage_override.clone(),
		eth_rpc_config.eth_log_block_cache,
		eth_rpc_config.eth_statuses_cache,
		prometheus_registry.clone(),
	));
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<Block>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);
	// for ethereum-compatibility rpc.
	config.rpc_id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider));
	let tracing_requesters = frontier::spawn_tasks(
		&task_manager,
		client.clone(),
		backend.clone(),
		frontier_backend.clone(),
		filter_pool.clone(),
		storage_override.clone(),
		fee_history_cache.clone(),
		fee_history_cache_limit,
		sync_service.clone(),
		pubsub_notification_sinks.clone(),
		eth_rpc_config.clone(),
		prometheus_registry,
	);
	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let filter_pool = filter_pool;
		let frontier_backend = frontier_backend;
		let storage_override = storage_override;
		let fee_history_cache = fee_history_cache;
		let max_past_logs = eth_rpc_config.max_past_logs;
		let collator = config.role.is_authority();
		let eth_rpc_config = eth_rpc_config.clone();
		let sync_service = sync_service.clone();

		let pending_create_inherent_data_providers = move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			let relay_chain_slot = Slot::from_timestamp(
				timestamp.timestamp(),
				SlotDuration::from_millis(RELAY_CHAIN_SLOT_DURATION_MILLIS as u64),
			);
			// Create a mocked parachain inherent data provider to pass all validations in the
			// parachain system.
			// Without this, the pending functionality will fail.
			let state_proof_builder = cumulus_test_relay_sproof_builder::RelayStateSproofBuilder {
				para_id,
				current_slot: relay_chain_slot,
				included_para_head: Some(polkadot_primitives::HeadData(vec![])),
				..Default::default()
			};
			let (relay_parent_storage_root, relay_chain_state) =
				state_proof_builder.into_state_root_and_proof();
			let parachain_inherent_data =
				cumulus_primitives_parachain_inherent::ParachainInherentData {
					validation_data: cumulus_primitives_core::PersistedValidationData {
						relay_parent_number: u32::MAX,
						relay_parent_storage_root,
						..Default::default()
					},
					relay_chain_state,
					downward_messages: Default::default(),
					horizontal_messages: Default::default(),
				};
			Ok((timestamp, parachain_inherent_data))
		};

		Box::new(move |deny_unsafe, subscription_task_executor| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				deny_unsafe,
				is_authority: collator,
				network: network.clone(),
				sync: sync_service.clone(),
				filter_pool: filter_pool.clone(),
				frontier_backend: match &*frontier_backend {
					fc_db::Backend::KeyValue(bd) => bd.clone(),
					fc_db::Backend::Sql(bd) => bd.clone(),
				},
				max_past_logs,
				fee_history_cache: fee_history_cache.clone(),
				fee_history_cache_limit,
				storage_override: storage_override.clone(),
				block_data_cache: block_data_cache.clone(),
				forced_parent_hashes: None,
				pending_create_inherent_data_providers,
			};

			if eth_rpc_config.tracing_api.contains(&crate::cli::TracingApi::Debug)
				|| eth_rpc_config.tracing_api.contains(&crate::cli::TracingApi::Trace)
			{
				crate::rpc::create_full::<_, _, _, _, crate::rpc::DefaultEthConfig<_, _>, _>(
					deps,
					subscription_task_executor,
					pubsub_notification_sinks.clone(),
					Some(crate::rpc::TracingConfig {
						tracing_requesters: tracing_requesters.clone(),
						trace_filter_max_count: eth_rpc_config.tracing_max_count,
					}),
				)
				.map_err(Into::into)
			} else {
				crate::rpc::create_full::<_, _, _, _, crate::rpc::DefaultEthConfig<_, _>, _>(
					deps,
					subscription_task_executor,
					pubsub_notification_sinks.clone(),
					None,
				)
				.map_err(Into::into)
			}
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder: Box::new(rpc_extensions_builder),
		client,
		transaction_pool,
		task_manager: &mut task_manager,
		config,
		keystore: keystore_container.keystore(),
		backend,
		network,
		sync_service,
		system_rpc_tx,
		tx_handler_controller,
		telemetry: None,
	})?;

	start_network.start_network();

	Ok(task_manager)
}
