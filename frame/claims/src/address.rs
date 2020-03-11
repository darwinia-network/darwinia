use codec::{Decode, Encode};
use rustc_hex::{FromHex, ToHex};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_runtime::RuntimeDebug;

pub type AddressT = [u8; 20];

macro_rules! impl_serde {
	($name:ident, $sname:expr, $prefix:expr) => {
		#[doc = "An "]
		#[doc = $sname]
		#[doc = " address (i.e. 20 bytes, used to represent an "]
		#[doc = $sname]
		#[doc = ".\n\nThis gets serialized to the "]
		#[doc = $prefix]
		#[doc = "-prefixed hex representation."]
		#[derive(Clone, Copy, Default, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
		pub struct $name(pub AddressT);

		#[cfg(feature = "std")]
		impl Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: Serializer,
			{
				let hex: String = ToHex::to_hex(&self.0[..]);
				serializer.serialize_str(&format!(concat!($prefix, "{}"), hex))
			}
		}

		#[cfg(feature = "std")]
		impl<'de> Deserialize<'de> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: Deserializer<'de>,
			{
				let base_string = String::deserialize(deserializer)?;
				let offset = if base_string.starts_with($prefix) { 2 } else { 0 };
				let s = &base_string[offset..];
				if s.len() != 40 {
					Err(serde::de::Error::custom(
						concat!("Bad length of ", $sname, " address (should be 42 including '", $prefix, "')"),
					))?;
				}
				let raw: Vec<u8> = FromHex::from_hex(s).map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?;
				let mut r = Self::default();
				r.0.copy_from_slice(&raw);
				Ok(r)
			}
		}
	};
}

impl_serde!(EthereumAddress, "Ethereum", "0x");
impl_serde!(TronAddress, "Tron", "41");
