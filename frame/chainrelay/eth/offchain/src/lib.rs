#![cfg_attr(not(feature = "std"), no_std)]

pub mod crypto {
	// --- third-party ---
	use sp_runtime::app_crypto::{app_crypto, sr25519};

	// --- custom ---
	use crate::KEY_TYPE;

	app_crypto!(sr25519, KEY_TYPE);
}

mod ethscan_url {
	pub const GTE_BLOCK_BY_TIMESTAMP: &'static [u8] =
		b"https://api.etherscan.io/api?module=block&action=getblocknobytime&timestamp=";
	pub const GTE_BLOCK_BY_BLOCK_NUMBER: &'static [u8] =
		b"https://api.etherscan.io/api?module=proxy&action=eth_getBlockByNumber&tag=0x";
}

#[cfg(all(feature = "std", test))]
mod mock;
#[cfg(all(feature = "std", test))]
mod tests;

// --- third-party ---
use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage, dispatch,
	traits::{Get, Time},
};
use frame_system::{self as system, offchain::SubmitSignedTransaction};
use hex::FromHex;
use simple_json::{self, json::JsonValue};
use sp_runtime::{offchain::http::Request, traits::SaturatedConversion, DispatchError, DispatchResult, KeyTypeId};
use sp_std::prelude::*;

// --- custom ---
use eth_primitives::{header::EthHeader, pow::EthashSeal};
use pallet_eth_relay::HeaderInfo;

type EthScanAPIKey = Option<Vec<u8>>;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ofpf");

const MAX_RETRY: u8 = 3;
const RETRY_INTERVAL: u64 = 1;

pub trait Trait: pallet_eth_relay::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type Time: Time;

	type Call: From<Call<Self>>;

	type SubmitSignedTransaction: SubmitSignedTransaction<Self, <Self as Trait>::Call>;

	type BlockFetchDur: Get<Self::BlockNumber>;

	type APIKey: Get<EthScanAPIKey>;
}
enum EthScanAPI {
	GetBlockNoByTime,
	GetBlockByNumber,
}

decl_event! {
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId
	{
		OffchainRelayChainApiKey(AccountId), // currently not use, implement someday not now
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// API Resoibse - UNEXPECTED
		APIRespUnexp,

		/// Json - PARSING FAILED
		JsonPF,
		/// Bloom - CONVERTION FAILED
		BloomCF,
		/// Bytes - CONVERTION FAILED
		BytesCF,
		/// EthAddress - CONVERTION FAILED
		EthAddrCF,
		/// H64 - CONVERTION FALLED
		H64CF,
		/// H256 - CONVERTION FALLED
		H256CF,
		/// U64 - CONVERTION FAILED
		U64CF,
		/// U256 - CONVERTION FAILED
		U256CF,
		/// Str - CONVERTION FAILED
		StrCF,
		/// URL - DECODE FAILED
		URLDF,

		/// Pending Length - MISMATCHED
		PaddingLenMis,
		/// Response Code - MISMATCHED
		RespCodeMis,

		/// Request - REACH MAX RETRY
		ReqRMR,
	}
}

decl_storage! {
  trait Store for Module<T: Trait> as EthOffchain {
  }
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		type Error = Error<T>;

		fn deposit_event() = default;

		pub fn record_header(
			origin,
			_block: T::BlockNumber,
			eth_header: EthHeader
		) -> dispatch::DispatchResult {
			<pallet_eth_relay::Module<T>>::relay_header(origin, eth_header)
		}

		fn offchain_worker(block: T::BlockNumber) {
			let duration = T::BlockFetchDur::get();
			if duration > 0.into() && block % duration == 0.into() && T::APIKey::get().is_some() {
				if let Err(e) = Self::fetch_eth_header(block) {
					debug::error!("[eth-offchain] Error: {:#?}", e);
				}
			}
		}
	}
}

impl<T: Trait> Module<T> {
	fn json_request(raw_url: &Vec<u8>, api_type: EthScanAPI) -> Result<JsonValue, DispatchError> {
		let url = core::str::from_utf8(raw_url).map_err(|_| <Error<T>>::URLDF)?;
		debug::warn!("[eth-offchain] Request: {}", url);
		let mut maybe_resp_body = None;

		for retry_time in 0..=MAX_RETRY {
			debug::warn!("[eth-offchain] Retry: {}", retry_time);
			if let Ok(pending) = Request::get(&url).send() {
				if let Ok(resp) = pending.wait() {
					if resp.code == 200 {
						let resp_body = resp.body().collect::<Vec<u8>>();
						if resp_body[0] == 123u8 {
							maybe_resp_body = Some(resp_body);
							break;
						}
					} else {
						debug::warn!("[eth-offchain] Status Code: {}", resp.code);
					}
				}
			}

			#[cfg(feature = "std")]
			std::thread::sleep(std::time::Duration::from_secs(RETRY_INTERVAL));
		}

		let mut resp_body = maybe_resp_body.ok_or(<Error<T>>::ReqRMR)?;
		debug::trace!(
			"[eth-offchain] Response: {}",
			core::str::from_utf8(&resp_body).unwrap_or("Resposne Body - INVALID"),
		);

		let json_val: JsonValue = match api_type {
			EthScanAPI::GetBlockByNumber => {
				if resp_body.len() < 1362 {
					Err(<Error<T>>::APIRespUnexp)?;
				}
				remove_trascation_and_uncle(&mut resp_body);
				// get the result part
				simple_json::parse_json(
					&core::str::from_utf8(&resp_body[33..resp_body.len() - 1]).map_err(|_| <Error<T>>::StrCF)?,
				)
				.map_err(|_| <Error<T>>::JsonPF)?
			}
			_ => simple_json::parse_json(&core::str::from_utf8(&resp_body).map_err(|_| <Error<T>>::StrCF)?)
				.map_err(|_| <Error<T>>::JsonPF)?,
		};

		Ok(json_val)
	}

	fn hex_padding<A: AsRef<[u8]>>(width: usize, content: A) -> Result<Vec<u8>, DispatchError> {
		let content = content.as_ref();
		let mut output = vec![48; width];
		output[width.checked_sub(content.len()).ok_or(<Error<T>>::PaddingLenMis)?..].copy_from_slice(content);

		Ok(output)
	}

	fn build_eth_seal<A: AsRef<[u8]>>(mix_hash_hex: A, nonce_hex: A) -> Result<EthashSeal, DispatchError> {
		let mix_hash_hex = mix_hash_hex.as_ref();
		let nonce_hex = nonce_hex.as_ref();
		let s = EthashSeal {
			mix_hash: <[u8; 32]>::from_hex(mix_hash_hex)
				.map_err(|_| <Error<T>>::H256CF)?
				.into(),
			nonce: <[u8; 8]>::from_hex(nonce_hex).map_err(|_| <Error<T>>::H64CF)?.into(),
		};

		Ok(s)
	}

	// TODO: we may store the eth header info on chain install of all eth headers
	fn _build_eth_header_info<A: AsRef<[u8]>>(
		block_height: u64,
		total_difficulty_hex: A,
		parent_hash_hex: A,
	) -> Result<HeaderInfo, DispatchError> {
		let total_difficulty = Self::hex_padding(64, total_difficulty_hex.as_ref())?;
		let parent_hash = parent_hash_hex.as_ref();
		let h = HeaderInfo {
			number: block_height,
			total_difficulty: <[u8; 32]>::from_hex(total_difficulty)
				.map_err(|_| "Total difficulty decoding failed")?
				.into(),
			parent_hash: <[u8; 32]>::from_hex(parent_hash)
				.map_err(|_| "parent hash decoding failed")?
				.into(),
		};

		Ok(h)
	}

	fn build_eth_header(number: u64, block_info: JsonValue) -> Result<EthHeader, DispatchError> {
		let parent_hash = &block_info.get_object()[10].1.get_bytes()[2..];
		let timestamp_hex = &block_info.get_object()[15].1.get_string()[2..];
		let author = &block_info.get_object()[6].1.get_bytes()[2..];
		let uncles_hash = &block_info.get_object()[12].1.get_bytes()[2..];
		let extra_data = &block_info.get_object()[1].1.get_bytes()[2..];
		let state_root = &block_info.get_object()[14].1.get_bytes()[2..];
		let receipts_root = &block_info.get_object()[11].1.get_bytes()[2..];
		let bloom = &block_info.get_object()[5].1.get_bytes()[2..];
		let gas_used = Self::hex_padding(64, &block_info.get_object()[3].1.get_bytes()[2..])?;
		let gas_limit = Self::hex_padding(64, &block_info.get_object()[2].1.get_bytes()[2..])?;
		let difficulty = Self::hex_padding(64, &block_info.get_object()[0].1.get_bytes()[2..])?;
		let seal = Self::build_eth_seal(
			&block_info.get_object()[7].1.get_bytes()[2..],
			&block_info.get_object()[8].1.get_bytes()[2..],
		)?;
		let transactions_root = &block_info.get_object()[17].1.get_bytes()[2..];
		let hash = &block_info.get_object()[4].1.get_bytes()[2..];

		let h = EthHeader {
			parent_hash: <[u8; 32]>::from_hex(parent_hash)
				.map_err(|_| <Error<T>>::H256CF)?
				.into(),
			timestamp: u64::from_str_radix(&timestamp_hex, 16).map_err(|_| <Error<T>>::U64CF)?,
			number,
			author: <[u8; 20]>::from_hex(author).map_err(|_| <Error<T>>::EthAddrCF)?.into(),
			transactions_root: <[u8; 32]>::from_hex(transactions_root)
				.map_err(|_| <Error<T>>::H256CF)?
				.into(),
			uncles_hash: <[u8; 32]>::from_hex(uncles_hash)
				.map_err(|_| <Error<T>>::H256CF)?
				.into(),
			extra_data: <Vec<u8>>::from_hex(extra_data).map_err(|_| <Error<T>>::BytesCF)?.into(),
			state_root: <[u8; 32]>::from_hex(state_root).map_err(|_| <Error<T>>::H256CF)?.into(),
			receipts_root: <[u8; 32]>::from_hex(receipts_root)
				.map_err(|_| <Error<T>>::H256CF)?
				.into(),
			log_bloom: <[u8; 256]>::from_hex(bloom).map_err(|_| <Error<T>>::BloomCF)?.into(),
			gas_used: <[u8; 32]>::from_hex(gas_used).map_err(|_| <Error<T>>::U256CF)?.into(),
			gas_limit: <[u8; 32]>::from_hex(gas_limit).map_err(|_| <Error<T>>::U256CF)?.into(),
			difficulty: <[u8; 32]>::from_hex(difficulty).map_err(|_| <Error<T>>::U256CF)?.into(),
			seal: vec![rlp::encode(&seal.mix_hash), rlp::encode(&seal.nonce)],
			hash: Some(<[u8; 32]>::from_hex(hash).map_err(|_| <Error<T>>::H256CF)?.into()),
		};

		Ok(h)
	}

	fn fetch_eth_header<'a>(block: T::BlockNumber) -> DispatchResult {
		let now = T::Time::now().saturated_into::<u64>() / 1000;
		let mut raw_url = ethscan_url::GTE_BLOCK_BY_TIMESTAMP.to_vec();
		raw_url.append(&mut now.to_be_bytes().to_vec());
		raw_url.append(&mut "&closest=before&apikey=".as_bytes().to_vec());
		let mut api_key = T::APIKey::get().unwrap();
		raw_url.append(&mut api_key.clone());

		let current_block_height = Self::json_request(&raw_url, EthScanAPI::GetBlockNoByTime)?.get_object()[2]
			.1
			.get_string()
			.parse::<u64>()
			.map_err(|_| <Error<T>>::U64CF)?;

		debug::trace!("[eth-offchain] current block height: {}", current_block_height);

		// TODO: check current header and skip this run

		let mut raw_url = ethscan_url::GTE_BLOCK_BY_BLOCK_NUMBER.to_vec();
		#[cfg(feature = "std")]
		raw_url.append(&mut current_block_height.to_be_bytes().to_vec());
		raw_url.append(&mut "&boolean=true&apikey=".as_bytes().to_vec());
		raw_url.append(&mut api_key);

		let block_info = Self::json_request(&raw_url, EthScanAPI::GetBlockByNumber)?;
		let eth_header = Self::build_eth_header(current_block_height, block_info)?;

		let call = Call::record_header(block, eth_header);

		let _ = T::SubmitSignedTransaction::submit_signed(call);

		Ok(())
	}
}

fn remove_trascation_and_uncle(r: &mut Vec<u8>) {
	let mut pr = 1266;
	for i in 1266..1632 {
		if r[i] == 91u8 {
			pr = i;
			break;
		}
	}
	let mut tail = r.split_off(pr - 16);
	if tail[tail.len() - 103 - 1] == 93u8 {
		tail = tail.split_off(tail.len() - 103);
		tail.split_off(tail.len() - 15)
	} else if tail[tail.len() - 103 - 68 - 1] == 93u8 {
		tail = tail.split_off(tail.len() - 103 - 68);
		tail.split_off(tail.len() - 15 - 68)
	} else {
		tail = tail.split_off(tail.len() - 103 - 68 * 2 - 1);
		tail.split_off(tail.len() - 15 - 68 * 2 - 1)
	};
	r.append(&mut tail);
	r.push(125u8);
	r.push(125u8);
}
