pub use frame_support::migration;

// polkadot-sdk
use frame_support::migration;

/// Pallet migration helper.
pub struct PalletCleaner {
	pub name: &'static [u8],
	pub values: &'static [&'static [u8]],
	pub maps: &'static [&'static [u8]],
}
impl PalletCleaner {
	/// Remove all storage from a pallet.
	pub fn remove_all(&self) -> u64 {
		self.remove_storage_values() + self.remove_storage_maps()
	}

	/// Remove multiple storage value from a pallet at once.
	pub fn remove_storage_values(&self) -> u64 {
		self.values.iter().for_each(|i| {
			let _ = migration::clear_storage_prefix(self.name, i, &[], None, None);
		});

		self.values.len() as u64
	}

	/// Remove multiple storage map from a pallet at once.
	pub fn remove_storage_maps(&self) -> u64 {
		self.maps.iter().fold(0, |acc, i| {
			acc + migration::clear_storage_prefix(self.name, i, &[], None, None).backend as u64
		})
	}
}
