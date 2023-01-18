pub const GWEI: u128 = 1_000_000_000;
pub const KTON_ID: u64 = 1026;
// https://github.dev/darwinia-network/darwinia-2.0/blob/c9fdfa170501648102bd0137c0437e367e743770/runtime/common/src/gov_origin.rs#L46
pub const ROOT: [u8; 20] = [0x72, 0x6f, 0x6f, 0x74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub trait Configurable {
	const NAME: &'static str;
	// This account's balance will be burned.
	// Please make sure no one transfer balance to this account.
	const PARACHAIN_BACKING: &'static str;
}
impl Configurable for () {
	const NAME: &'static str = "";
	const PARACHAIN_BACKING: &'static str = "";
}

pub struct Darwinia;
impl Configurable for Darwinia {
	const NAME: &'static str = "darwinia";
	const PARACHAIN_BACKING: &'static str =
		"0x1000000000000000000000000000000000000000000000000000000000000000";
}

pub struct Crab;
impl Configurable for Crab {
	const NAME: &'static str = "crab";
	const PARACHAIN_BACKING: &'static str =
		"0x64766d3a0000000000000035a314e53e2fddfeca7b743042aacfb1abaf0adea3";
}

pub struct Pangolin;
impl Configurable for Pangolin {
	const NAME: &'static str = "pangolin";
	const PARACHAIN_BACKING: &'static str =
		"0x64766d3a000000000000008c585f9791ee5b4b23fe82888ce576dbb69607ebe9";
}
