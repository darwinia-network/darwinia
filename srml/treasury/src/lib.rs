

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use rstd::prelude::*;
use srml_support::{StorageValue, StorageMap, decl_module, decl_storage, decl_event, ensure};
use srml_support::traits::{Currency, ReservableCurrency, OnDilution, OnUnbalanced, Imbalance};
use primitives::{Permill, traits::{Zero, EnsureOrigin, StaticLookup}};
use parity_codec::{Encode, Decode};
use system::ensure_signed;
use evo_support::traits::DarwiniaDilution;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type PositiveImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: timestamp::Trait {

    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Treasury {
		Pot get(pot): BalanceOf<T>;
    }
}

decl_event! {
    pub enum Event<T>
    where
        Balance = BalanceOf<T>
    {
        IntoPot(Balance),
    }
}

//impl<T: Trait> OnDilution<BalanceOf<T>> for Module<T> {
//    fn on_dilution(minted: BalanceOf<T>, portion: BalanceOf<T>) {
//        // Mint extra funds for the treasury to keep the ratio of portion to total_issuance equal
//        // pre dilution and post-dilution.
//        if !minted.is_zero() && !portion.is_zero() {
//            let total_issuance = T::Currency::total_issuance();
//            let funding = (total_issuance - portion) / portion * minted;
//            <Pot<T>>::mutate(|x| *x += funding);
//        }
//    }
//}

impl<T: Trait> DarwiniaDilution<BalanceOf<T>> for Module<T> {
    fn on_dilution(treasury_income: BalanceOf<T>) {

        if !treasury_income.is_zero() {
            <Pot<T>>::mutate(|x| *x += treasury_income);
        }
    }
}