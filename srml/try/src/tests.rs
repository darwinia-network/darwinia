// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Test utilities

#![cfg(test)]

use super::*;
use sr_primitives::BuildStorage;
use sr_primitives::{traits::{IdentityLookup}, testing::Header};
use primitives::{H256, Blake2Hasher};
use sr_io::with_externalities;
use support::{ impl_outer_origin, assert_ok };
use crate::{GenesisConfig, Module, Trait};

impl_outer_origin!{
	pub enum Origin for Test {}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;
impl system::Trait for Test {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = ::sr_primitives::traits::BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
}




impl Trait for Test {
	type Event = ();
}


type System = system::Module<Test>;
type Try = Module<Test>;

fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
	let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
	t.extend(GenesisConfig::<Test> {
		_genesis_phantom_data: Default::default(),
		someoption: 42,
	}.build_storage().unwrap().0);
	t.into()

}

#[test]
fn it_works_for_default_value() {
	with_externalities(&mut new_test_ext(), || {

		assert_eq!(Try::something(), 0);
		assert_eq!(Try::someoption(), Some(42));
	});
}

#[test]
fn it_works_with_map() {
	with_externalities(&mut new_test_ext(), || {
		assert_ok!(Try::do_map(Origin::signed(1), 42));
		assert_eq!(Try::map_option(42), Some(1));
		assert_eq!(Try::map(42), 1);
	})
}

#[test]
fn check_default_value() {
	with_externalities(&mut new_test_ext(), || {
		assert_ok!(Try::do_map(Origin::signed(1), 42));
		assert_eq!(Try::map_option(40), None);
		assert_eq!(Try::map(40), 0);
	});
}

#[test]
fn check_delete() {
	with_externalities(&mut new_test_ext(), || {
		Try::update_list(1, true);
		assert_eq!(Try::list(1), vec![1]);
		Try::update_list(2, true);
		assert_eq!(Try::list(1), vec![1, 2]);
	});
}