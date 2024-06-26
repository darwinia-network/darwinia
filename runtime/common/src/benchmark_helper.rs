// crates.io
use codec::Encode;
// darwinia
use dc_primitives::*;
// polkadot-sdk
use sp_core::crypto::FromEntropy;

/// Helper for pallet-assets benchmarking.
pub enum Assets {}
impl pallet_assets::BenchmarkHelper<codec::Compact<u64>> for Assets {
	fn create_asset_id_parameter(id: u32) -> codec::Compact<u64> {
		u64::from(id).into()
	}
}

pub enum Treasury {}
impl<AssetKind> pallet_treasury::ArgumentsFactory<AssetKind, AccountId> for Treasury
where
	AssetKind: FromEntropy,
{
	fn create_asset_kind(seed: u32) -> AssetKind {
		AssetKind::from_entropy(&mut seed.encode().as_slice()).unwrap()
	}

	fn create_beneficiary(seed: [u8; 32]) -> AccountId {
		<[u8; 20]>::from_entropy(&mut seed.as_slice()).unwrap().into()
	}
}
