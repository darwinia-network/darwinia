/// sample runtime module implementing the ERC20 interface

use rstd::prelude::*;
use parity_codec::Codec;
use support::{dispatch::Result, StorageMap, Parameter, StorageValue, decl_storage, decl_module, decl_event, ensure};
use system::{self, ensure_signed};
use runtime_primitives::traits::{CheckedSub, CheckedAdd, Member, SimpleArithmetic, As};
use rstd::{result, ops::{Mul, Div}, cmp};

// the module trait
// contains type definitions
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64>;
}

// public interface for this runtime module
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
      // initialize the default event for this module
      fn deposit_event<T>() = default;

      // initialize the token
      // transfers the total_supply amout to the caller
      // the token becomes usable
      // not part of ERC20 standard interface
      // replicates the ERC20 smart contract constructor functionality
      fn init(origin) -> Result {
          let sender = ensure_signed(origin)?;
          ensure!(Self::is_init() == false, "Already initialized.");
          ensure!(Self::owner() == sender, "Only owner can initialize.");

          <BalanceOf<T>>::insert(sender.clone(), Self::total_supply());
          <Init<T>>::put(true);

          Ok(())
      }

      // transfer tokens from one account to another
      fn transfer(_origin, to: T::AccountId, value: T::TokenBalance) -> Result {
          let sender = ensure_signed(_origin)?;
          Self::_transfer(sender, to, value)
      }

      // approve token transfer from one account to another
      // once this is done, then transfer_from can be called with corresponding values
      fn approve(origin, spender: T::AccountId, value: T::TokenBalance) -> Result {
          let sender = ensure_signed(origin)?;
          // make sure the approver/owner owns this token
          ensure!(<BalanceOf<T>>::exists(&sender), "Account does not own this token");

          // get the current value of the allowance for this sender and spender combination
          // if doesnt exist then default 0 will be returned
          let allowance = Self::allowance((sender.clone(), spender.clone()));
          
          // add the value to the current allowance
          // using checked_add (safe math) to avoid overflow
          let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;

          // insert the new allownace value of this sender and spender combination
          <Allowance<T>>::insert((sender.clone(), spender.clone()), updated_allowance);
          
          // raise the approval event
          Self::deposit_event(RawEvent::Approval(sender, spender, value));
          Ok(())
      }

      // if approved, transfer from an account to another account without needing owner's signature
      fn transfer_from(_origin, from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> Result {
          ensure!(<Allowance<T>>::exists((from.clone(), to.clone())), "Allowance does not exist.");
          let allowance = Self::allowance((from.clone(), to.clone()));
          ensure!(allowance >= value, "Not enough allowance.");

          // using checked_sub (safe math) to avoid overflow
          let updated_allowance = allowance.checked_sub(&value).ok_or("overflow in calculating allowance")?;
          // insert the new allownace value of this sender and spender combination
          <Allowance<T>>::insert((from.clone(), to.clone()), updated_allowance);

          Self::deposit_event(RawEvent::Approval(from.clone(), to.clone(), value));
          Self::_transfer(from, to, value)
      }
  }
}

// storage for this runtime module
decl_storage! {
  trait Store for Module<T: Trait> as SDR {
    // bool flag to allow init to be called only once
    Init get(is_init): bool;

    // owner gets all the tokens when calls initialize
    // setting via genesis config to avoid race condition
    Owner get(owner) config(): T::AccountId;

    // total supply of the token
    // set in the genesis config
    // see ../../src/chain_spec.rs - line 105
    TotalSupply get(total_supply) config(): T::TokenBalance;
    
    // not really needed - name and ticker, but why not?
    Name get(name) config(): Vec<u8>;
    Ticker get (ticker) config(): Vec<u8>;

    // standard balances and allowances mappings for ERC20 implementation
    BalanceOf get(balance_of): map T::AccountId => T::TokenBalance;
    Allowance get(allowance): map (T::AccountId, T::AccountId) => T::TokenBalance;
  }
}

// events
decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, 
      Balance = <T as self::Trait>::TokenBalance {
        // event for transfer of tokens
        // from, to, value
        Transfer(AccountId, AccountId, Balance),
        // event when an approval is made
        // owner, spender, value
        Approval(AccountId, AccountId, Balance),
    }
);

// module implementation block
// utility and private functions
// if marked public, accessible by other modules
impl<T: Trait> Module<T> {
    // internal transfer function for ERC20 interface
     pub fn _transfer(
        from: T::AccountId,
        to: T::AccountId,
        value: T::TokenBalance,
    ) -> Result {
        ensure!(<BalanceOf<T>>::exists(from.clone()), "Account does not own this token");
        let sender_balance = Self::balance_of(from.clone());
        ensure!(sender_balance >= value, "Not enough balance.");

        let updated_from_balance = sender_balance.checked_sub(&value).ok_or("overflow in calculating balance")?;
        let receiver_balance = Self::balance_of(to.clone());
        let updated_to_balance = receiver_balance.checked_add(&value).ok_or("overflow in calculating balance")?;
        
        // reduce sender's balance
        <BalanceOf<T>>::insert(from.clone(), updated_from_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert(to.clone(), updated_to_balance);

        Self::deposit_event(RawEvent::Transfer(from, to, value));
        Ok(())
    }

    pub fn get_balance(who: T::AccountId) -> T::TokenBalance {
        Self::balance_of(who)
    }
}