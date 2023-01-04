// darwinia
use crate::*;

impl<S> Processor<S> {
	pub fn process_indices(&mut self) -> &mut Self {
		// Storage items.
		// https://github.dev/darwinia-network/substrate/blob/darwinia-v0.12.5/frame/indices/src/lib.rs#L291
		let mut accounts = <Map<([u8; 32], u128, bool)>>::default();

		log::info!("take solo `Indices::Accounts`");
		self.solo_state.take_map(b"Indices", b"Accounts", &mut accounts, get_identity_key);

		// https://github.dev/darwinia-network/substrate/blob/darwinia-v0.12.5/frame/indices/src/lib.rs#L154
		log::info!("adjust the reserved's decimals then free solo `Indices` reservations");
		accounts.into_iter().for_each(|(_, (a, mut v, _))| {
			v.adjust();
			self.shell_state.unreserve(a, v);
		});

		self
	}
}
