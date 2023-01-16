// darwinia
use crate::*;

impl<S> Processor<S> {
	/// Only care about the solo chain, since parachains don't have identity now.
	pub fn process_identity(&mut self) -> &mut Self {
		let mut identities = <Map<Registration>>::default();
		let mut registrars = Vec::<Option<RegistrarInfo>>::default();
		let mut subs_of = Map::<(u128, Vec<[u8; 32]>)>::default();

		log::info!("take `Identity::IdentityOf`, `Identity::Registrars`, `Identity::SuperOf` and `Identity::SuperOf`");
		self.solo_state
			.take_map(b"Identity", b"IdentityOf", &mut identities, get_hashed_key)
			.take_value(b"Identity", b"Registrars", "", &mut registrars)
			.take_map(b"Identity", b"SubsOf", &mut subs_of, get_last_64_key);

		log::info!("update identities's deposit and judgement decimal");
		identities.iter_mut().for_each(|(_k, v)| {
			v.adjust();
		});

		log::info!("update registrars fee decimal");
		registrars.iter_mut().for_each(|o| {
			if let Some(r) = o {
				r.adjust()
			}
		});

		log::info!("update super_id's reserved balance");
		subs_of.into_iter().for_each(|(super_id, (mut subs_deposit, _))| {
			subs_deposit.adjust();

			self.shell_state
				.unreserve(array_bytes::hex2array_unchecked::<_, 32>(super_id), subs_deposit);
		});

		log::info!("set `AccountMigration::IdentityOf`");
		{
			let ik = item_key(b"AccountMigration", b"IdentityOf");

			self.shell_state.insert_map(identities, |h| format!("{ik}{h}"));
		}

		log::info!("set `AccountMigration::Registrars`");
		self.shell_state.insert_value(b"AccountMigration", b"Registrars", "", registrars);

		self
	}
}
