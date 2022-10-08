#[allow(unused)]
use {
	crate::*,
	frame_support::{migration, traits::OnRuntimeUpgrade, weights::Weight},
	sp_std::prelude::*,
};

pub struct CustomOnRuntimeUpgrade;
impl OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		migrate()
	}
}

fn migrate() -> Weight {
	let storages: &[(&[u8], &[&[u8]])] = &[
		// TODO: check if we have left some balances locks
		(
			b"EcdsaRelayAuthority",
			&[
				b"Candidates",
				b"Authorities",
				b"NextAuthorities",
				b"NextTerm",
				b"AuthoritiesToSign",
				b"MmrRootsToSignKeys",
				b"MmrRootsToSign",
				b"SubmitDuration",
			],
		),
		(
			// FrameV1 name.
			b"DarwiniaEthereumRelay",
			&[
				b"ConfirmedHeaderParcels",
				b"ConfirmedBlockNumbers",
				b"BestConfirmedBlockNumber",
				b"ConfirmedDepth",
				b"DagsMerkleRoots",
				b"ReceiptVerifyFee",
				b"PendingRelayHeaderParcels",
			],
		),
		// TODO: check if we have left some balances locks
		(
			// FrameV1 name.
			b"Instance1DarwiniaRelayerGame",
			&[
				b"RelayHeaderParcelToResolve",
				b"Affirmations",
				b"BestConfirmedHeaderId",
				b"RoundCounts",
				b"AffirmTime",
				b"GamesToUpdate",
				b"Stakes",
				b"GameSamplePoints",
			],
		),
		(
			b"EthereumBacking",
			&[
				b"TokenRedeemAddress",
				b"DepositRedeemAddress",
				b"SetAuthoritiesAddress",
				b"RingTokenAddress",
				b"KtonTokenAddress",
				b"RedeemStatus",
				b"LockAssetEvents",
			],
		),
	];
	storages.iter().for_each(|(module, items)| {
		items.iter().for_each(|item| migration::remove_storage_prefix(module, item, &[]))
	});

	// Helix DAO multisig address:
	// - 0xBd1a110ec476b4775c43905000288881367B1a88
	// - 2qSbd2umtD4KmV2X89YfqmCQgDraEabAaLNFiR96xUJ1m31G
	// - 0x64766d3a00000000000000bd1a110ec476b4775c43905000288881367b1a88ad
	let dest = array_bytes::hex_into_unchecked::<AccountId, 32>(
		"0x64766d3a00000000000000bd1a110ec476b4775c43905000288881367b1a88ad",
	);
	// Ethereum backing account:
	// - modlda/ethbk
	// - 2qeMxq616BhqvTW8a1bp2g7VKPAmpda1vXuAAz5TxV5ehivG
	// - 0x6d6f646c64612f657468626b0000000000000000000000000000000000000000
	let origin = array_bytes::hex_into_unchecked::<AccountId, 32>(
		"0x6d6f646c64612f657468626b0000000000000000000000000000000000000000",
	);
	<darwinia_balances::Pallet<Runtime, RingInstance>>::transfer_all(
		Origin::signed(origin.clone()),
		dest.clone().into(),
		false,
	);
	<darwinia_balances::Pallet<Runtime, KtonInstance>>::transfer_all(
		Origin::signed(origin),
		dest.into(),
		false,
	);

	// 0
	RuntimeBlockWeights::get().max_block
}
