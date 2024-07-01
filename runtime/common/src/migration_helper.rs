pub use frame_support::migration;

// darwinia
use dc_primitives::{AccountId, Balance};
// polkadot-sdk
use frame_support::traits::ReservableCurrency;
use frame_support_old::traits::ConstU32;
use pallet_identity::{Judgement, Registration};

/// Pallet migration helper.
pub struct PalletCleaner {
	pub name: &'static [u8],
	pub values: &'static [&'static [u8]],
	pub maps: &'static [&'static [u8]],
}
impl PalletCleaner {
	/// Remove all storage from a pallet.
	pub fn remove_all(&self) -> u64 {
		self.remove_storage_values() + self.remove_storage_maps()
	}

	/// Remove multiple storage value from a pallet at once.
	pub fn remove_storage_values(&self) -> u64 {
		self.values.iter().for_each(|i| {
			let _ = migration::clear_storage_prefix(self.name, i, &[], None, None);
		});

		self.values.len() as u64
	}

	/// Remove multiple storage map from a pallet at once.
	pub fn remove_storage_maps(&self) -> u64 {
		self.maps.iter().fold(0, |acc, i| {
			acc + migration::clear_storage_prefix(self.name, i, &[], None, None).backend as u64
		})
	}
}

pub fn migrate_identity_of<C>() -> u64
where
	C: ReservableCurrency<AccountId, Balance = Balance>,
{
	migration::storage_iter_with_suffix::<Registration<Balance, ConstU32<20>, ConstU32<100>>>(
		b"Identity",
		b"IdentityOf",
		&[],
	)
	.drain()
	.fold(0, |acc, (k, v)| {
		if k.len() > 20 {
			let mut who = [0u8; 20];

			who.copy_from_slice(&k[k.len() - 20..]);

			let who = AccountId::from(who);
			let deposit = v.deposit
				+ v.judgements
					.iter()
					.map(|(_, ref j)| if let Judgement::FeePaid(fee) = j { *fee } else { 0 })
					.sum::<Balance>();

			C::unreserve(&who, deposit);

			acc + 3
		} else {
			acc
		}
	})
}
