// std
use std::{
	fs::File,
	io::{Read, Write},
	marker::PhantomData,
	mem,
	path::Path,
	sync::RwLock,
};
// darwinia
use crate::*;
// crates.io
use anyhow::Result;
use fxhash::FxHashMap;
use once_cell::sync::Lazy;
use parity_scale_codec::{Decode, Encode};
use serde::de::DeserializeOwned;
use serde_json::Value;
// hack-ink
use subspector::ChainSpec;

pub type Map<V> = FxHashMap<String, V>;

pub static NOW: Lazy<RwLock<u32>> = Lazy::new(|| RwLock::new(0));

pub struct Processor<S> {
	pub solo_state: State<S>,
	pub para_state: State<()>,
	pub shell_state: State<()>,
	pub shell_chain_spec: ChainSpec,
	pub test: bool,
}
impl<S> Processor<S>
where
	S: Configurable,
{
	pub fn new() -> Result<Self> {
		build_spec(S::NAME)?;

		let mut shell_chain_spec = from_file::<ChainSpec>(&format!("data/{}-shell.json", S::NAME))?;
		let solo_path = format!("data/{}-solo.json", S::NAME);
		let para_path = format!("data/{}-para.json", S::NAME);

		if !Path::new(&solo_path).is_file() || !Path::new(&para_path).is_file() {
			download_specs(S::NAME)?;
		}

		Ok(Self {
			solo_state: State::from_file(&solo_path)?,
			para_state: State::from_file(&para_path)?,
			shell_state: State {
				map: mem::take(&mut shell_chain_spec.genesis.raw.top),
				_runtime: Default::default(),
			},
			shell_chain_spec,
			test: false,
		})
	}

	pub fn test(mut self) -> Self {
		self.test = true;

		self
	}

	pub fn process(mut self) -> Result<()> {
		self.solo_state.get_value(b"System", b"Number", "", &mut *NOW.write().unwrap());

		let _guard = NOW.read().unwrap();

		assert!(*_guard != 0);

		self.process_system()
			.process_indices()
			.process_vesting()
			.process_proxy()
			.process_staking()
			.process_evm();

		self.save()
	}

	pub fn save(mut self) -> Result<()> {
		log::info!("saving processed chain spec");

		mem::swap(&mut self.shell_state.map, &mut self.shell_chain_spec.genesis.raw.top);

		if self.test {
			self.shell_chain_spec.chain_type = "Local".into();
			self.shell_chain_spec.extensions["relay_chain"] = Value::String("rococo-local".into());
		}

		let mut f = File::create(format!("data/{}-processed.json", S::NAME))?;
		let v = serde_json::to_vec(&self.shell_chain_spec)?;

		f.write_all(&v)?;

		Ok(())
	}
}

pub struct State<R> {
	pub map: Map<String>,
	_runtime: PhantomData<R>,
}
impl<R> State<R> {
	pub fn from_file(path: &str) -> Result<Self> {
		Ok(Self {
			map: from_file::<ChainSpec>(path)?.genesis.raw.top,
			_runtime: <PhantomData<R>>::default(),
		})
	}

	pub fn insert_raw_key_raw_value(&mut self, key: String, value: String) -> &mut Self {
		self.map.insert(key, value);

		self
	}

	pub fn insert_raw_key_value<E>(&mut self, key: String, value: E) -> &mut Self
	where
		E: Encode,
	{
		self.map.insert(key, encode_value(value));

		self
	}

	pub fn take_raw_map<F>(
		&mut self,
		prefix: &str,
		buffer: &mut Map<String>,
		process_key: F,
	) -> &mut Self
	where
		F: Fn(&str, &str) -> String,
	{
		self.map.retain(|k, v| {
			if k.starts_with(prefix) {
				buffer.insert(process_key(k, prefix), v.to_owned());

				false
			} else {
				true
			}
		});

		self
	}

	pub fn insert_raw_key_map(&mut self, pairs: Map<String>) -> &mut Self {
		pairs.into_iter().for_each(|(k, v)| {
			if self.map.contains_key(&k) {
				log::error!("key({k}) has already existed, overriding");
			}

			self.map.insert(k, v);
		});

		self
	}

	pub fn get_value<D>(&self, pallet: &[u8], item: &[u8], hash: &str, value: &mut D) -> &Self
	where
		D: Decode,
	{
		let key = full_key(pallet, item, hash);

		if let Some(v) = self.map.get(&key) {
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

	pub fn take_value<D>(
		&mut self,
		pallet: &[u8],
		item: &[u8],
		hash: &str,
		value: &mut D,
	) -> &mut Self
	where
		D: Decode,
	{
		let key = full_key(pallet, item, hash);

		if let Some(v) = self.map.remove(&key) {
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

	pub fn insert_value<E>(&mut self, pallet: &[u8], item: &[u8], hash: &str, value: E) -> &mut Self
	where
		E: Encode,
	{
		self.map.insert(full_key(pallet, item, hash), encode_value(value));

		self
	}

	pub fn mutate_value<D, F>(&mut self, pallet: &[u8], item: &[u8], hash: &str, f: F) -> &mut Self
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

	pub fn take_map<D, F>(
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

		self.map.retain(|full_key, v| {
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

	pub fn insert_map<E, F>(&mut self, pairs: Map<E>, process_key: F) -> &mut Self
	where
		E: Encode,
		F: Fn(&str) -> String,
	{
		pairs.into_iter().for_each(|(k, v)| {
			self.map.insert(process_key(&k), encode_value(v));
		});

		self
	}

	pub fn contains_key(&self, key: &str) -> bool {
		self.map.contains_key(key)
	}

	pub fn starts_with(&self, key: &str) -> bool {
		self.map.keys().into_iter().any(|k| k.starts_with(key))
	}

	// pub fn inc_consumers(&mut self, who: &str) {}

	// pub fn transfer(&mut self, from: &str, to: &str, amount: u128) {}

	pub fn unreserve<A>(&mut self, account_id_32: A, amount: u128)
	where
		A: AsRef<[u8]>,
	{
		let account_id_32 = account_id_32.as_ref();
		let (p, i, h) = if is_evm_address(account_id_32) {
			(&b"System"[..], &b"Account"[..], &account_id_32[11..31])
		} else {
			(&b"AccountMigration"[..], &b"Accounts"[..], account_id_32)
		};

		self.mutate_value(p, i, &blake2_128_concat_to_string(h), |a: &mut AccountInfo| {
			a.data.free += amount;
			a.data.reserved -= amount;
		});
	}
}

pub fn from_file<D>(path: &str) -> Result<D>
where
	D: DeserializeOwned,
{
	log::info!("load data from {path:?}");

	let mut f = File::open(path)?;
	let mut v = Vec::new();

	f.read_to_end(&mut v)?;

	Ok(serde_json::from_slice(&v)?)
}
