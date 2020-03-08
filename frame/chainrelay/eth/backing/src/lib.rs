//! Prototype module for cross chain assets backing.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]

#[cfg(all(feature = "std", test))]
mod mock;
#[cfg(all(feature = "std", test))]
mod tests;

mod types {
	use crate::*;

	/// Balance of an account.
	pub type Balance = u128;
	pub type DepositId = U256;
	/// Type used for expressing timestamp.
	pub type Moment = Timestamp;

	pub type MomentT<T> = <TimeT<T> as Time>::Moment;

	pub type RingBalance<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;
	pub type RingPositiveImbalance<T> =
		<<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

	pub type KtonBalance<T> = <<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::Balance;
	pub type KtonPositiveImbalance<T> =
		<<T as Trait>::Kton as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

	pub type EthTransactionIndex = (H256, u64);

	/// A timestamp: milliseconds since the unix epoch.
	/// `u64` is enough to represent a duration of half a billion years, when the
	/// time scale is milliseconds.
	type Timestamp = u64;

	type TimeT<T> = <T as Trait>::Time;
}

use ethabi::{Event as EthEvent, EventParam as EthEventParam, ParamType, RawLog};
use frame_support::{
	decl_event, decl_module, decl_storage, ensure,
	traits::{Currency, OnUnbalanced, Time},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::{CheckedSub, SaturatedConversion};
#[cfg(not(feature = "std"))]
use sp_std::borrow::ToOwned;
use sp_std::{convert::TryFrom, marker::PhantomData, vec};

use darwinia_eth_relay::{EthReceiptProof, VerifyEthReceipts};
use darwinia_support::traits::{LockableCurrency, OnDepositRedeem};
use eth_primitives::{EthAddress, H256, U256};
use types::*;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type Time: Time;

	type DetermineAccountId: AccountIdFor<Self::AccountId>;

	type EthRelay: VerifyEthReceipts;

	type OnDepositRedeem: OnDepositRedeem<Self::AccountId, Balance = RingBalance<Self>, Moment = MomentT<Self>>;

	type Ring: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	type RingReward: OnUnbalanced<RingPositiveImbalance<Self>>;

	type Kton: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	type KtonReward: OnUnbalanced<KtonPositiveImbalance<Self>>;
}

decl_storage! {
	trait Store for Module<T: Trait> as EthBacking {
		pub RingLocked get(fn ring_locked) config(): RingBalance<T>;
		pub RingProofVerified get(fn ring_proof_verfied): map hasher(blake2_256) EthTransactionIndex => Option<EthReceiptProof>;
		pub RingRedeemAddress get(fn ring_redeem_address) config(): EthAddress;

		pub KtonLocked get(fn kton_locked) config(): KtonBalance<T>;
		pub KtonProofVerified get(fn kton_proof_verfied): map hasher(blake2_256) EthTransactionIndex => Option<EthReceiptProof>;
		pub KtonRedeemAddress get(fn kton_redeem_address) config(): EthAddress;

		pub DepositProofVerified get(fn deposit_proof_verfied): map hasher(blake2_256) EthTransactionIndex => Option<EthReceiptProof>;
		pub DepositRedeemAddress get(fn deposit_redeem_address) config(): EthAddress;
	}
}

decl_event! {
	pub enum Event<T>
	where
		<T as system::Trait>::AccountId,
		RingBalance = RingBalance<T>,
		KtonBalance = KtonBalance<T>,
	{
		RedeemRing(AccountId, RingBalance, EthTransactionIndex),
		RedeemKton(AccountId, KtonBalance, EthTransactionIndex),
		RedeemDeposit(AccountId, DepositId, RingBalance, EthTransactionIndex),
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		fn deposit_event() = default;

		// event RingBurndropTokens(address indexed token, address indexed owner, uint amount, bytes data)
		// https://ropsten.etherscan.io/tx/0x81f699c93b00ab0b7db701f87b6f6045c1e0692862fcaaf8f06755abb0536800
		pub fn redeem_ring(origin, proof_record: EthReceiptProof) {
			let _relayer = ensure_signed(origin)?;

			ensure!(
				!RingProofVerified::contains_key((proof_record.header_hash, proof_record.index)),
				"Ring For This Proof - ALREADY BEEN REDEEMED",
			);

			let (darwinia_account, redeemed_amount) = Self::parse_token_redeem_proof(&proof_record, "RingBurndropTokens")?;

			let redeemed_ring = <RingBalance<T>>::saturated_from(redeemed_amount);

			let new_ring_locked = Self::ring_locked()
				.checked_sub(&redeemed_ring)
				.ok_or("RING Locked - NO SUFFICIENT BACKING ASSETS")?;
			let redeemed_positive_imbalance_ring = T::Ring::deposit_into_existing(&darwinia_account, redeemed_ring)?;

			T::RingReward::on_unbalanced(redeemed_positive_imbalance_ring);

			RingProofVerified::insert((proof_record.header_hash, proof_record.index), &proof_record);

			<RingLocked<T>>::mutate(|l| {
				*l = new_ring_locked;
			});

			<Module<T>>::deposit_event(RawEvent::RedeemRing(
				darwinia_account,
				redeemed_ring,
				(proof_record.header_hash, proof_record.index),
			));
		}

		// event KtonBurndropTokens(address indexed token, address indexed owner, uint amount, bytes data)
		pub fn redeem_kton(origin, proof_record: EthReceiptProof) {
			let _relayer = ensure_signed(origin)?;

			ensure!(
				!KtonProofVerified::contains_key((proof_record.header_hash, proof_record.index)),
				"Kton For This Proof - ALREADY BEEN REDEEMED",
			);

			let (darwinia_account, redeemed_amount) = Self::parse_token_redeem_proof(&proof_record, "KtonBurndropTokens")?;

			let redeemed_kton = <KtonBalance<T>>::saturated_from(redeemed_amount);
			let new_kton_locked = Self::kton_locked()
				.checked_sub(&redeemed_kton)
				.ok_or("KTON Locked - NO SUFFICIENT BACKING ASSETS")?;

			let redeemed_positive_imbalance_kton = T::Kton::deposit_into_existing(&darwinia_account, redeemed_kton)?;
			T::KtonReward::on_unbalanced(redeemed_positive_imbalance_kton);

			KtonProofVerified::insert((proof_record.header_hash, proof_record.index), &proof_record);

			<KtonLocked<T>>::mutate(|l| {
				*l = new_kton_locked;
			});

			<Module<T>>::deposit_event(RawEvent::RedeemKton(
				darwinia_account,
				redeemed_kton,
				(proof_record.header_hash, proof_record.index),
			));
		}

		// https://github.com/evolutionlandorg/bank
		// event Burndrop(uint256 indexed _depositID,  address _depositor, uint48 _months, uint48 _startAt, uint64 _unitInterest, uint128 _value, bytes _data)
		// https://ropsten.etherscan.io/tx/0xfd2cac791bb0c0bee7c5711f17ef93401061d314f4eb84e1bc91f32b73134ca1
		pub fn redeem_deposit(origin, proof_record: EthReceiptProof) {
			let _relayer = ensure_signed(origin)?;

			ensure!(
				!DepositProofVerified::contains_key((proof_record.header_hash, proof_record.index)),
				"Deposit For This Proof - ALREADY BEEN REDEEMED",
			);

			let result = {
				let verified_receipt = T::EthRelay::verify_receipt(&proof_record)?;
				let eth_event = EthEvent {
					name: "Burndrop".to_owned(),
					inputs: vec![
						EthEventParam { name: "_depositID".to_owned(), kind: ParamType::Uint(256), indexed: true },
						EthEventParam { name: "_depositor".to_owned(), kind: ParamType::Address, indexed: false },
						EthEventParam { name: "_months".to_owned(), kind: ParamType::Uint(48), indexed: false },
						EthEventParam { name: "_startAt".to_owned(), kind: ParamType::Uint(48), indexed: false },
						EthEventParam { name: "_unitInterest".to_owned(), kind: ParamType::Uint(64), indexed: false },
						EthEventParam { name: "_value".to_owned(), kind: ParamType::Uint(128), indexed: false },
						EthEventParam { name: "_data".to_owned(), kind: ParamType::Bytes, indexed: false }
					],
					anonymous: false,
				};
				let log_entry = verified_receipt
					.logs
					.iter()
					.find(|&x| x.address == Self::deposit_redeem_address() && x.topics[0] == eth_event.signature())
					.ok_or("Log Entry - NOT FOUND")?;
				let log = RawLog {
					topics: vec![log_entry.topics[0],log_entry.topics[1]],
					data: log_entry.data.clone()
				};

				eth_event.parse_log(log).map_err(|_| "Parse Eth Log - FAILED")?
			};
			let deposit_id = result
				.params[0]
				.value
				.clone()
				.to_uint()
				.ok_or("Convert to Int - FAILED")?;
			let month = {
				let month = result
					.params[2]
					.value
					.clone()
					.to_uint()
					.ok_or("Convert to Int - FAILED")?;

				<MomentT<T>>::saturated_from(Moment::try_from(month)? as _)
			};
			// https://github.com/evolutionlandorg/bank/blob/master/contracts/GringottsBankV2.sol#L178
			// The start_at here is in seconds, will be converted to milliseconds later in on_deposit_redeem
			let start_at = {
				let start_at = result
					.params[3]
					.value
					.clone()
					.to_uint()
					.ok_or("Convert to Int - FAILED")?;

				<MomentT<T>>::saturated_from(Moment::try_from(start_at)? as _)
			};
			let redeemed_ring = {
				// The decimal in Ethereum is 10**18, and the decimal in Darwinia is 10**9,
				// div 10**18 and mul 10**9
				let amount = result.params[5]
					.value
					.clone()
					.to_uint()
					.map(|x| x / U256::from(1_000_000_000u64))
					.ok_or("Convert to Int - FAILED")?;

				<RingBalance<T>>::saturated_from(Balance::try_from(amount)?)
			};
			let darwinia_account = {
				let raw_sub_key = result.params[6]
					.value
					.clone()
					.to_bytes()
					.ok_or("Convert to Bytes - FAILED")?;
//				let decoded_sub_key = hex::decode(&raw_sub_key).map_err(|_| "Decode Address - FAILED")?;

				T::DetermineAccountId::account_id_for(&raw_sub_key)?
			};
			let new_ring_locked = Self::ring_locked()
				.checked_sub(&redeemed_ring)
				.ok_or("RING Locked - NO SUFFICIENT BACKING ASSETS")?;

			T::OnDepositRedeem::on_deposit_redeem(start_at, month, redeemed_ring, &darwinia_account)?;

			// TODO: check deposit_id duplication

			// TODO: Ignore Unit Interest for now

			DepositProofVerified::insert((proof_record.header_hash, proof_record.index), &proof_record);

			<RingLocked<T>>::mutate(|l| {
				*l = new_ring_locked;
			});

			<Module<T>>::deposit_event(RawEvent::RedeemDeposit(
				darwinia_account,
				deposit_id,
				redeemed_ring,
				(proof_record.header_hash, proof_record.index),
			));
		}
	}
}

impl<T: Trait> Module<T> {
	fn parse_token_redeem_proof(
		proof: &EthReceiptProof,
		event_name: &str,
	) -> Result<(T::AccountId, Balance), &'static str> {
		let result = {
			let verified_receipt = T::EthRelay::verify_receipt(proof)?;
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

			//			let decoded_sub_key = hex::decode(&raw_sub_key).map_err(|_| "Decode Address - FAILED")?;

			T::DetermineAccountId::account_id_for(&raw_sub_key)?
		};

		Ok((darwinia_account, redeemed_amount))
	}
}

pub trait AccountIdFor<AccountId> {
	fn account_id_for(decoded_sub_key: &[u8]) -> Result<AccountId, &'static str>;
}

pub struct AccountIdDeterminator<T: Trait>(PhantomData<T>);

impl<T: Trait> AccountIdFor<T::AccountId> for AccountIdDeterminator<T>
where
	T::AccountId: sp_std::convert::From<[u8; 32]> + AsRef<[u8]>,
{
	fn account_id_for(decoded_sub_key: &[u8]) -> Result<T::AccountId, &'static str> {
		ensure!(decoded_sub_key.len() == 33, "Address Length - MISMATCHED");
		ensure!(decoded_sub_key[0] == 42, "Pubkey Prefix - MISMATCHED");

		let mut raw_account = [0u8; 32];
		raw_account.copy_from_slice(&decoded_sub_key[1..]);

		Ok(raw_account.into())
	}
}

impl<T: Trait> Module<T> {
	pub fn adjust_deposit_value() {
		unimplemented!()
	}
}
