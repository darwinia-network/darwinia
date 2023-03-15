// darwinia
use crate::*;

impl<S> Processor<S> {
	pub fn process_vesting(&mut self) -> &mut Self {
		// Storage items.
		// https://github.dev/darwinia-network/substrate/blob/darwinia-v0.12.5/frame/vesting/src/lib.rs#L188
		let mut vestings = <Map<Vec<VestingInfo>>>::default();

		log::info!("take solo `Vesting::Vesting`");
		self.solo_state.take_map(b"Vesting", b"Vesting", &mut vestings, get_hashed_key);

		log::info!("adjust and remove expired solo `VestingInfo`s");
		let vestings = vestings
			.into_iter()
			.filter_map(|(k, v)| {
				let v = v
					.into_iter()
					.filter_map(|mut v| {
						v.adjust();

						if v.locked == 0 {
							None
						} else {
							Some(v)
						}
					})
					.collect::<Vec<_>>();

				if v.is_empty() {
					None
				} else {
					Some((k, v))
				}
			})
			.collect::<Map<Vec<VestingInfo>>>();

		log::info!("set `AccountMigration::Vestings`");
		{
			let ik = item_key(b"AccountMigration", b"Vestings");

			self.shell_state.insert_map(vestings, |h| format!("{ik}{h}"));
		}

		self
	}
}
