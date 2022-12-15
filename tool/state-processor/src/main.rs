mod balances;
mod evm;
mod system;
mod vesting;

mod type_registry;
use type_registry::*;

// std
use std::{
	env,
	fs::File,
	io::{Read, Write},
	mem,
};
// crates.io
use anyhow::Result;
use fxhash::FxHashMap;
use parity_scale_codec::{Decode, Encode};
use serde::de::DeserializeOwned;
// hack-ink
use subspector::ChainSpec;

type Map<V> = FxHashMap<String, V>;

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
		self.process_system().process_vesting().process_evm();

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

	#[allow(unused)]
	fn prune(&mut self, pallet: &[u8], items: Option<&[&[u8]]>) -> &mut Self {
		// Prune specific storages.
		if let Some(items) = items {
			for item in items {
				let k = item_key(pallet, item);

				self.0.remove(&k).or_else(|| {
					log::warn!(
						"`{}::{}: {k}` not found",
						String::from_utf8_lossy(pallet),
						String::from_utf8_lossy(item)
					);

					None
				});
			}
		}
		// Prune entire pallet.
		else {
			let prefix = pallet_key(pallet);
			let mut pruned = false;

			self.0.retain(|full_key, _| {
				if full_key.starts_with(&prefix) {
					pruned = true;

					false
				} else {
					true
				}
			});

			if !pruned {
				log::warn!("`{}: {prefix}` not found", String::from_utf8_lossy(pallet));
			}
		}

		self
	}

	fn take_raw<F>(
		&mut self,
		prefix: &str,
		buffer: &mut Map<String>,
		preprocess_key: F,
	) -> &mut Self
	where
		F: Fn(&str, &str) -> String,
	{
		self.0.retain(|k, v| {
			if k.starts_with(prefix) {
				buffer.insert(preprocess_key(k, prefix), v.to_owned());

				false
			} else {
				true
			}
		});

		self
	}

	fn insert_raw(&mut self, pairs: Map<String>) -> &mut Self {
		pairs.into_iter().for_each(|(k, v)| {
			if self.0.contains_key(&k) {
				log::error!("key({k}) has already existed, overriding");
			}

			self.0.insert(k, v);
		});

		self
	}

	fn take_value<D>(&mut self, pallet: &[u8], item: &[u8], value: &mut D) -> &mut Self
	where
		D: Decode,
	{
		let key = item_key(pallet, item);

		if let Some(v) = self.0.remove(&key) {
			match decode(&v) {
				Ok(v) => *value = v,
				Err(e) => log::warn!("failed to decode `{key}:{v}`, due to `{e}`"),
			}
		}

		self
	}

	fn take_map<D, F>(
		&mut self,
		pallet: &[u8],
		item: &[u8],
		buffer: &mut Map<D>,
		preprocess_key: F,
	) -> &mut Self
	where
		D: Decode,
		F: Fn(&str, &str) -> String,
	{
		let len = buffer.len();
		let item_key = item_key(pallet, item);

		self.0.retain(|full_key, v| {
			if full_key.starts_with(&item_key) {
				match decode(v) {
					Ok(v) => {
						buffer.insert(preprocess_key(full_key, &item_key), v);
					},
					Err(e) => log::warn!("failed to decode `{full_key}:{v}`, due to `{e}`"),
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

fn pallet_key(pallet: &[u8]) -> String {
	let prefix = subhasher::twox128(pallet);

	array_bytes::bytes2hex("0x", prefix)
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

fn identity(key: &str, _: &str) -> String {
	key.into()
}

fn replace_first_match(key: &str, from: &str, to: &str) -> String {
	key.replacen(from, to, 1)
}
