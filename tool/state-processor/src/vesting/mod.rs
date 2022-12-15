// darwinia
use crate::*;

impl Processor {
	pub fn process_vesting(&mut self) -> &mut Self {
		// Storage items.
		// https://github.dev/paritytech/substrate/blob/19162e43be45817b44c7d48e50d03f074f60fbf4/frame/vesting/src/lib.rs#L188-L196
		let mut vestings = Map::default();

		log::info!("take solo `Vesting::Vesting`");
		self.solo_state.take_raw(&item_key(b"Vesting", b"Vesting"), &mut vestings, |key, from| {
			replace_first_match(key, from, &item_key(b"AccountMigration", b"Vestings"))
		});
		log::info!("set `Vesting::Vesting`");
		self.shell_state.insert_raw(vestings);

		self
	}
}
