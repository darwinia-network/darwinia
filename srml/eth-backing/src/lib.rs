#![feature(in_band_lifetimes)]
//!  prototype module for bridging in ethereum poa blockchain

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use rstd::{borrow::ToOwned, result};
use rstd::{fmt::Debug, marker::PhantomData, prelude::*};
use support::{decl_event, decl_module, decl_storage, dispatch::Result, traits::Currency};
use system::ensure_signed;

use sr_primitives::RuntimeDebug;
use sr_primitives::{
	traits::{Convert, SaturatedConversion},
	AccountId32,
};

use primitives::crypto::UncheckedFrom;

use core::convert::TryFrom;

use darwinia_eth_relay::{ActionRecord, VerifyEthReceipts};
use darwinia_support::{LockableCurrency, OnDepositRedeem};
use ethabi::{Event as EthEvent, EventParam as EthEventParam, ParamType, RawLog};
use sr_eth_primitives::{receipt::LogEntry, receipt::Receipt, EthAddress, H256, U256};

//#[cfg(feature = "std")]
//use sr_primitives::{Deserialize, Serialize};

use hex_literal::hex;

use hex as xhex;

use rstd::vec::Vec;

pub type Moment = u64;

type RingBalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;
type RingPositiveImbalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type RingNegativeImbalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

type KtonBalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::Balance;
type KtonPositiveImbalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type KtonNegativeImbalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type EthRelay: VerifyEthReceipts;
	type Ring: LockableCurrency<Self::AccountId, Moment = Self::Moment>;
	type Kton: LockableCurrency<Self::AccountId, Moment = Self::Moment>;
	type OnDepositRedeem: OnDepositRedeem<Self::AccountId, Moment = Self::Moment>;
	type DetermineAccountId: AccountIdFor<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as EthBacking {
		pub RingRedeemAddress get(ring_redeem_address) build(|config: &GenesisConfig<T>| {
//			let mut r = [0u8; 20];
//			r.copy_from_slice(&config.ring_redeem_address_vec[..]);
//			r.into()
			config.ring_redeem_address_vec
		}): EthAddress;
		pub KtonRedeemAddress get(kton_redeem_address) build(|config: &GenesisConfig<T>| {
//			let mut r = [0u8; 20];
//			r.copy_from_slice(&config.kton_redeem_address_vec[..]);
//			r.into()
			config.kton_redeem_address_vec
		} ): EthAddress;
		pub DepositRedeemAddress get(deposit_redeem_address) build(|config: &GenesisConfig<T>| {
//			let mut r = [0u8; 20];
//			r.copy_from_slice(&config.deposit_redeem_address_vec[..]);
//			r.into()
			config.deposit_redeem_address_vec
		} ): EthAddress;

		pub RingLocked get(fn ring_locked) config(): RingBalanceOf<T>;
		pub KtonLocked get(fn kton_locked) config(): KtonBalanceOf<T>;
	}
	add_extra_genesis {
		// The smallest (atomic) number of points worth considering
		config(ring_redeem_address_vec): EthAddress;
		config(kton_redeem_address_vec): EthAddress;
		config(deposit_redeem_address_vec): EthAddress;
	  }
}

decl_event! {
	pub enum Event<T>
	where
		<T as system::Trait>::AccountId
	{
		TODO(AccountId),
	}
}

impl<T: Trait> Module<T> {
	pub fn adjust_deposit_value() {
		unimplemented!()
	}

	//	fn _release(_dest: &T::AccountId, _value: RingBalanceOf<T>) -> Result {
	//		unimplemented!()
	//	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		pub fn redeem_ring(origin, proof_record: ActionRecord) {
			let _relayer = ensure_signed(origin)?;

			let verified_receipt = T::EthRelay::verify_receipt(&proof_record)?;

			// event RingBurndropTokens(address indexed token, address indexed owner, uint amount, bytes data);
			// https://ropsten.etherscan.io/tx/0x81f699c93b00ab0b7db701f87b6f6045c1e0692862fcaaf8f06755abb0536800
			let eth_event = EthEvent {
				name: "RingBurndropTokens".to_owned(),
				inputs: vec![
				EthEventParam {
					name: "token".to_owned(),
					kind: ParamType::Address,
					indexed: true,
				}, EthEventParam {
					name: "owner".to_owned(),
					kind: ParamType::Address,
					indexed: true,
				}, EthEventParam {
					name: "amount".to_owned(),
					kind: ParamType::Uint(256),
					indexed: false,
				}, EthEventParam {
					name: "data".to_owned(),
					kind: ParamType::Bytes,
					indexed: false,
				}],
				anonymous: false,
			};

			// H256::from(hex!("38045eaef0a21b74ff176350f18df02d9041a25d6694b5f63e9474b7b6cd6b94")
			let log_entry = verified_receipt.logs.iter().find(
				|&x| x.address == Self::ring_redeem_address()
					 && x.topics[0] == eth_event.signature()
			).expect("Log Entry Not Found");

			let log = RawLog {
				topics: [log_entry.topics[0],log_entry.topics[1],log_entry.topics[2]].to_vec(),
				data: log_entry.data.clone()
			};

			let result = eth_event.parse_log(log).expect("Parse Eth Log Error");

			let amount : U256 = result.params[2].value.clone().to_uint().expect("Param Convert to Int Failed.");
			let raw_sub_key : Vec<u8> = result.params[3].value.clone().to_bytes().expect("Param Convert to Bytes Failed.");

			let decoded_sub_key = xhex::decode(&raw_sub_key[..]).expect("Address Hex decode Failed.");
			let darwinia_account = T::DetermineAccountId::account_id_for(&decoded_sub_key[..])?;

			 T::Ring::deposit_into_existing(&darwinia_account, (amount.as_u128() as u64).saturated_into());

//			 T::RingReward::on_unbalanced(total_imbalance);
//			 T::RingRewardRemainder::on_unbalanced(T::Ring::issue(rest));
		}

		// event KtonBurndropTokens(address indexed token, address indexed owner, uint amount, bytes data);
		pub fn redeem_kton(origin, proof_record: ActionRecord) {
			let _locker = ensure_signed(origin)?;

			let verified_receipt = T::EthRelay::verify_receipt(&proof_record)?;
		}

		// https://github.com/evolutionlandorg/bank
		// event Burndrop(uint256 indexed _depositID,  address _depositor, uint48 _months, uint48 _startAt, uint64 _unitInterest, uint128 _value, bytes _data);
		// https://ropsten.etherscan.io/tx/0xfd2cac791bb0c0bee7c5711f17ef93401061d314f4eb84e1bc91f32b73134ca1
		pub fn redeem_deposit(origin, proof_record: ActionRecord) {
			let _redeemer = ensure_signed(origin)?;

			let verified_receipt = T::EthRelay::verify_receipt(&proof_record)?;
		}
	}
}

pub trait AccountIdFor<AccountId> {
	//	fn contract_address_for(code_hash: &CodeHash, data: &[u8], origin: &AccountId) -> AccountId;
	fn account_id_for(decoded_sub_key: &[u8]) -> result::Result<AccountId, &'static str>;
}

pub struct AccountIdDeterminator<T: Trait>(PhantomData<T>);

impl<T: Trait> AccountIdFor<T::AccountId> for AccountIdDeterminator<T>
where
	T::AccountId: rstd::convert::From<[u8; 32]> + AsRef<[u8]>,
{
	fn account_id_for(decoded_sub_key: &[u8]) -> result::Result<T::AccountId, &'static str> {
		if decoded_sub_key.len() != 32 {
			return Err("Address Length - MISMATCHED");
		}

		let mut r = [0u8; 32];
		r.copy_from_slice(&decoded_sub_key[..]);

		let darwinia_account = r.into();

		//		let darwinia_account = T::AccountId::try_from(raw_sub_key).map_err(|_| "Account Parse Failed.")?;
		//		let darwinia_account = UncheckedFrom::unchecked_from(raw_sub_key[..]);
		Ok(darwinia_account)
	}
}
