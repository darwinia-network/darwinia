// darwinia
use crate::*;

impl Processor {
	pub fn process_vesting(&mut self) -> &mut Self {
		// Storage items.
		// https://github.dev/darwinia-network/substrate/blob/darwinia-v0.12.5/frame/vesting/src/lib.rs#L188
		let mut vestings = Map::default();

		// TODO: adjust decimals
		// TODO: adjust block number
		log::info!("take solo `Vesting::Vesting`");
		self.solo_state.take_raw_map(
			&item_key(b"Vesting", b"Vesting"),
			&mut vestings,
			|key, from| replace_first_match(key, from, &item_key(b"AccountMigration", b"Vestings")),
		);

		log::info!("set `Vesting::Vesting`");
		self.shell_state.insert_raw_key_map(vestings);

		self
	}
}
