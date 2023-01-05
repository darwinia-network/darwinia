// darwinia
use crate::*;

impl<S> Processor<S> {
	pub fn process_sudo(&mut self) -> &mut Self {
		// Storage items.
		// https://github.dev/darwinia-network/substrate/blob/darwinia-v0.12.5/frame/sudo/src/lib.rs#L268
		//
		// The new sudo key will be set on the genesis side.
		// We just need to kill the old keys.
		log::info!("drain solo and para `Sudo::Key`");
		self.solo_state.take_value(b"Sudo", b"Key", "", &mut [0_u8; 32]);
		self.para_state.take_value(b"Sudo", b"Key", "", &mut [0_u8; 32]);

		self
	}
}
