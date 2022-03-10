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

//! Crab service. Specialized wrapper over substrate service.

pub use crab_runtime;

// --- std ---
use std::{
	collections::BTreeMap,
	path::PathBuf,
	sync::{Arc, Mutex},
	time::Duration,
};
// --- crates.io ---
use futures::stream::StreamExt;
// --- paritytech ---
use fc_rpc_core::types::FilterPool;
use sc_authority_discovery::WorkerConfig;
use sc_basic_authorship::ProposerFactory;
use sc_client_api::{ExecutorProvider, StateBackendFor};
use sc_consensus::{BasicQueue, DefaultImportQueue, LongestChain};
use sc_consensus_babe::{
	BabeBlockImport, BabeLink, BabeParams, Config as BabeConfig, SlotProportion,
};
use sc_executor::{NativeElseWasmExecutor, NativeExecutionDispatch};
use sc_finality_grandpa::{
	warp_proof::NetworkProvider, Config as GrandpaConfig,
	FinalityProofProvider as GrandpaFinalityProofProvider, GrandpaParams, LinkHalf,
	SharedVoterState as GrandpaSharedVoterState, VotingRulesBuilder as GrandpaVotingRulesBuilder,
};
use sc_network::Event;
use sc_service::{
	config::KeystoreConfig, BasePath, BuildNetworkParams, Configuration, Error as ServiceError,
	PartialComponents, RpcHandlers, SpawnTasksParams, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool::{BasicPool, FullPool};
use sp_api::ConstructRuntimeApi;
use sp_consensus::CanAuthorWithNativeVersion;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use sp_trie::PrefixedMemoryDB;
// --- darwinia-network ---
use crate::{
	client::CrabClient,
	service::{self, RpcResult, *},
};
use darwinia_common_primitives::*;
use darwinia_rpc::{crab::FullDeps, *};
use dc_db::{Backend, DatabaseSettings, DatabaseSettingsSrc};

pub struct Executor;
impl NativeExecutionDispatch for Executor {
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		crab_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		crab_runtime::native_version()
	}
}

impl_runtime_apis![
	darwinia_fee_market_rpc_runtime_api::FeeMarketApi<Block, Balance>,
	dvm_rpc_runtime_api::EthereumRuntimeRPCApi<Block>,
	dp_evm_trace_apis::DebugRuntimeApi<Block>
];

pub fn dvm_database_dir(config: &Configuration) -> PathBuf {
	let config_dir = config
		.base_path
		.as_ref()
		.map(|base_path| base_path.config_dir(config.chain_spec.id()))
		.unwrap_or_else(|| {
			BasePath::from_project("", "", "crab").config_dir(config.chain_spec.id())
		});

	config_dir.join("dvm").join("db")
}

fn open_dvm_backend(config: &Configuration) -> Result<Arc<Backend<Block>>, String> {
	Ok(Arc::new(Backend::<Block>::new(&DatabaseSettings {
		source: DatabaseSettingsSrc::RocksDb {
			path: dvm_database_dir(&config),
			cache_size: 0,
		},
	})?))
}

#[cfg(feature = "full-node")]
fn new_partial<RuntimeApi, Executor>(
	config: &mut Configuration,
) -> Result<
	PartialComponents<
		FullClient<RuntimeApi, Executor>,
		FullBackend,
		FullSelectChain,
		DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
		FullPool<Block, FullClient<RuntimeApi, Executor>>,
		(
			(
				BabeBlockImport<
					Block,
					FullClient<RuntimeApi, Executor>,
					FullGrandpaBlockImport<RuntimeApi, Executor>,
				>,
				LinkHalf<Block, FullClient<RuntimeApi, Executor>, FullSelectChain>,
				BabeLink<Block>,
			),
			Option<Telemetry>,
		),
	>,
	ServiceError,
>
where
	Executor: 'static + NativeExecutionDispatch,
	RuntimeApi:
		'static + Send + Sync + ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
{
	if config.keystore_remote.is_some() {
		return Err(ServiceError::Other(format!(
			"Remote Keystores are not supported."
		)));
	}

	service::set_prometheus_registry(config)?;

	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;
	let executor = <NativeElseWasmExecutor<Executor>>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
	);
	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			&config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", worker.run());
		telemetry
	});
	let client = Arc::new(client);
	let select_chain = LongestChain::new(backend.clone());
	let transaction_pool = BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);
	let grandpa_hard_forks = vec![];
	let (grandpa_block_import, grandpa_link) =
		sc_finality_grandpa::block_import_with_authority_set_hard_forks(
			client.clone(),
			&(client.clone() as Arc<_>),
			select_chain.clone(),
			grandpa_hard_forks,
			telemetry.as_ref().map(|x| x.handle()),
		)?;
	let justification_import = grandpa_block_import.clone();
	let (babe_import, babe_link) = sc_consensus_babe::block_import(
		BabeConfig::get_or_compute(&*client)?,
		grandpa_block_import,
		client.clone(),
	)?;
	let slot_duration = babe_link.config().slot_duration();
	let import_queue = sc_consensus_babe::import_queue(
		babe_link.clone(),
		babe_import.clone(),
		Some(Box::new(justification_import)),
		client.clone(),
		select_chain.clone(),
		move |_, ()| async move {
			let uncles =
				sp_authorship::InherentDataProvider::<<Block as BlockT>::Header>::check_inherents();
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			let slot =
				sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_duration(
					*timestamp,
					slot_duration,
				);

			Ok((timestamp, slot, uncles))
		},
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
		CanAuthorWithNativeVersion::new(client.executor().clone()),
		telemetry.as_ref().map(|x| x.handle()),
	)?;
	let import_setup = (babe_import.clone(), grandpa_link, babe_link.clone());

	Ok(PartialComponents {
		client,
		backend,
		task_manager,
		keystore_container,
		select_chain,
		import_queue,
		transaction_pool,
		other: (import_setup, telemetry),
	})
}

#[cfg(feature = "full-node")]
fn new_full<RuntimeApi, Executor>(
	mut config: Configuration,
	authority_discovery_disabled: bool,
	eth_rpc_config: EthRpcConfig,
) -> Result<
	(
		TaskManager,
		Arc<FullClient<RuntimeApi, Executor>>,
		RpcHandlers,
	),
	ServiceError,
>
where
	Executor: 'static + NativeExecutionDispatch,
	RuntimeApi:
		'static + Send + Sync + ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
{
	let role = config.role.clone();
	let is_authority = role.is_authority();
	// let is_archive = config.state_pruning.is_archive();
	let force_authoring = config.force_authoring;
	let disable_grandpa = config.disable_grandpa;
	let name = config.network.node_name.clone();
	let prometheus_registry = config.prometheus_registry().cloned();
	let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;

	config
		.network
		.extra_sets
		.push(sc_finality_grandpa::grandpa_peers_set_config());

	let backoff_authoring_blocks =
		Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
	let PartialComponents {
		client,
		backend,
		mut task_manager,
		mut keystore_container,
		select_chain,
		import_queue,
		transaction_pool,
		other: ((babe_import, grandpa_link, babe_link), mut telemetry),
	} = new_partial::<RuntimeApi, Executor>(&mut config)?;

	if let Some(url) = &config.keystore_remote {
		match service::remote_keystore(url) {
			Ok(k) => keystore_container.set_remote_keystore(k),
			Err(e) => {
				return Err(ServiceError::Other(format!(
					"Error hooking up remote keystore for {}: {}",
					url, e
				)))
			}
		};
	}

	let warp_sync = Arc::new(NetworkProvider::new(
		backend.clone(),
		grandpa_link.shared_authority_set().clone(),
	));
	let (network, system_rpc_tx, network_starter) =
		sc_service::build_network(BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: None,
			warp_sync: Some(warp_sync),
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let dvm_backend = open_dvm_backend(&config)?;
	let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));
	// let eth_tracing_requesters = dvm_tasks::spawn(DvmTasksParams {
	// 	task_manager: &task_manager,
	// 	client: client.clone(),
	// 	substrate_backend: backend.clone(),
	// 	dvm_backend: dvm_backend.clone(),
	// 	filter_pool: filter_pool.clone(),
	// 	is_archive,
	// 	rpc_config: eth_rpc_config.clone(),
	// });
	let subscription_task_executor = SubscriptionTaskExecutor::new(task_manager.spawn_handle());
	let shared_voter_state = GrandpaSharedVoterState::empty();
	let babe_config = babe_link.config().clone();
	let shared_epoch_changes = babe_link.epoch_changes().clone();
	let justification_stream = grandpa_link.justification_stream();
	let shared_authority_set = grandpa_link.shared_authority_set().clone();
	let finality_proof_provider = GrandpaFinalityProofProvider::new_for_service(
		backend.clone(),
		Some(shared_authority_set.clone()),
	);
	let rpc_extensions_builder = {
		let client = client.clone();
		let keystore = keystore_container.sync_keystore();
		let transaction_pool = transaction_pool.clone();
		let select_chain = select_chain.clone();
		let chain_spec = config.chain_spec.cloned_box();
		let shared_voter_state = shared_voter_state.clone();

		move |deny_unsafe, is_authority, network, subscription_executor| -> RpcResult {
			let deps = FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				graph: transaction_pool.pool().clone(),
				select_chain: select_chain.clone(),
				chain_spec: chain_spec.cloned_box(),
				deny_unsafe,
				is_authority,
				network,
				babe: BabeDeps {
					babe_config: babe_config.clone(),
					shared_epoch_changes: shared_epoch_changes.clone(),
					keystore: keystore.clone(),
				},
				grandpa: GrandpaDeps {
					shared_voter_state: shared_voter_state.clone(),
					shared_authority_set: shared_authority_set.clone(),
					justification_stream: justification_stream.clone(),
					subscription_executor,
					finality_provider: finality_proof_provider.clone(),
				},
				backend: dvm_backend.clone(),
				filter_pool: filter_pool.clone(),
				// eth_tracing_requesters: tracing_requesters.clone(),
				eth_rpc_config: eth_rpc_config.clone(),
			};

			darwinia_rpc::crab::create_full(deps, subscription_task_executor.clone())
				.map_err(Into::into)
		}
	};
	let rpc_handlers = sc_service::spawn_tasks(SpawnTasksParams {
		config,
		backend,
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		network: network.clone(),
		rpc_extensions_builder: {
			let network = network.clone();
			let wrap_rpc_extensions_builder = {
				move |deny_unsafe, subscription_executor| -> RpcResult {
					rpc_extensions_builder(
						deny_unsafe,
						is_authority,
						network.clone(),
						subscription_executor,
					)
				}
			};

			Box::new(wrap_rpc_extensions_builder)
		},
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		on_demand: None,
		remote_blockchain: None,
		system_rpc_tx,
		telemetry: telemetry.as_mut(),
	})?;

	if is_authority {
		let can_author_with = CanAuthorWithNativeVersion::new(client.executor().clone());
		let proposer = ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);
		let client_clone = client.clone();
		let slot_duration = babe_link.config().slot_duration();
		let babe_config = BabeParams {
			keystore: keystore_container.sync_keystore(),
			client: client.clone(),
			select_chain,
			block_import: babe_import,
			env: proposer,
			sync_oracle: network.clone(),
			justification_sync_link: network.clone(),
			create_inherent_data_providers: move |parent, ()| {
				let client_clone = client_clone.clone();
				async move {
					let uncles = sc_consensus_uncles::create_uncles_inherent_data_provider(
						&*client_clone,
						parent,
					)?;
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
					let slot =
						sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_duration(
							*timestamp,
							slot_duration,
						);

					Ok((timestamp, slot, uncles))
				}
			},
			force_authoring,
			backoff_authoring_blocks,
			babe_link,
			can_author_with,
			block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
			max_block_proposal_slot_portion: None,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
		};
		let babe = sc_consensus_babe::start_babe(babe_config)?;

		task_manager
			.spawn_essential_handle()
			.spawn_blocking("babe", babe);
	}

	if is_authority && !authority_discovery_disabled {
		let authority_discovery_role =
			sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore());
		let dht_event_stream =
			network
				.event_stream("authority-discovery")
				.filter_map(|e| async move {
					match e {
						Event::Dht(e) => Some(e),
						_ => None,
					}
				});
		let (authority_discovery_worker, _service) =
			sc_authority_discovery::new_worker_and_service_with_config(
				WorkerConfig {
					publish_non_global_ips: auth_disc_publish_non_global_ips,
					..Default::default()
				},
				client.clone(),
				network.clone(),
				Box::pin(dht_event_stream),
				authority_discovery_role,
				prometheus_registry.clone(),
			);

		task_manager.spawn_handle().spawn(
			"authority-discovery-worker",
			authority_discovery_worker.run(),
		);
	}

	let keystore = if is_authority {
		Some(keystore_container.sync_keystore())
	} else {
		None
	};
	let grandpa_config = GrandpaConfig {
		// FIXME substrate#1578 make this available through chainspec
		gossip_duration: Duration::from_millis(1000),
		justification_period: 512,
		name: Some(name),
		observer_enabled: false,
		keystore,
		local_role: role,
		telemetry: telemetry.as_ref().map(|x| x.handle()),
	};
	let enable_grandpa = !disable_grandpa;

	if enable_grandpa {
		let grandpa_config = GrandpaParams {
			config: grandpa_config,
			link: grandpa_link,
			network,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			voting_rule: GrandpaVotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state,
		};

		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	network_starter.start_network();

	Ok((task_manager, client, rpc_handlers))
}

/// Builds a new object suitable for chain operations.
#[cfg(feature = "full-node")]
pub fn new_chain_ops<Runtime, Dispatch>(
	config: &mut Configuration,
) -> Result<
	(
		Arc<FullClient<Runtime, Dispatch>>,
		Arc<FullBackend>,
		BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
	),
	ServiceError,
>
where
	Dispatch: 'static + NativeExecutionDispatch,
	Runtime: 'static + Send + Sync + ConstructRuntimeApi<Block, FullClient<Runtime, Dispatch>>,
	Runtime::RuntimeApi: RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
{
	config.keystore = KeystoreConfig::InMemory;

	let PartialComponents {
		client,
		backend,
		import_queue,
		task_manager,
		..
	} = new_partial::<Runtime, Dispatch>(config)?;

	Ok((client, backend, import_queue, task_manager))
}

/// Create a new Crab service for a full node.
#[cfg(feature = "full-node")]
pub fn crab_new_full(
	config: Configuration,
	authority_discovery_disabled: bool,
	eth_rpc_config: EthRpcConfig,
) -> Result<
	(
		TaskManager,
		Arc<impl CrabClient<Block, FullBackend, crab_runtime::RuntimeApi>>,
		RpcHandlers,
	),
	ServiceError,
> {
	let (components, client, rpc_handlers) = new_full::<crab_runtime::RuntimeApi, CrabExecutor>(
		config,
		authority_discovery_disabled,
		eth_rpc_config,
	)?;

	Ok((components, client, rpc_handlers))
}
