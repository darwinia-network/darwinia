pub const GWEI: u128 = 1_000_000_000;
pub const KTON_ID: u64 = 1026;
// https://github.dev/darwinia-network/darwinia-2.0/blob/c9fdfa170501648102bd0137c0437e367e743770/runtime/common/src/gov_origin.rs#L46
pub const ROOT: [u8; 20] = [0x72, 0x6f, 0x6f, 0x74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub trait Configurable {
	const NAME: &'static str;

	const KTON_NAME: &'static [u8];
	const KTON_SYMBOL: &'static [u8];
}
impl Configurable for () {
	const KTON_NAME: &'static [u8] = b"";
	const KTON_SYMBOL: &'static [u8] = b"";
	const NAME: &'static str = "";
}

pub struct Darwinia;
impl Configurable for Darwinia {
	const KTON_NAME: &'static [u8] = b"Darwinia Commitment Token";
	const KTON_SYMBOL: &'static [u8] = b"KTON";
	const NAME: &'static str = "darwinia";
}

pub struct Crab;
impl Configurable for Crab {
	const KTON_NAME: &'static [u8] = b"Crab Commitment Token";
	const KTON_SYMBOL: &'static [u8] = b"CKTON";
	const NAME: &'static str = "crab";
}

pub struct Pangolin;
impl Configurable for Pangolin {
	const KTON_NAME: &'static [u8] = b"Pangolin Commitment Token";
	const KTON_SYMBOL: &'static [u8] = b"PKTON";
	const NAME: &'static str = "pangolin";
}
