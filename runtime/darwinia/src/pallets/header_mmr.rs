// --- darwinia-network ---
use crate::*;
use darwinia_header_mmr::Config;

impl Config for Runtime {
	type WeightInfo = ();

	const INDEXING_PREFIX: &'static [u8] = b"header-mmr-";
}
