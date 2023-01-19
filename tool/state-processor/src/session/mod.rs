// darwinia
use crate::*;

impl<S> Processor<S> {
	pub fn process_session(&mut self) -> (Set, Set) {
		// Session storage items.
		// https://github.dev/darwinia-network/substrate/blob/darwinia-v0.12.5/frame/session/src/lib.rs#L532

		let prefix = item_key(b"Session", b"NextKeys");

		let mut s_keys = Set::default();
		let mut p_keys = Set::default();

		self.solo_state.take_keys(&prefix, &mut s_keys, |k, _| {
			blake2_128_concat_to_string(key_to_account32(k))
		});
		self.para_state.take_keys(&prefix, &mut p_keys, |k, _| {
			blake2_128_concat_to_string(key_to_account32(k))
		});

		(s_keys, p_keys)
	}
}
