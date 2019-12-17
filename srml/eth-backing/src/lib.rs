#![feature(in_band_lifetimes)]
//!  prototype module for cross chain assets backing

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

//use codec::{Decode, Encode};
use ethabi::{Event as EthEvent, EventParam as EthEventParam, ParamType, RawLog};
use rstd::{borrow::ToOwned, convert::TryFrom, marker::PhantomData, result, vec::Vec}; // fmt::Debug
use sr_primitives::traits::{SaturatedConversion, Saturating};
use support::{decl_event, decl_module, decl_storage, ensure, traits::Currency, traits::OnUnbalanced}; // dispatch::Result,
use system::ensure_signed; // Convert,

//use sr_primitives::RuntimeDebug;
//use primitives::crypto::UncheckedFrom;
//use core::convert::TryFrom;

use darwinia_eth_relay::{EthReceiptProof, VerifyEthReceipts};
use darwinia_support::{LockableCurrency, OnDepositRedeem};
use sr_eth_primitives::{EthAddress, H256, U256}; // receipt::LogEntry, receipt::Receipt,

//#[cfg(feature = "std")]
//use sr_primitives::{Deserialize, Serialize};

pub type Balance = u128;
pub type Moment = u64;

type RingBalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;
type PositiveImbalanceRing<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
//type NegativeImbalanceRing<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

type KtonBalanceOf<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::Balance;
type PositiveImbalanceKton<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
//type NegativeImbalanceKton<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type EthRelay: VerifyEthReceipts;
	type Ring: LockableCurrency<Self::AccountId, Moment = Self::Moment>;
	type Kton: LockableCurrency<Self::AccountId, Moment = Self::Moment>;
	type OnDepositRedeem: OnDepositRedeem<Self::AccountId, Moment = Self::Moment>;
	type DetermineAccountId: AccountIdFor<Self::AccountId>;
	type RingReward: OnUnbalanced<PositiveImbalanceRing<Self>>;
	type KtonReward: OnUnbalanced<PositiveImbalanceKton<Self>>;
}

decl_storage! {
	trait Store for Module<T: Trait> as EthBacking {
		pub RingRedeemAddress get(ring_redeem_address) config(): EthAddress;
		pub KtonRedeemAddress get(kton_redeem_address) config(): EthAddress;
		pub DepositRedeemAddress get(deposit_redeem_address) config(): EthAddress;

		pub RingLocked get(fn ring_locked) config(): RingBalanceOf<T>;
		pub KtonLocked get(fn kton_locked) config(): KtonBalanceOf<T>;

		pub RingProofVerified get(ring_proof_verfied): map (H256, u64) => Option<EthReceiptProof>;
		pub KtonProofVerified get(kton_proof_verfied): map (H256, u64) => Option<EthReceiptProof>;
		pub DepositProofVerified get(deposit_proof_verfied): map (H256, u64) => Option<EthReceiptProof>;
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
		pub fn redeem_ring(origin, proof_record: EthReceiptProof) {
			let _relayer = ensure_signed(origin)?;

			ensure!(
				!RingProofVerified::exists((proof_record.header_hash, proof_record.index)),
				"Ring For This Proof - ALREADY BEEN REDEEMED",
			);

			let (darwinia_account, redeemed_amount) = Self::parse_proof(&proof_record, "RingBurndropTokens")?;
			let redeemed_ring = <RingBalanceOf<T>>::saturated_from(redeemed_amount);
			let redeemed_positive_imbalance_ring = T::Ring::deposit_into_existing(&darwinia_account, redeemed_ring)?;

			T::RingReward::on_unbalanced(redeemed_positive_imbalance_ring);

			RingProofVerified::insert((proof_record.header_hash, proof_record.index), proof_record);
			<RingLocked<T>>::mutate(|l| {
				*l = l.saturating_sub(redeemed_ring);
			});
		}

		// event KtonBurndropTokens(address indexed token, address indexed owner, uint amount, bytes data);
		pub fn redeem_kton(origin, proof_record: EthReceiptProof) {
			let _relayer = ensure_signed(origin)?;

			ensure!(
				!KtonProofVerified::exists((proof_record.header_hash, proof_record.index)),
				"Kton For This Proof - ALREADY BEEN REDEEMED",
			);

			let (darwinia_account, redeemed_amount) = Self::parse_proof(&proof_record, "KtonBurndropTokens")?;
			let redeemed_kton = <KtonBalanceOf<T>>::saturated_from(redeemed_amount);
			let redeemed_positive_imbalance_kton = T::Kton::deposit_into_existing(&darwinia_account, redeemed_kton)?;

			T::KtonReward::on_unbalanced(redeemed_positive_imbalance_kton);

			KtonProofVerified::insert((proof_record.header_hash, proof_record.index), proof_record);
			<KtonLocked<T>>::mutate(|l| {
				*l = l.saturating_sub(redeemed_kton);
			});
		}

		// https://github.com/evolutionlandorg/bank
		// event Burndrop(uint256 indexed _depositID,  address _depositor, uint48 _months, uint48 _startAt, uint64 _unitInterest, uint128 _value, bytes _data);
		// https://ropsten.etherscan.io/tx/0xfd2cac791bb0c0bee7c5711f17ef93401061d314f4eb84e1bc91f32b73134ca1
		pub fn redeem_deposit(origin, proof_record: EthReceiptProof) {
			let _relayer = ensure_signed(origin)?;

			ensure!(!<DepositProofVerified>::exists((proof_record.header_hash, proof_record.index)), "Deposit for this proof has already been redeemed.");

			let verified_receipt = T::EthRelay::verify_receipt(&proof_record)?;

			// event RingBurndropTokens(address indexed token, address indexed owner, uint amount, bytes data);
			// https://ropsten.etherscan.io/tx/0x81f699c93b00ab0b7db701f87b6f6045c1e0692862fcaaf8f06755abb0536800
			let eth_event = EthEvent {
				name: "Burndrop".to_owned(),
				inputs: vec![
					EthEventParam {name: "_depositID".to_owned(), kind: ParamType::Uint(256), indexed: true,},
					EthEventParam {name: "_depositor".to_owned(), kind: ParamType::Address, indexed: false,},
					EthEventParam {name: "_months".to_owned(), kind: ParamType::Uint(48), indexed: false,},
					EthEventParam {name: "_startAt".to_owned(), kind: ParamType::Uint(48), indexed: false,},
					EthEventParam {name: "_unitInterest".to_owned(), kind: ParamType::Uint(64), indexed: false,},
					EthEventParam {name: "_value".to_owned(), kind: ParamType::Uint(128), indexed: false,},
					EthEventParam {name: "_data".to_owned(), kind: ParamType::Bytes, indexed: false,}
				],
				anonymous: false,
			};

			let log_entry = verified_receipt.logs.iter().find(
				|&x| x.address == Self::deposit_redeem_address()
					 && x.topics[0] == eth_event.signature()
			).expect("Log Entry Not Found");

			let log = RawLog {
				topics: [log_entry.topics[0],log_entry.topics[1]].to_vec(),
				data: log_entry.data.clone()
			};

			let result = eth_event.parse_log(log).expect("Parse Eth Log Error");

			let _deposit_id : U256 = result.params[0].value.clone().to_uint().expect("Param Convert to Int Failed.");
			let month : U256 = result.params[2].value.clone().to_uint().expect("Param Convert to Int Failed.");

			// TODO: Check the time unit in seconds or milliseconds
			let start_at : U256 = result.params[3].value.clone().to_uint().expect("Param Convert to Int Failed.");
			// TODO: div 10**18 and mul 10**9
			let mut amount: U256 = result.params[5].value.clone().to_uint().expect("Param Convert to Int Failed.");
			amount = amount / U256::from(1_000_000_000u64);
			let raw_sub_key : Vec<u8> = result.params[6].value.clone().to_bytes().expect("Param Convert to Bytes Failed.");

			let decoded_sub_key = hex::decode(&raw_sub_key[..]).expect("Address Hex decode Failed.");
			let darwinia_account = T::DetermineAccountId::account_id_for(&decoded_sub_key[..])?;

			let redeemed_amount = amount.as_u128().saturated_into();

			<RingLocked<T>>::mutate(|l| {
				*l = l.saturating_sub(redeemed_amount);
			});

			<DepositProofVerified>::insert((proof_record.header_hash, proof_record.index), proof_record);

			let redeemed_kton = T::Ring::deposit_into_existing(&darwinia_account, redeemed_amount).expect("Deposit into existing failed.");

			T::RingReward::on_unbalanced(redeemed_kton);

			// TODO: check deposit_id duplication

			// TODO: Ignore Unit Interest for now

			T::OnDepositRedeem::on_deposit_redeem(month.saturated_into(), start_at.saturated_into(), amount.as_u128(), &darwinia_account)?;
		}
	}
}

impl<T: Trait> Module<T> {
	fn parse_proof(proof: &EthReceiptProof, event_name: &str) -> result::Result<(T::AccountId, Balance), &'static str> {
		let verified_receipt = T::EthRelay::verify_receipt(proof)?;

		// event RingBurndropTokens(address indexed token, address indexed owner, uint amount, bytes data);
		// https://ropsten.etherscan.io/tx/0x81f699c93b00ab0b7db701f87b6f6045c1e0692862fcaaf8f06755abb0536800
		// event KtonBurndropTokens(address indexed token, address indexed owner, uint amount, bytes data);
		// TODO: address
		let result = {
			let eth_event = EthEvent {
				name: event_name.to_owned(),
				inputs: vec![
					EthEventParam {
						name: "token".to_owned(),
						kind: ParamType::Address,
						indexed: true,
					},
					EthEventParam {
						name: "owner".to_owned(),
						kind: ParamType::Address,
						indexed: true,
					},
					EthEventParam {
						name: "amount".to_owned(),
						kind: ParamType::Uint(256),
						indexed: false,
					},
					EthEventParam {
						name: "data".to_owned(),
						kind: ParamType::Bytes,
						indexed: false,
					},
				],
				anonymous: false,
			};
			let log_entry = verified_receipt
				.logs
				.into_iter()
				.find(|x| x.address == Self::ring_redeem_address() && x.topics[0] == eth_event.signature())
				.ok_or("Log Entry - NOT FOUND")?;
			let log = RawLog {
				topics: vec![log_entry.topics[0], log_entry.topics[1], log_entry.topics[2]],
				data: log_entry.data.clone(),
			};

			eth_event.parse_log(log).map_err(|_| "Parse Eth Log - FAILED")?
		};
		let redeemed_amount = {
			// TODO: div 10**18 and mul 10**9
			let amount = result.params[2]
				.value
				.clone()
				.to_uint()
				.map(|x| x / U256::from(1_000_000_000u64))
				.ok_or("Convert to Int - FAILED")?;

			Balance::try_from(amount)?
		};
		let darwinia_account = {
			let raw_sub_key = result.params[3]
				.value
				.clone()
				.to_bytes()
				.ok_or("Convert to Bytes - FAILED")?;
			let decoded_sub_key = hex::decode(&raw_sub_key).map_err(|_| "Decode Address - FAILED")?;

			T::DetermineAccountId::account_id_for(&decoded_sub_key)?
		};

		Ok((darwinia_account, redeemed_amount))
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
