pub mod crab;
pub mod darwinia;

use sc_service::{config::PrometheusConfig, ChainSpec, Configuration, Error as ServiceError};
use substrate_prometheus_endpoint::Registry;

/// Can be called for a `Configuration` to check if it is a configuration for the `Crab` network.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Crab` network.
	fn is_crab(&self) -> bool;

	/// Returns if this is a configuration for the `Darwinia` network.
	fn is_darwinia(&self) -> bool;
}
impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_crab(&self) -> bool {
		self.id().starts_with("crab")
	}

	fn is_darwinia(&self) -> bool {
		self.id().starts_with("darwinia")
	}
}

// If we're using prometheus, use a registry with a prefix of `darwinia`.
fn set_prometheus_registry(config: &mut Configuration) -> Result<(), ServiceError> {
	if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
		*registry = Registry::new_custom(Some("darwinia".into()), None)?;
	}

	Ok(())
}
