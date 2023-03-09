// darwinia
use crate::*;

// https://github.dev/paritytech/frontier/blob/polkadot-v0.9.30/primitives/storage/src/lib.rs#L23
const PALLET_ETHEREUM_SCHEMA: &str = "0x3a657468657265756d5f736368656d61";

impl<S> Processor<S> {
	fn process_ethereum_schema(&mut self) -> &mut Self {
		log::info!("set `PALLET_ETHEREUM_SCHEMA`");
		self.shell_state.insert_raw_key_raw_value(PALLET_ETHEREUM_SCHEMA.into(), "0x3".into());

		self
	}

	pub fn process_evm(&mut self) -> &mut Self {
		self.process_ethereum_schema();

		// Storage items.
		// https://github.dev/darwinia-network/frontier/blob/darwinia-v0.12.5/frame/evm/src/lib.rs#L407
		let mut codes = Map::default();
		let mut storages = Map::default();

		log::info!("take `EVM::AccountCodes` and `EVM::AccountStorages`");
		self.solo_state
			.take_prefix_raw(&item_key(b"EVM", b"AccountCodes"), &mut codes, |k, _| k.to_owned())
			.take_prefix_raw(&item_key(b"EVM", b"AccountStorages"), &mut storages, |k, _| {
				k.to_owned()
			});

		log::info!("set `EVM::AccountCodes` and `EVM::AccountStorages`");
		self.shell_state.insert_raw_key_map(codes).insert_raw_key_map(storages);

		self
	}
}

#[test]
fn pallet_ethereum_schema_should_work() {
	assert_eq!(array_bytes::bytes2hex("0x", b":ethereum_schema"), PALLET_ETHEREUM_SCHEMA);
}
