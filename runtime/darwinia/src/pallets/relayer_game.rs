pub use darwinia_relayer_game::Instance1 as EthereumRelayerGameInstance;

// --- paritytech ---
use frame_support::traits::LockIdentifier;
// --- darwinia-network ---
use crate::*;
use darwinia_relay_primitives::AdjustableRelayerGame;
use darwinia_relayer_game::Config;
use ethereum_primitives::EthereumBlockNumber;

pub struct EthereumRelayerGameAdjustor;
impl AdjustableRelayerGame for EthereumRelayerGameAdjustor {
	type Moment = BlockNumber;
	type Balance = Balance;
	type RelayHeaderId = EthereumBlockNumber;

	fn max_active_games() -> u8 {
		32
	}

	fn affirm_time(round: u32) -> Self::Moment {
		match round {
			// 3 mins
			0 => 30,
			// 1.5 mins
			_ => 15,
		}
	}

	fn complete_proofs_time(_: u32) -> Self::Moment {
		// 3 mins
		30
	}

	fn update_sample_points(sample_points: &mut Vec<Vec<Self::RelayHeaderId>>) {
		if let Some(last_round_sample_points) = sample_points.last() {
			if let Some(last_sample_point) = last_round_sample_points.last() {
				let new_sample_points = vec![*last_sample_point - 1];

				sample_points.push(new_sample_points);
			} else {
				// Should never be reached
				log::error!(target: "ethereum-relayer-game", "Sample Round - NOT EXISTED");
			}
		} else {
			// Should never be reached
			log::error!(target: "ethereum-relayer-game", "Sample Point - NOT EXISTED");
		}
	}

	fn estimate_stake(round: u32, affirmations_count: u32) -> Self::Balance {
		match round {
			0 => match affirmations_count {
				0 => 100 * COIN,
				_ => 150 * COIN,
			},
			_ => 10 * COIN,
		}
	}
}

frame_support::parameter_types! {
	pub const EthereumRelayerGameLockId: LockIdentifier = *b"da/rgame";
}

impl Config<EthereumRelayerGameInstance> for Runtime {
	type RingCurrency = Ring;
	type LockId = EthereumRelayerGameLockId;
	type RingSlash = Treasury;
	type RelayerGameAdjustor = EthereumRelayerGameAdjustor;
	type RelayableChain = EthereumRelay;
	type WeightInfo = ();
}
