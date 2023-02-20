// darwinia
use crate::*;

pub const KTON_ID: u64 = 1026;
// https://github.dev/darwinia-network/darwinia-2.0/blob/c9fdfa170501648102bd0137c0437e367e743770/runtime/common/src/gov_origin.rs#L46
pub const ROOT: [u8; 20] = [0x72, 0x6f, 0x6f, 0x74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub trait Configurable {
	const NAME: &'static str;
	// This account's balance will be burned.
	// Please make sure no one transfer balance to this account.
	const PARACHAIN_BACKING: &'static str;

	// Make sure these account doesn't exist in the old chains.
	// To prevent their data get overridden.
	fn genesis_collators() -> Vec<AccountId20> {
		vec![
			array_bytes::hex2array_unchecked("0x0eef9fabb6eb6fed2ab24a842931f8950426070a"),
			array_bytes::hex2array_unchecked("0xa858cde8f6cf178786578a3b0becf5c27d18300c"),
			array_bytes::hex2array_unchecked("0x986b41d07776aa48f6d7a80caad49485f9a71714"),
		]
	}

	// https://github.com/paritytech/substrate/blob/129fee774a6d185d117a57fd1e81b3d0d05ad747/frame/identity/src/lib.rs#L113
	fn basic_deposit() -> Balance;

	// https://github.com/paritytech/substrate/blob/129fee774a6d185d117a57fd1e81b3d0d05ad747/frame/identity/src/lib.rs#L117
	fn field_deposit() -> Balance;
}
impl Configurable for () {
	const NAME: &'static str = "";
	const PARACHAIN_BACKING: &'static str = "";

	fn basic_deposit() -> Balance {
		0
	}

	fn field_deposit() -> Balance {
		0
	}
}

pub struct Darwinia;
impl Configurable for Darwinia {
	const NAME: &'static str = "darwinia";
	const PARACHAIN_BACKING: &'static str =
		"0x1000000000000000000000000000000000000000000000000000000000000000";

	fn basic_deposit() -> Balance {
		100 * UNIT + 258 * 100 * MICROUNIT
	}

	fn field_deposit() -> Balance {
		66 * 100 * MICROUNIT
	}
}

pub struct Crab;
impl Configurable for Crab {
	const NAME: &'static str = "crab";
	const PARACHAIN_BACKING: &'static str =
		"0x64766d3a0000000000000035a314e53e2fddfeca7b743042aacfb1abaf0adea3";

	fn basic_deposit() -> Balance {
		100 * UNIT + 258 * 100 * MICROUNIT
	}

	fn field_deposit() -> Balance {
		66 * 100 * MICROUNIT
	}
}

pub struct Pangolin;
impl Configurable for Pangolin {
	const NAME: &'static str = "pangolin";
	const PARACHAIN_BACKING: &'static str =
		"0x64766d3a000000000000008c585f9791ee5b4b23fe82888ce576dbb69607ebe9";

	fn basic_deposit() -> Balance {
		100 * UNIT + 258 * 100 * MICROUNIT
	}

	fn field_deposit() -> Balance {
		66 * 100 * MICROUNIT
	}
}

pub struct Pangoro;
impl Configurable for Pangoro {
	const NAME: &'static str = "pangoro";
	const PARACHAIN_BACKING: &'static str =
		"0x64766d3a000000000000008c585f9791ee5b4b23fe82888ce576dbb69607ebe9";

	fn basic_deposit() -> Balance {
		100 * UNIT + 258 * 100 * MICROUNIT
	}

	fn field_deposit() -> Balance {
		66 * 100 * MICROUNIT
	}
}
