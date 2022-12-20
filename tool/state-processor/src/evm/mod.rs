// darwinia
use crate::*;

// https://github.com/paritytech/frontier/blob/polkadot-v0.9.30/primitives/storage/src/lib.rs#L23
const PALLET_ETHEREUM_SCHEMA: &str = "0x3a657468657265756d5f736368656d61";

impl Processor {
	fn process_ethereum_schema(&mut self) -> &mut Self {
		log::info!("set `PALLET_ETHEREUM_SCHEMA`");
		self.shell_state.0.insert(PALLET_ETHEREUM_SCHEMA.into(), "0x3".into());

		self
	}

	pub fn process_evm(&mut self) -> &mut Self {
		self.process_ethereum_schema();

		// Storage items.
		// https://github.com/paritytech/frontier/blob/aca04f2269a9d6da2011f0c04069f0354fab01a1/frame/evm/src/lib.rs#L495-L502
		let mut account_codes = Map::default();
		let mut account_storages = Map::default();

		log::info!("take `EVM::AccountCodes` and `EVM::AccountStorages`");
		self.solo_state
			.take_raw(&item_key(b"EVM", b"AccountCodes"), &mut account_codes, |key, from| {
				replace_first_match(key, from, &item_key(b"Evm", b"AccountCodes"))
			})
			.take_raw(&item_key(b"EVM", b"AccountStorages"), &mut account_storages, |key, from| {
				replace_first_match(key, from, &item_key(b"Evm", b"AccountStorages"))
			});

		log::info!("set `Evm::AccountCodes` and `Evm::AccountStorages`");
		self.shell_state.insert_raw(account_codes).insert_raw(account_storages);

		self
	}
}

#[test]
fn pallet_ethereum_schema_should_work() {
	assert_eq!(array_bytes::bytes2hex("0x", b":ethereum_schema"), PALLET_ETHEREUM_SCHEMA);
}
