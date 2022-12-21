mod balances;
mod evm;
mod indices;
mod staking;
mod system;
mod vesting;

mod adjust;
use adjust::*;

mod type_registry;
use type_registry::*;

// std
use std::{
	env,
	fs::File,
	io::{Read, Write},
	mem,
	sync::RwLock,
};
// crates.io
use anyhow::Result;
use fxhash::FxHashMap;
use once_cell::sync::Lazy;
use parity_scale_codec::{Decode, Encode};
use serde::de::DeserializeOwned;
// hack-ink
use subspector::ChainSpec;

type Map<V> = FxHashMap<String, V>;

static NOW: Lazy<RwLock<u32>> = Lazy::new(|| RwLock::new(0));

fn main() -> Result<()> {
	env::set_var("RUST_LOG", "state_processor");
	pretty_env_logger::init();

	Processor::new()?.process()?;

	Ok(())
}

struct Processor {
	solo_state: State,
	para_state: State,
	shell_state: State,
	shell_chain_spec: ChainSpec,
}
impl Processor {
	fn new() -> Result<Self> {
		let mut shell_chain_spec = from_file::<ChainSpec>("test-data/shell.json")?;

		Ok(Self {
			solo_state: State::from_file("test-data/solo.json")?,
			para_state: State::from_file("test-data/para.json")?,
			shell_state: State(mem::take(&mut shell_chain_spec.genesis.raw.top)),
			shell_chain_spec,
		})
	}

	fn process(mut self) -> Result<()> {
		self.solo_state.get_value(b"System", b"Number", "", &mut *NOW.write().unwrap());

		let _guard = NOW.read().unwrap();

		assert!(*_guard != 0);

		self.process_system().process_indices().process_vesting().process_staking().process_evm();

		self.save()
	}

	fn save(mut self) -> Result<()> {
		log::info!("saving processed chain spec");

		mem::swap(&mut self.shell_state.0, &mut self.shell_chain_spec.genesis.raw.top);

		let mut f = File::create("test-data/processed.json")?;
		let v = serde_json::to_vec(&self.shell_chain_spec)?;

		f.write_all(&v)?;

		Ok(())
	}
}

struct State(Map<String>);
impl State {
	fn from_file(path: &str) -> Result<Self> {
		Ok(Self(from_file::<ChainSpec>(path)?.genesis.raw.top))
	}

	fn insert_raw_key_raw_value(&mut self, key: String, value: String) -> &mut Self {
		self.0.insert(key, value);

		self
	}

	fn insert_raw_key_value<E>(&mut self, key: String, value: E) -> &mut Self
	where
		E: Encode,
	{
		self.0.insert(key, encode_value(value));

		self
	}

	fn take_raw_map<F>(
		&mut self,
		prefix: &str,
		buffer: &mut Map<String>,
		process_key: F,
	) -> &mut Self
	where
		F: Fn(&str, &str) -> String,
	{
		self.0.retain(|k, v| {
			if k.starts_with(prefix) {
				buffer.insert(process_key(k, prefix), v.to_owned());

				false
			} else {
				true
			}
		});

		self
	}

	fn insert_raw_key_map(&mut self, pairs: Map<String>) -> &mut Self {
		pairs.into_iter().for_each(|(k, v)| {
			if self.0.contains_key(&k) {
				log::error!("key({k}) has already existed, overriding");
			}

			self.0.insert(k, v);
		});

		self
	}

	fn get_value<D>(&self, pallet: &[u8], item: &[u8], hash: &str, value: &mut D) -> &Self
	where
		D: Decode,
	{
		let key = full_key(pallet, item, hash);

		if let Some(v) = self.0.get(&key) {
			match decode(v) {
				Ok(v) => *value = v,
				Err(e) => log::error!(
					"failed to decode `{}::{}::{hash}({v})`, due to `{e}`",
					String::from_utf8_lossy(pallet),
					String::from_utf8_lossy(item),
				),
			}
		} else {
			log::error!(
				"key not found `{}::{}::{hash}`",
				String::from_utf8_lossy(pallet),
				String::from_utf8_lossy(item),
			);
		}

		self
	}

	fn take_value<D>(&mut self, pallet: &[u8], item: &[u8], hash: &str, value: &mut D) -> &mut Self
	where
		D: Decode,
	{
		let key = full_key(pallet, item, hash);

		if let Some(v) = self.0.remove(&key) {
			match decode(&v) {
				Ok(v) => *value = v,
				Err(e) => log::error!(
					"failed to decode `{}::{}::{hash}({v})`, due to `{e}`",
					String::from_utf8_lossy(pallet),
					String::from_utf8_lossy(item)
				),
			}
		} else {
			log::error!(
				"key not found `{}::{}::{hash}`",
				String::from_utf8_lossy(pallet),
				String::from_utf8_lossy(item),
			);
		}

		self
	}

	fn insert_value<E>(&mut self, pallet: &[u8], item: &[u8], hash: &str, value: E) -> &mut Self
	where
		E: Encode,
	{
		self.0.insert(full_key(pallet, item, hash), encode_value(value));

		self
	}

	fn mutate_value<D, F>(&mut self, pallet: &[u8], item: &[u8], hash: &str, f: F) -> &mut Self
	where
		D: Default + Encode + Decode,
		F: FnOnce(&mut D),
	{
		let mut v = D::default();

		self.get_value(pallet, item, hash, &mut v);

		f(&mut v);

		self.insert_value(pallet, item, hash, v);

		self
	}

	fn take_map<D, F>(
		&mut self,
		pallet: &[u8],
		item: &[u8],
		buffer: &mut Map<D>,
		process_key: F,
	) -> &mut Self
	where
		D: Decode,
		F: Fn(&str, &str) -> String,
	{
		let len = buffer.len();
		let prefix = item_key(pallet, item);

		self.0.retain(|full_key, v| {
			if full_key.starts_with(&prefix) {
				match decode(v) {
					Ok(v) => {
						buffer.insert(process_key(full_key, &prefix), v);
					},
					Err(e) => log::error!("failed to decode `{full_key}:{v}`, due to `{e}`"),
				}

				false
			} else {
				true
			}
		});

		if buffer.len() == len {
			log::info!(
				"no new item inserted for {}::{}",
				String::from_utf8_lossy(pallet),
				String::from_utf8_lossy(item)
			);
		}

		self
	}

	fn insert_map<E, F>(&mut self, pairs: Map<E>, process_key: F) -> &mut Self
	where
		E: Encode,
		F: Fn(&str) -> String,
	{
		pairs.into_iter().for_each(|(k, v)| {
			self.0.insert(process_key(&k), encode_value(v));
		});

		self
	}

	// fn inc_consumers(&mut self, who: &str) {}

	// fn transfer(&mut self, from: &str, to: &str, amount: u128) {}

	fn unreserve<A>(&mut self, who: A, amount: u128)
	where
		A: AsRef<[u8]>,
	{
		self.mutate_value(
			b"System",
			b"Account",
			&blake2_128_concat_to_string(who),
			|a: &mut AccountInfo| {
				a.data.free += amount;
				a.data.reserved -= amount;
			},
		);
	}
}

fn from_file<D>(path: &str) -> Result<D>
where
	D: DeserializeOwned,
{
	log::info!("load data from {path:?}");

	let mut f = File::open(path)?;
	let mut v = Vec::new();

	f.read_to_end(&mut v)?;

	Ok(serde_json::from_slice(&v)?)
}

fn item_key(pallet: &[u8], item: &[u8]) -> String {
	let k = substorager::storage_key(pallet, item);

	array_bytes::bytes2hex("0x", &k.0)
}

fn full_key(pallet: &[u8], item: &[u8], hash: &str) -> String {
	format!("{}{hash}", item_key(pallet, item))
}

fn encode_value<V>(v: V) -> String
where
	V: Encode,
{
	array_bytes::bytes2hex("0x", &v.encode())
}

fn decode<D>(hex: &str) -> Result<D>
where
	D: Decode,
{
	let v = array_bytes::hex2bytes(hex).map_err(|e| anyhow::anyhow!("{e:?}"))?;

	Ok(D::decode(&mut &*v)?)
}

// twox128(pallet) + twox128(item) -> twox128(pallet) + twox128(item)
fn get_identity_key(key: &str, _: &str) -> String {
	key.into()
}

// twox128(pallet) + twox128(item) + *(item_key) -> *(item_key)
fn get_hashed_key(full_key: &str, item_key: &str) -> String {
	full_key.trim_start_matches(item_key).into()
}

// twox128(pallet) + twox128(item) + *_concat(account_id_32) -> account_id_32
fn get_last_64(key: &str) -> String {
	format!("0x{}", &key[key.len() - 64..])
}

fn replace_first_match(key: &str, from: &str, to: &str) -> String {
	key.replacen(from, to, 1)
}

fn blake2_128_concat_to_string<D>(data: D) -> String
where
	D: AsRef<[u8]>,
{
	array_bytes::bytes2hex("", subhasher::blake2_128_concat(data))
}
