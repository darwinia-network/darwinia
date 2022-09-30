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
	[
		// TODO: check if we have left some balances locks
		(
			b"EcdsaRelayAuthority",
			[
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
			[
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
			b"DarwiniaRelayerGameInstance1",
			[
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
			[
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

	// 0
	RuntimeBlockWeights::get().max_block
}
