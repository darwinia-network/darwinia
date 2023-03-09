// std
use std::{
	fs::File,
	ops::Range,
	path::Path,
	process::{Command, Stdio},
};
// crates.io
use parity_scale_codec::{Decode, Encode};
use tar::Archive;
use zstd::Decoder;
// darwinia
use crate::*;

pub fn item_key(pallet: &[u8], item: &[u8]) -> String {
	let k = substorager::storage_key(pallet, item);

	array_bytes::bytes2hex("0x", k.0)
}

pub fn full_key(pallet: &[u8], item: &[u8], hash: &str) -> String {
	format!("{}{hash}", item_key(pallet, item))
}

pub fn encode_value<V>(v: V) -> String
where
	V: Encode,
{
	array_bytes::bytes2hex("0x", v.encode())
}

pub fn decode<D>(hex: &str) -> Result<D>
where
	D: Decode,
{
	let v = array_bytes::hex2bytes(hex).map_err(|e| anyhow::anyhow!("{e:?}"))?;

	Ok(D::decode(&mut &*v)?)
}

pub fn to_0x(s: &str) -> String {
	format!("0x{s}")
}

pub fn strip_prefix_key(key: &str, prefix: &str) -> String {
	key.strip_prefix(prefix).unwrap().into()
}

// twox128(pallet) + twox128(item) + *(item_key) -> *(item_key)
pub fn get_hashed_key(full_key: &str, item_key: &str) -> String {
	full_key.trim_start_matches(item_key).into()
}

// twox128(pallet) + twox128(item) + *_concat(n) -> n
pub fn get_last_n_key(key: &str, _: &str, n: usize) -> String {
	get_last_n(key, n)
}

// twox128(pallet) + twox128(item) + *_concat(n) -> n
pub fn get_last_n(key: &str, n: usize) -> String {
	to_0x(&key[key.len() - n..])
}

// twox128(pallet) + twox128(item) + *_concat(u32) -> u32
pub fn get_last_8_key(key: &str, _: &str) -> String {
	get_last_n(key, 8)
}

// twox128(pallet) + twox128(item) + *_concat(account_id_32) -> account_id_32
pub fn get_last_64_key(key: &str, _: &str) -> String {
	get_last_64(key)
}

// twox128(pallet) + twox128(item) + *_concat(account_id_32) -> account_id_32
pub fn get_last_64(key: &str) -> String {
	get_last_n(key, 64)
}

pub fn blake2_128_concat_to_string<D>(data: D) -> String
where
	D: AsRef<[u8]>,
{
	array_bytes::bytes2hex("", subhasher::blake2_128_concat(data))
}

pub fn is_evm_address(address: &[u8]) -> bool {
	address.starts_with(b"dvm:")
		&& address[1..31].iter().fold(address[0], |checksum, &b| checksum ^ b) == address[31]
}

pub fn build_double_map<T>(
	map: Map<T>,
	k1_range: Range<usize>,
	k2_range: Range<usize>,
) -> Map<Map<T>>
where
	T: Clone,
{
	map.into_iter().fold(Map::default(), |mut double_map, (k, v)| {
		let k1 = to_0x(&k[k1_range.clone()]);
		let k2 = to_0x(&k[k2_range.clone()]);

		double_map
			.entry(k1)
			.and_modify(|m| {
				m.insert(k2.clone(), v.clone());
			})
			.or_insert_with(|| Map::from_iter([(k2, v)]));

		double_map
	})
}

pub fn build_spec(chain: &str) -> Result<()> {
	let mut path = "../../target/release/darwinia";

	if !Path::new(path).is_file() {
		path = "../../target/x86_64-unknown-linux-gnu/release/darwinia";
	}

	log::info!("run `{path} build-spec --raw --disable-default-bootnode --chain {chain}-genesis`");

	Command::new(path)
		.args([
			"build-spec",
			"--raw",
			"--disable-default-bootnode",
			"--chain",
			&format!("{chain}-genesis"),
		])
		.stdout(Stdio::from(File::create(format!("data/{chain}-shell.json"))?))
		.output()?;

	Ok(())
}

pub fn download_specs(chain: &str) -> Result<()> {
	log::info!("download {chain} spec");

	let decoder = Decoder::new(
		ureq::get(&format!(
			"https://github.com/darwinia-network/darwinia/releases/download/{chain}2/{chain}-state.tar.zst"
		))
		.call()?
		.into_reader(),
	)?;

	for e in Archive::new(decoder).entries()? {
		let mut e = e?;

		e.unpack(format!("data/{}", e.path()?.to_string_lossy()))?;
	}

	Ok(())
}
