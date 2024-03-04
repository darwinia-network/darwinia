// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

pub use crate::{self as darwinia_ecdsa_authority};
pub use dc_primitives::AccountId;

// darwinia
use darwinia_ecdsa_authority::{primitives::*, *};
// substrate
use frame_support::traits::OnInitialize;
use sp_io::TestExternalities;
use sp_runtime::BuildStorage;

pub type BlockNumber = u64;

frame_support::parameter_types! {
	pub Version: sp_version::RuntimeVersion = sp_version::RuntimeVersion {
		spec_name: sp_runtime::RuntimeString::Owned("Darwinia".into()),
		..Default::default()
	};
}
impl frame_system::Config for Runtime {
	type AccountData = ();
	type AccountId = AccountId;
	type BaseCallFilter = frame_support::traits::Everything;
	type Block = frame_system::mocking::MockBlock<Self>;
	type BlockHashCount = ();
	type BlockLength = ();
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type Nonce = u64;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = Version;
}

frame_support::parameter_types! {
	pub const SignThreshold: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(60);
	pub static MessageRoot: Option<darwinia_ecdsa_authority::primitives::Hash> = Some(Default::default());
}
impl Config for Runtime {
	type ChainId = frame_support::traits::ConstU64<46>;
	type MaxAuthorities = frame_support::traits::ConstU32<3>;
	type MaxPendingPeriod = frame_support::traits::ConstU64<20>;
	type MessageRoot = MessageRoot;
	type RuntimeEvent = RuntimeEvent;
	type SignThreshold = SignThreshold;
	type SyncInterval = frame_support::traits::ConstU64<10>;
	type WeightInfo = ();
}

frame_support::construct_runtime! {
	pub enum Runtime {
		System: frame_system,
		EcdsaAuthority: darwinia_ecdsa_authority,
	}
}

#[derive(Default)]
pub(crate) struct ExtBuilder {
	authorities: Vec<AccountId>,
}
impl ExtBuilder {
	pub(crate) fn authorities(mut self, authorities: Vec<AccountId>) -> Self {
		self.authorities = authorities;

		self
	}

	pub(crate) fn build(self) -> TestExternalities {
		let Self { authorities } = self;
		let mut storage =
			<frame_system::GenesisConfig<Runtime>>::default().build_storage().unwrap();

		darwinia_ecdsa_authority::GenesisConfig::<Runtime> { authorities }
			.assimilate_storage(&mut storage)
			.unwrap();

		let mut ext = TestExternalities::from(storage);

		ext.execute_with(|| {
			System::set_block_number(1);
			<EcdsaAuthority as OnInitialize<_>>::on_initialize(1);
		});

		ext
	}
}

pub(crate) fn account_id_of(id: u8) -> AccountId {
	Address::repeat_byte(id).0.into()
}

pub(crate) fn message_root_of(byte: u8) -> Hash {
	Hash::repeat_byte(byte)
}
pub(crate) fn new_message_root(byte: u8) -> Hash {
	let message_root = message_root_of(byte);

	MESSAGE_ROOT.with(|v| *v.borrow_mut() = Some(message_root));

	message_root
}

pub(crate) fn run_to_block_with<F>(n: BlockNumber, f: F)
where
	F: Fn(),
{
	(System::block_number() + 1..=n).for_each(|b| {
		System::set_block_number(b);
		<EcdsaAuthority as OnInitialize<_>>::on_initialize(b);

		f();
	});
}
pub(crate) fn run_to_block(n: BlockNumber) {
	run_to_block_with(n, || {});
}

pub(crate) fn ecdsa_authority_events() -> Vec<Event<Runtime>> {
	fn events() -> Vec<RuntimeEvent> {
		let events = System::events().into_iter().map(|evt| evt.event).collect::<Vec<_>>();

		System::reset_events();

		events
	}

	events()
		.into_iter()
		.filter_map(|e| match e {
			RuntimeEvent::EcdsaAuthority(e) => Some(e),
			_ => None,
		})
		.collect::<Vec<_>>()
}
