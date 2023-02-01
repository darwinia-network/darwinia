// darwinia
use crate::*;

impl<S> Processor<S>
where
	S: Configurable,
{
	/// Only care about the solo chain, since parachains don't have identity now.
	pub fn process_identity(&mut self) -> &mut Self {
		let mut identities = <Map<Registration>>::default();
		let mut registrars = Vec::<Option<RegistrarInfo<AccountId32>>>::default();

		log::info!("take `Identity::IdentityOf`, `Identity::Registrars`, `Identity::SubsOf`");
		self.solo_state
			.take_map(b"Identity", b"IdentityOf", &mut identities, get_hashed_key)
			.take_value(b"Identity", b"Registrars", "", &mut registrars);

		log::info!("adjust registrations and set `AccountMigration::Identities`");
		identities.into_iter().for_each(|(k, mut v)| {
			v.adjust();

			let a = get_last_64(&k);
			// Calculate the identity reservation.
			//
			// https://github.com/paritytech/substrate/blob/129fee774a6d185d117a57fd1e81b3d0d05ad747/frame/identity/src/lib.rs#L364
			let r = S::basic_deposit() + v.info.additional.len() as Balance * S::field_deposit();
			// Calculate the judgement reservation.
			//
			// https://github.com/paritytech/substrate/blob/129fee774a6d185d117a57fd1e81b3d0d05ad747/frame/identity/src/lib.rs#L564
			let rj = v.judgements.iter().fold(0, |acc, (i, _)| {
				registrars
					.get(*i as usize)
					.and_then(|r| r.as_ref().map(|r| acc + r.fee))
					.unwrap_or_else(|| {
						log::error!("failed to find a registrar for `Account({a})`");

						acc
					})
			});

			self.shell_state.reserve(a, r + rj);
			self.shell_state.insert_value(b"AccountMigration", b"Identities", &k, v);
		});

		log::info!("truncate registrar account id and adjust registrars fee decimal");
		let registrars = registrars
			.into_iter()
			.map(|o| {
				if let Some(mut r) = o {
					r.adjust();

					let mut account = [0_u8; 20];

					account.copy_from_slice(&r.account[..20]);

					Some(RegistrarInfo { account, fee: r.fee, fields: r.fields })
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		log::info!("set `Identity::Registrars`");
		self.shell_state.insert_value(b"Identity", b"Registrars", "", registrars);

		self
	}
}
