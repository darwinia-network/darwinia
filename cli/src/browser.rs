// --- std ---
use std::str::FromStr;
// --- third-party ---
use log::info;
// --- substrate ---
use substrate_browser_utils::{
	browser_configuration, init_console_log, set_console_error_panic_hook, Client,
};
use wasm_bindgen::prelude::*;

/// Starts the client.
#[wasm_bindgen]
pub async fn start_client(chain_spec: String, log_level: String) -> Result<Client, JsValue> {
	start_inner(chain_spec, log_level)
		.await
		.map_err(|err| JsValue::from_str(&err.to_string()))
}

async fn start_inner(
	chain_spec: String,
	log_level: String,
) -> Result<Client, Box<dyn std::error::Error>> {
	set_console_error_panic_hook();
	init_console_log(log_level.parse()?)?;

	let chain_spec = service::PolkadotChainSpec::from_json_bytes(chain_spec.as_bytes().to_vec())
		.map_err(|e| format!("{:?}", e))?;
	let config = browser_configuration(chain_spec).await?;

	info!("Darwinia browser node");
	info!("  version {}", config.full_version());
	info!("  _____                      _       _       ");
	info!(" |  __ \\                    (_)     (_)      ");
	info!(" | |  | | __ _ _ ____      ___ _ __  _  __ _ ");
	info!(" | |  | |/ _` | '__\\ \\ /\\ / / | '_ \\| |/ _` |");
	info!(" | |__| | (_| | |   \\ V  V /| | | | | | (_| |");
	info!(" |_____/ \\__,_|_|    \\_/\\_/ |_|_| |_|_|\\__,_|");
	info!("  by Darwinia Network, 2018-2020");
	info!(
		"📋 Chain specification: {}",
		config.expect_chain_spec().name()
	);
	info!("🏷 Node name: {}", config.name);
	info!("👤 Roles: {}", config.roles);

	// Create the service. This is the most heavy initialization step.
	let service = service::kusama_new_light(config).map_err(|e| format!("{:?}", e))?;

	Ok(browser_utils::start_client(service))
}
