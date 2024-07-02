pub use frame_support::migration;

// crates.io
use codec::{Decode, Encode};
// darwinia
use dc_primitives::{AccountId, Balance};
// polkadot-sdk
use frame_support::traits::ReservableCurrency;
use sp_runtime::traits::AppendZerosInput;
use sp_std::prelude::*;

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
	migration::storage_iter_with_suffix::<Registration>(b"Identity", b"IdentityOf", &[])
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

#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Encode)]
struct Registration {
	judgements: Vec<(u32, Judgement)>,
	deposit: Balance,
	info: IdentityInfo,
}
impl Decode for Registration {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let (judgements, deposit, info) = Decode::decode(&mut AppendZerosInput::new(input))?;
		Ok(Self { judgements, deposit, info })
	}
}

#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Encode, Decode)]
enum Judgement {
	Unknown,
	FeePaid(Balance),
	Reasonable,
	KnownGood,
	OutOfDate,
	LowQuality,
	Erroneous,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Encode, Decode)]
struct IdentityInfo {
	additional: Vec<(Data, Data)>,
	display: Data,
	legal: Data,
	web: Data,
	riot: Data,
	email: Data,
	pgp_fingerprint: Option<[u8; 20]>,
	image: Data,
	twitter: Data,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
enum Data {
	None,
	Raw(Vec<u8>),
	BlakeTwo256([u8; 32]),
	Sha256([u8; 32]),
	Keccak256([u8; 32]),
	ShaThree256([u8; 32]),
}
impl Decode for Data {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let b = input.read_byte()?;
		Ok(match b {
			0 => Data::None,
			n @ 1..=33 => {
				let mut r = vec![0u8; n as usize - 1];
				input.read(&mut r[..])?;
				Data::Raw(r)
			},
			34 => Data::BlakeTwo256(<[u8; 32]>::decode(input)?),
			35 => Data::Sha256(<[u8; 32]>::decode(input)?),
			36 => Data::Keccak256(<[u8; 32]>::decode(input)?),
			37 => Data::ShaThree256(<[u8; 32]>::decode(input)?),
			_ => return Err(codec::Error::from("invalid leading byte")),
		})
	}
}
impl Encode for Data {
	fn encode(&self) -> Vec<u8> {
		match self {
			Data::None => vec![0u8; 1],
			Data::Raw(ref x) => {
				let l = x.len().min(32);
				let mut r = vec![l as u8 + 1; l + 1];
				r[1..].copy_from_slice(&x[..l]);
				r
			},
			Data::BlakeTwo256(ref h) => core::iter::once(34u8).chain(h.iter().cloned()).collect(),
			Data::Sha256(ref h) => core::iter::once(35u8).chain(h.iter().cloned()).collect(),
			Data::Keccak256(ref h) => core::iter::once(36u8).chain(h.iter().cloned()).collect(),
			Data::ShaThree256(ref h) => core::iter::once(37u8).chain(h.iter().cloned()).collect(),
		}
	}
}

#[test]
fn identity_codec_should_work() {
	let chain_raw_data = "0x040100000003008044fe2307236c0500000000000000000f41757265766f69725861766965720b586176696572204c61752168747470733a2f2f6c696e6b74722e65652f61757265766f69727861766965721b4061757265766f69727861766965723a6d61747269782e6f72671078617669657240696e762e636166650000104041757265766f6972586176696572";
	let encoded = array_bytes::hex2bytes_unchecked(chain_raw_data);
	let decoded = Registration::decode(&mut &encoded[..]).unwrap();
	let expected = Registration {
		judgements: vec![(1, Judgement::KnownGood)],
		deposit: 100_025_800_000_000_000_000,
		info: IdentityInfo {
			additional: vec![],
			display: Data::Raw(b"AurevoirXavier".to_vec()),
			legal: Data::Raw(b"Xavier Lau".to_vec()),
			web: Data::Raw(b"https://linktr.ee/aurevoirxavier".to_vec()),
			riot: Data::Raw(b"@aurevoirxavier:matrix.org".to_vec()),
			email: Data::Raw(b"xavier@inv.cafe".to_vec()),
			pgp_fingerprint: None,
			image: Data::None,
			twitter: Data::Raw(b"@AurevoirXavier".to_vec()),
		},
	};

	assert_eq!(decoded, expected);
	assert_eq!(encoded, expected.encode())
}
