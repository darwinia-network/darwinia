
// minimum contract
// only test functions that interact with token system
// including `buy_gas` and `refund_gas`
#![cfg_attr(not(feature = "std"), no_std)]

extern crate srml_system as system;
extern crate srml_timestamp as timestamp;
extern crate sr_primitives as runtime_primitives;
extern crate sr_std as rstd;

use parity_codec::{Codec, Encode, Decode};
use runtime_primitives::traits::{Hash, As, SimpleArithmetic, Bounded, StaticLookup, Zero, CheckedMul, CheckedSub};
use srml_support::dispatch::{Result, Dispatchable};
use srml_support::{Parameter, StorageMap, StorageValue, decl_module, decl_event, decl_storage, traits::Currency};
use evo_support::traits::SystemCurrency;
use system::{ensure_signed, RawOrigin};
use rstd::result;

#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub enum GasMeterResult {
    Proceed,
    OutOfGas,
}

impl GasMeterResult {
    pub fn is_out_of_gas(&self) -> bool {
        match *self {
            GasMeterResult::OutOfGas => true,
            GasMeterResult::Proceed => false,
        }
    }
}




pub struct GasMeter<T: Trait> {
    limit: T::Gas,
    /// Amount of gas left from initial gas limit. Can reach zero.
    gas_left: T::Gas,
    gas_price: BalanceOf<T>,
}

impl<T: Trait> GasMeter<T> {
    pub fn gas_left(&self) -> T::Gas {
        self.gas_left
    }
    fn spent(&self) -> T::Gas {
        self.limit - self.gas_left
    }
    fn spend_gas(&mut self, gas_consumed: T::Gas) -> GasMeterResult {
        let new_value = match self.gas_left.checked_sub(&gas_consumed) {
            None => None,
            Some(val) if val.is_zero() => None,
            Some(val) => Some(val),
        };

        self.gas_left = new_value.unwrap_or_else(Zero::zero);

        match new_value {
            Some(_) => GasMeterResult::Proceed,
            None => GasMeterResult::OutOfGas,
        }
    }
}



pub type BalanceOf<T> = <<T as Trait>::SystemCurrency as SystemCurrency<<T as system::Trait>::AccountId>>::CurrencyOf;
pub type NegativeImbalanceOf<T> = <<T as Trait>::SystemCurrency as SystemCurrency<<T as system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: timestamp::Trait {

    type SystemCurrency: SystemCurrency<Self::AccountId>;

    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Gas: Parameter + Default + Codec + SimpleArithmetic + Bounded + Copy + As<BalanceOf<Self>> + As<u64> + As<u32>;
}

decl_event! {
    pub enum Event <T>
    where
        Balance = BalanceOf < T >,
        < T as system::Trait >::AccountId
    {
        GasConsumed(AccountId, Balance),
        GasRefund(AccountId, Balance),
    }
}

decl_module! {
/// Contracts module.
    pub struct Module < T: Trait > for enum Call where origin: < T as system::Trait >::Origin {
        fn deposit_event < T > () = default;

        fn operate_with_contact(origin, #[compact] gas_limit: T::Gas, gas_consumed: T::Gas) -> Result {
            let origin = ensure_signed(origin)?;
            let (mut gas_meter, imbalance) = Self::buy_gas(&origin, gas_limit)?;

            if gas_meter
			.spend_gas(gas_consumed)
			.is_out_of_gas()
		{
			return Err("not enough gas to pay base call fee");
		}
		    Self::refund_unused_gas(&origin, gas_meter, imbalance);

             Ok(())
        }
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as ContractStorage {

		GasPrice get(gas_price) config(): BalanceOf<T> = BalanceOf::<T>::sa(1);

		BlockGasLimit get(block_gas_limit) config(): T::Gas = T::Gas::sa(10_000_000);

		GasSpent get(gas_spent): T::Gas;
    }
}

impl<T: Trait> Module<T> {

    // PUB MUTABLE
    pub fn buy_gas(
        transactor: &T::AccountId,
        gas_limit: T::Gas
    ) -> result::Result<(GasMeter<T>, NegativeImbalanceOf<T>), &'static str> {
        let gas_available = Self::block_gas_limit() - Self::gas_spent();
        if gas_limit > gas_available {
            return Err("block size limit is reached");
        }
        let gas_price = Self::gas_price();
        let cost = <T::Gas as As<BalanceOf<T>>>::as_(gas_limit.clone())
            .checked_mul(&gas_price)
            .ok_or("overflow multiplying gas limit by price")?;

        let imbalance = T::SystemCurrency::system_withdraw(
            transactor,
            cost
        )?;

        Ok((GasMeter {
            limit: gas_limit,
            gas_left: gas_limit,
            gas_price,
        }, imbalance))

    }

    pub fn refund_unused_gas (
        transactor: &T::AccountId,
        gas_meter: GasMeter<T>,
        imbalance: NegativeImbalanceOf<T>,
    ) {
        let gas_spent = gas_meter.spent();
        let gas_left = gas_meter.gas_left();

        <GasSpent<T>>::mutate(|block_gas_spent| *block_gas_spent += gas_spent);
        let refund = <T::Gas as As<BalanceOf<T>>>::as_(gas_left) * gas_meter.gas_price;
        T::SystemCurrency::system_refund(transactor, refund, imbalance);
    }
}


