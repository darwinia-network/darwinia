// darwinia
use crate::*;

impl<S> Processor<S> {
	/// Only care about the solo chain, since parachains don't have identity now.
	pub fn process_identity(&mut self) -> &mut Self {
		let mut identities = <Map<Registration>>::default();
		let mut registrars = Vec::<Option<RegistrarInfo<[u8; 32]>>>::default();
		let mut subs_of = Map::<(u128, Vec<[u8; 32]>)>::default();

		log::info!("take `Identity::IdentityOf`, `Identity::Registrars`, `Identity::SuperOf` and `Identity::SuperOf`");
		self.solo_state
			.take_map(b"Identity", b"IdentityOf", &mut identities, get_hashed_key)
			.take_value(b"Identity", b"Registrars", "", &mut registrars)
			.take_map(b"Identity", b"SubsOf", &mut subs_of, get_last_64_key);

		log::info!("free super_id's reservation");
		subs_of.into_iter().for_each(|(super_id, (mut subs_deposit, _))| {
			subs_deposit.adjust();

			self.shell_state
				.unreserve(array_bytes::hex2array_unchecked::<_, 32>(super_id), subs_deposit);
		});

		log::info!("adjust identities' deposit and judgement decimal");
		identities.iter_mut().for_each(|(_, v)| v.adjust());

		log::info!("set `AccountMigration::Identities`");
		{
			let ik = item_key(b"AccountMigration", b"Identities");

			self.shell_state.insert_map(identities, |h| format!("{ik}{h}"));
		}

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
