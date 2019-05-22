// this crate is only for practice and verify syntax and functions.

//DONE: test initial value in Store
//DONE: test Option value in Store
//TODO: test difference between dispatch::Result & rstd::result::Result
#![cfg_attr(not(feature = "std"), no_std)]
extern crate parity_codec;
extern crate parity_codec_derive;
extern crate sr_io as runtime_io;
extern crate sr_primitives as runtime_primitives;
extern crate sr_std as rstd;
extern crate srml_support as support;
#[macro_use]
extern crate srml_system as system;
#[cfg(test)]
extern crate substrate_primitives as primitives;

use rstd::{cmp, result};
use rstd::prelude::*;
use support::{decl_event, decl_module, decl_storage, StorageMap, StorageValue};
use support::dispatch::Result;
use system::ensure_signed;

pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
	    SomethingStored(u32, AccountId),
	}
);

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		SomeOption get(someoption): Option<u32>;
		Something get(something): u32;
		MapOption get(map_option): map u32 => Option<T::AccountId>;
		Map get(map): map u32 => T::AccountId;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event<T>() = default;

		pub fn do_something(origin, something: u32) -> Result {
			let who = ensure_signed(origin)?;

			<Something<T>>::put(something);
			<SomeOption<T>>::put(something);

			// here we are raising the Something event
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}

		pub fn do_map(origin, uint: u32) -> Result {
		    let who = ensure_signed(origin)?;

		    <MapOption<T>>::insert(uint, who.clone());
		    <Map<T>>::insert(uint, who.clone());

            Ok(())
		}
	}
}


#[cfg(test)]
mod tests {
    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        BuildStorage,
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
    };
    use support::{assert_ok, impl_outer_origin};

    use super::*;

    impl_outer_origin! {
		pub enum Origin for Test {}
	}

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;

    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }

    impl Trait for Test {
        type Event = ();
    }

    type Try = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {

            assert_eq!(Try::something(), 0);
            assert_eq!(Try::someoption(), None);

            assert_ok!(Try::do_something(Origin::signed(1), 42));

            assert_eq!(Try::something(), 42);
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
}




