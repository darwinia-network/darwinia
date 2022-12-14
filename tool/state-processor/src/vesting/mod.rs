// darwinia
use crate::*;

impl Processor {
	pub fn process_vesting(&mut self) -> &mut Self {
		let mut vestings = Map::default();

		self.solo_state.take_raw(&item_key(b"Vesting", b"Vesting"), &mut vestings, |key, from| {
			replace_first_match(key, from, &item_key(b"AccountMigration", b"Vestings"))
		});
		self.shell_state.insert_raw(vestings);

		self
	}
}
