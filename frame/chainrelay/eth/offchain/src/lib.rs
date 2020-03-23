#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", test))]
mod tests;

extern crate alloc;
use crate::sp_api_hidden_includes_decl_storage::hidden_include::sp_runtime::traits::SaturatedConversion;
use alloc::string::String as AllocString;
use eth_primitives::{header::EthHeader, pow::EthashSeal};
use ethereum_types::H64;
use frame_support::{debug, decl_event, decl_module, decl_storage, dispatch, traits::Get};
use frame_system as system;
use frame_system::offchain::SubmitSignedTransaction;
use hex::FromHex;
#[cfg(not(feature = "std"))]
use num_traits::float::FloatCore;
use pallet_eth_relay::HeaderInfo;
use parity_scale_codec::Encode;
use primitive_types::{H160, H256, U256};
use simple_json::{self, json::JsonValue};
use sp_runtime::{
	offchain::http,
	transaction_validity::{InvalidTransaction, TransactionLongevity, TransactionValidity, ValidTransaction},
	KeyTypeId,
};
use sp_std::{convert::From, prelude::*};

type Result<T> = core::result::Result<T, &'static str>;
type EthScanAPIKey = Option<Vec<u8>>;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ofpf");

pub mod crypto {
	pub use super::KEY_TYPE;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	app_crypto!(sr25519, KEY_TYPE);
}

pub trait Trait: pallet_timestamp::Trait + pallet_eth_relay::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
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

decl_storage! {
  trait Store for Module<T: Trait> as EthOffchain {
  }
}

decl_module! {
pub struct Module<T: Trait> for enum Call where origin: T::Origin {
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
				debug::error!("[eth-offchain] Error: {:}", e);
			}
		}
	}
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

fn json_request(raw_url: &Vec<u8>, api_type: EthScanAPI) -> Result<JsonValue> {
	let url = core::str::from_utf8(raw_url).map_err(|_| "url decode error")?;
	debug::trace!("[eth-offchain] request: {:?}", url);

	let pending = http::Request::get(&url)
		.send()
		.map_err(|_| "Error in sending http GET request")?;

	let mut response = pending.wait().map_err(|_| "Error in waiting http response back")?;

	let mut retry_time = 0;
	let mut r: Vec<u8>;
	loop {
		if response.code != 200 {
			if retry_time == 3 {
				debug::warn!("Unexpected status code: {}", response.code);
				return Err("Non-200 status code returned from http request");
			}
			response = http::Request::get(&url)
				.send()
				.map_err(|_| "Error in sending http GET request")?
				.wait()
				.map_err(|_| "Error in waiting http response back")?;
			retry_time += 1;
			debug::info!("[eth-offchain] retry {} times", retry_time);
		} else {
			r = response.body().collect::<Vec<u8>>();
			if r[0] == 123u8 {
				break;
			}
		}
		// TODO: figure out how to sleep in no-std here
	}

	debug::trace!("[eth-offchain] response: {:?}", core::str::from_utf8(&r));

	let json_val: JsonValue = match api_type {
		EthScanAPI::GetBlockByNumber => {
			if r.len() < 1362 {
				debug::warn!("[eth-offchain] response: {:?}", core::str::from_utf8(&r));
				return Err("unexpected api response");
			}
			remove_trascation_and_uncle(&mut r);
			// get the result part
			simple_json::parse_json(
				&core::str::from_utf8(&r[33..r.len() - 1]).map_err(|_| "result part cannot convert to string")?,
			)
			.map_err(|_| "JSON parsing error")?
		}
		_ => simple_json::parse_json(&core::str::from_utf8(&r).map_err(|_| "JSON result cannot convert to string")?)
			.map_err(|_| "JSON parsing error")?,
	};

	Ok(json_val)
}

fn hexstr_padding(width: usize, content: AllocString) -> AllocString {
	if content.len() < width {
		let mut output: Vec<u8> = Vec::new();
		for _ in 0..(width - content.len() + 2) {
			output.push(48);
		}
		output.append(&mut content.into_bytes()[2..].to_vec());
		return output.iter().map(|u| *u as char).collect::<AllocString>();
	}
	content
}

fn build_eth_header(number: u64, block_info: JsonValue) -> Result<EthHeader> {
	let parent_hash = &block_info.get_object()[10].1.get_string()[2..];
	let timestamp_hexstr = block_info.get_object()[15].1.get_string();
	let author = &block_info.get_object()[6].1.get_string()[2..];
	let uncles_hash = &block_info.get_object()[12].1.get_string()[2..];
	let extra_data = &block_info.get_object()[1].1.get_string()[2..];
	let state_root = &block_info.get_object()[14].1.get_string()[2..];
	let receipts_root = &block_info.get_object()[11].1.get_string()[2..];
	let bloom = &block_info.get_object()[5].1.get_string()[2..];
	let gas_used = &hexstr_padding(64, block_info.get_object()[3].1.get_string());
	let gas_limit = &hexstr_padding(64, block_info.get_object()[2].1.get_string());
	let difficulty = &hexstr_padding(64, block_info.get_object()[0].1.get_string());
	let seal = build_eth_seal(
		block_info.get_object()[7].1.get_string(),
		block_info.get_object()[8].1.get_string(),
	)?;
	let transactions_root = &block_info.get_object()[17].1.get_string()[2..];
	let hash = &block_info.get_object()[4].1.get_string()[2..];

	let h = EthHeader {
		parent_hash: H256::from(<[u8; 32]>::from_hex(parent_hash).expect("parent hash decoding failed")),
		timestamp: u64::from_str_radix(timestamp_hexstr.trim_start_matches("0x"), 16).unwrap(),
		number,
		author: H160::from(<[u8; 20]>::from_hex(author).expect("author decoding failed")),
		transactions_root: H256::from(
			<[u8; 32]>::from_hex(transactions_root).expect("transactions root decoding failed"),
		),
		uncles_hash: H256::from(<[u8; 32]>::from_hex(uncles_hash).expect("uncles hash hash decoding failed")),
		state_root: H256::from(<[u8; 32]>::from_hex(state_root).expect("state root decoding failed")),
		receipts_root: H256::from(<[u8; 32]>::from_hex(receipts_root).expect("receipts root decoding failed")),
		gas_used: U256::from(<[u8; 32]>::from_hex(gas_used).expect("gas used root decoding failed")),
		gas_limit: U256::from(<[u8; 32]>::from_hex(gas_limit).expect("gas limit root decoding failed")),
		difficulty: U256::from(<[u8; 32]>::from_hex(difficulty).expect("difficulty decoding failed")),
		seal: vec![rlp::encode(&seal.mix_hash).to_vec(), rlp::encode(&seal.nonce).to_vec()],
		hash: Some(H256::from(<[u8; 32]>::from_hex(hash).expect("hash decoding failed"))),
		extra_data: <Vec<u8>>::from_hex(extra_data).expect("extra data decoding failed"),
		log_bloom: <[u8; 256]>::from_hex(bloom).expect("hash decoding failed").into(),
	};
	Ok(h)
}

// TODO: we may store the eth header info on chain install of all eth headers
fn _build_eth_header_info(
	block_height: u64,
	total_difficulty_hexstr: AllocString,
	parent_hash_hexstr: AllocString,
) -> Result<HeaderInfo> {
	let total_difficulty = hexstr_padding(64, total_difficulty_hexstr);
	let parent_hash = &parent_hash_hexstr[2..];
	let h = HeaderInfo {
		number: block_height,
		total_difficulty: U256::from(<[u8; 32]>::from_hex(total_difficulty).expect("Total difficulty decoding failed")),
		parent_hash: H256::from(<[u8; 32]>::from_hex(parent_hash).expect("parent hash decoding failed")),
	};
	Ok(h)
}

fn build_eth_seal(mix_hash_hexstr: AllocString, nonce_hexstr: AllocString) -> Result<EthashSeal> {
	let mix_hash = &mix_hash_hexstr[2..];
	let nonce = &nonce_hexstr[2..];
	let s = EthashSeal {
		mix_hash: H256::from(<[u8; 32]>::from_hex(mix_hash).expect("Total difficulty decoding failed")),
		nonce: H64::from(<[u8; 8]>::from_hex(nonce).expect("Nonce decoding failed")),
	};
	Ok(s)
}

impl<T: Trait> Module<T> {
	fn fetch_eth_header<'a>(block: T::BlockNumber) -> Result<()> {
		let now = <pallet_timestamp::Module<T>>::get().saturated_into::<usize>() / 1000;
		let mut raw_url = "https://api.etherscan.io/api?module=block&action=getblocknobytime&timestamp="
			.as_bytes()
			.to_vec();
		debug::trace!("[eth-offchain] now: {}", now);
		#[cfg(feature = "std")]
		raw_url.append(&mut now.to_string().as_bytes().to_vec());
		raw_url.append(&mut "&closest=before&apikey=".as_bytes().to_vec());
		let mut api_key = T::APIKey::get().unwrap();
		debug::error!("[eth-offchain] api_key: {:?}", api_key);
		raw_url.append(&mut api_key.clone());

		let current_block_height = json_request(&raw_url, EthScanAPI::GetBlockNoByTime)?.get_object()[2]
			.1
			.get_string()
			.parse::<u64>()
			.map_err(|_| "fetch current block height error: parsing to u64 error")?;

		debug::trace!("[eth-offchain] current block height: {:?}", current_block_height);

		// TODO: check current header and skip this run

		let mut raw_url = "https://api.etherscan.io/api?module=proxy&action=eth_getBlockByNumber&tag=0x"
			.as_bytes()
			.to_vec();
		#[cfg(feature = "std")]
		raw_url.append(&mut format!("{:x}", current_block_height).as_bytes().to_vec());
		raw_url.append(&mut "&boolean=true&apikey=".as_bytes().to_vec());
		raw_url.append(&mut api_key);

		let block_info = json_request(&raw_url, EthScanAPI::GetBlockByNumber)?;
		let eth_header = build_eth_header(current_block_height, block_info)?;

		let call = Call::record_header(block, eth_header);

		let _ = T::SubmitSignedTransaction::submit_signed(call);

		Ok(())
	}
}

#[allow(deprecated)]
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;

	#[allow(deprecated)]
	fn validate_unsigned(call: &Self::Call) -> TransactionValidity {
		match call {
			Call::record_header(block, _eth_header) => Ok(ValidTransaction {
				priority: 0,
				requires: vec![],
				provides: vec![(block).encode()],
				longevity: TransactionLongevity::max_value(),
				propagate: true,
			}),
			_ => InvalidTransaction::Call.into(),
		}
	}
}
