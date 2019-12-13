//!  prototype module for bridging in ethereum poa blockcahin

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use rstd::borrow::ToOwned;
use rstd::prelude::*;
use support::{decl_event, decl_module, decl_storage, dispatch::Result, traits::Currency};
use system::ensure_signed;

use sr_primitives::AccountId32;
use sr_primitives::RuntimeDebug;

use core::convert::TryFrom;

use darwinia_eth_relay::{ActionRecord, VerifyEthReceipts};
use darwinia_support::{LockableCurrency, OnDepositRedeem};
use ethabi::{Event as EthEvent, EventParam as EthEventParam, ParamType, RawLog};
use sr_eth_primitives::{receipt::LogEntry, receipt::Receipt, Address, H256, U256};

//#[cfg(feature = "std")]
//use sr_primitives::{Deserialize, Serialize};

use hex_literal::hex;

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
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct EventTopic {
	/// The address of the contract executing at the point of the `LOG` operation.
	pub address: Address,
	/// The signature of the log event, matches the first topic from the topic vector, the following topic are the indexed parameters.
	pub signature: H256,
}

// config() require `serde = { version = "1.0.101", optional = true }`
// tracking issue: https://github.com/rust-lang/rust/issues/27812
decl_storage! {
	trait Store for Module<T: Trait> as EthBacking {
		pub RingRedeemTopic get(ring_redeem_topic): Option<EventTopic>;
		pub KtonRedeemTopic get(kton_redeem_topic): Option<EventTopic>;
		pub DepositRedeemTopic get(deposit_redeem_topic): Option<EventTopic>;

		pub RingLocked get(fn ring_locked): RingBalanceOf<T>;
		pub KtonLocked get(fn kton_locked): KtonBalanceOf<T>;
	}
	add_extra_genesis {
		config(genesis_ring_redeem_topic): Option<Vec<u8>>;
		config(genesis_kton_redeem_topic): Option<Vec<u8>>;
		config(genesis_deposit_redeem_topic): Option<Vec<u8>>;
		build(|config| {
			// https://github.com/evolutionlandorg/bank
			// event Burndrop(uint256 indexed _depositID,  address _depositor, uint48 _months, uint48 _startAt, uint64 _unitInterest, uint128 _value, bytes _data);
			// https://ropsten.etherscan.io/tx/0xfd2cac791bb0c0bee7c5711f17ef93401061d314f4eb84e1bc91f32b73134ca1

			// event RingBurndropTokens(address indexed token, address indexed owner, uint amount, bytes data);
			// https://ropsten.etherscan.io/tx/0x81f699c93b00ab0b7db701f87b6f6045c1e0692862fcaaf8f06755abb0536800

			// event KtonBurndropTokens(address indexed token, address indexed owner, uint amount, bytes data);
			RingRedeemTopic::put(EventTopic {
				address: Address::from(hex!("dbc888d701167cbfb86486c516aafbefc3a4de6e")),
				signature: H256::from(hex!("38045eaef0a21b74ff176350f18df02d9041a25d6694b5f63e9474b7b6cd6b94")),
			});

			//pub const KtonRedeemTopic : EventTopic  = EventTopic {
			//	address: Address::from_str("dbc888d701167cbfb86486c516aafbefc3a4de6e").unwrap(),
			//	signature: H256::from(hex!(""))
			//};

			DepositRedeemTopic::put(EventTopic {
				address: Address::from(hex!("ad52e0f67b6f44cd5b9a6f4fbc7c0f78f37e094b")),
				signature: H256::from(hex!("455d5fda67197daa1239477da37301be9abb7771027186e589d8c341c609d285")),
			});

		});
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

			let log_entry = verified_receipt.logs.iter().find(
				|&x| x.address == Address::from(hex!("dbc888d701167cbfb86486c516aafbefc3a4de6e"))
					 && x.topics[0] == H256::from(hex!("38045eaef0a21b74ff176350f18df02d9041a25d6694b5f63e9474b7b6cd6b94"))
			).expect("Log Entry Not Found");

			// event RingBurndropTokens(address indexed token, address indexed owner, uint amount, bytes data);
			let eth_event = EthEvent {
				name: "RingBurndropTokens".to_owned(),
				inputs: vec![EthEventParam {
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

			let log = RawLog {
				topics: [log_entry.topics[0],log_entry.topics[1],log_entry.topics[2]].to_vec(),
				data: log_entry.data.clone()
			};

			let result = eth_event.parse_log(log).expect("Parse Eth Log Error");

			let amount : U256 = result.params[2].value.clone().to_uint().expect("Param Convert to Int Failed.");
			let raw_sub_key : Vec<u8> = result.params[3].value.clone().to_bytes().expect("Param Convert to Bytes Failed.");

			let darwinia_account: AccountId32 = AccountId32::try_from(raw_sub_key.as_slice()).expect("Account Parse Failed.");
//			let mut r = T::AccountId::default();
//			r.0.copy_from_slice(raw_sub_key.as_slice());

//			let darwinia_account: AccountId32 = hex!["d7b504ddbe25a05647312daa8d0bbbafba360686241b7e193ca90f9b01f95faa"].into();

// Self::Ring::issue(raw_sub_key, amount);
		}

		pub fn redeem_kton(origin, proof_record: ActionRecord) {
			let _locker = ensure_signed(origin)?;

			let verified_receipt = T::EthRelay::verify_receipt(&proof_record)?;
		}

		pub fn redeem_deposit(origin, proof_record: ActionRecord) {
			let _redeemer = ensure_signed(origin)?;

			let verified_receipt = T::EthRelay::verify_receipt(&proof_record)?;
		}
	}
}

impl<T: Trait> Module<T> {
	//	pub fn to_hex_string(bytes: Vec<u8>) -> String {
	//		let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
	//		strs.join("")
	//	}
}
