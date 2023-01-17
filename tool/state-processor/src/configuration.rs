pub const GWEI: u128 = 1_000_000_000;
pub const KTON_ID: u64 = 1026;
// https://github.dev/darwinia-network/darwinia-2.0/blob/c9fdfa170501648102bd0137c0437e367e743770/runtime/common/src/gov_origin.rs#L46
pub const ROOT: [u8; 20] = [0x72, 0x6f, 0x6f, 0x74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub trait Configurable {
	const NAME: &'static str;
}
impl Configurable for () {
	const NAME: &'static str = "";
}

pub struct Darwinia;
impl Configurable for Darwinia {
	const NAME: &'static str = "darwinia";
}

pub struct Crab;
impl Configurable for Crab {
	const NAME: &'static str = "crab";
}

pub struct Pangolin;
impl Configurable for Pangolin {
	const NAME: &'static str = "pangolin";
}
