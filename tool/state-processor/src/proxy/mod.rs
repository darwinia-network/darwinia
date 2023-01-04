// darwinia
use crate::*;

impl<S> Processor<S> {
	pub fn process_proxy(&mut self) -> &mut Self {
		// Storage items.
		// https://github.dev/darwinia-network/substrate/blob/darwinia-v0.12.5/frame/proxy/src/lib.rs#L599
		// Skip the `Announcements`.
		// Need to make sure the storage is empty.
		//
		// The size of encoded `pallet_proxy::ProxyDefinition` is 37 bytes.
		let mut proxies = <Map<(Vec<[u8; 37]>, u128)>>::default();

		log::info!("take solo `Proxy::Proxies`");
		self.solo_state.take_map(b"Proxy", b"Proxies", &mut proxies, get_identity_key);

		// https://github.dev/darwinia-network/substrate/blob/darwinia-v0.12.5/frame/proxy/src/lib.rs#L735
		log::info!("adjust the reserved's decimals then free solo `Proxies` reservations");
		proxies.into_iter().for_each(|(a, (_, mut v))| {
			v.adjust();
			self.shell_state.unreserve(array_bytes::hex2bytes_unchecked(get_last_64(&a)), v);
		});

		// Skip the parachain part.
		// Need to make sure the storage is empty.

		self
	}
}
