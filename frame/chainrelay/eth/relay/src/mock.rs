//! Mock file for eth-relay.

use eth_primitives::{H256 as EH256, H64 as EH64};
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use hex_literal::hex;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};

use crate::*;

// --- substrate ---
pub type System = frame_system::Module<Test>;

// --- current ---
pub type EthRelay = Module<Test>;

type AccountId = u64;
type BlockNumber = u64;

impl_outer_origin! {
	pub enum Origin for Test {}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Trait for Test {
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type MigrateAccount = ();
}

parameter_types! {
//	pub const EthMainnet: u64 = 0;
	pub const EthRopsten: u64 = 1;
}
impl Trait for Test {
	type Event = ();
	type EthNetwork = EthRopsten;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();

	GenesisConfig::<Test> {
		number_of_blocks_finality: 30,
		number_of_blocks_safe: 10,
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t.into()
}

/// Mock a header from prev header_hash
pub fn mock_header_from(prev: &EthHeader, fake_nonce: i32) -> EthHeader {
	let epn = EthashPartial::production();
	let mut header = EthHeader::default();
	header.parent_hash = prev.hash.unwrap();
	header.number = prev.number + 1;
	header.difficulty = epn.calculate_difficulty(&header, &prev);

	// calculate mix_hash
	//
	// ```rust
	let light_dag = DAG::new(header.number.into());
	let mix_hash = light_dag.hashimoto(header.bare_hash(), EH64::from([0; 8])).0;
	println!("mix_hash: {:#?}", mix_hash);
	// ```
	//
	// result:
	// - 0: 0x3f517a9e5c0eece0b13aa323da099459c6817997f24ded564d98e2beef97e9f3
	// - 1: 0x880826e15c91b64f4da72e705757d08e1644ecf2535b45a43148699c6cb9e733
	match fake_nonce {
		0 => {
			header.seal = vec![
				rlp::encode(&EH256::from(hex!(
					"880826e15c91b64f4da72e705757d08e1644ecf2535b45a43148699c6cb9e733"
				))),
				rlp::encode(&EH64::from([1; 8])),
			];
		}
		_ => {
			header.seal = vec![
				rlp::encode(&EH256::from(hex!(
					"880826e15c91b64f4da72e705757d08e1644ecf2535b45a43148699c6cb9e733"
				))),
				rlp::encode(&EH64::from([1; 8])),
			];
		}
	}

	header.compute_hash();
	header
}
