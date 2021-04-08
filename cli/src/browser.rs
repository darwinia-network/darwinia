// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
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
use std::{error::Error, str::FromStr};
// --- crates ---
use log::info;
use substrate_browser_utils::{
	browser_configuration, init_console_log, set_console_error_panic_hook, start_client, Client,
};
use wasm_bindgen::prelude::*;

/// Starts the client.
#[wasm_bindgen]
pub async fn start_client(chain_spec: String, log_level: String) -> Result<Client, JsValue> {
	start_inner(chain_spec, log_level)
		.await
		.map_err(|err| JsValue::from_str(&err.to_string()))
}

async fn start_inner(chain_spec: String, log_level: String) -> Result<Client, Box<dyn Error>> {
	set_console_error_panic_hook();
	init_console_log(log_level.parse()?)?;

	let chain_spec =
		darwinia_service::CrabChainSpec::from_json_bytes(chain_spec.as_bytes().to_vec())
			.map_err(|e| format!("{:?}", e))?;
	let config = browser_configuration(chain_spec).await?;

	info!("Darwinia browser node");
	info!("  version {}", config.impl_version);
	info!("  _____                      _       _       ");
	info!(" |  __ \\                    (_)     (_)      ");
	info!(" | |  | | __ _ _ ____      ___ _ __  _  __ _ ");
	info!(" | |  | |/ _` | '__\\ \\ /\\ / / | '_ \\| |/ _` |");
	info!(" | |__| | (_| | |   \\ V  V /| | | | | | (_| |");
	info!(" |_____/ \\__,_|_|    \\_/\\_/ |_|_| |_|_|\\__,_|");
	info!("  by Darwinia Network, 2018-2020");
	info!("📋 Chain specification: {}", config.chain_spec.name());
	info!("🏷  Node name: {}", config.network.node_name);
	info!("👤 Role: {}", config.display_role());

	// Create the service. This is the most heavy initialization step.
	let (task_manager, rpc_handlers) =
		darwinia_service::crab_new_light(config).map_err(|e| format!("{:?}", e))?;

	Ok(start_client(task_manager, rpc_handlers))
}
