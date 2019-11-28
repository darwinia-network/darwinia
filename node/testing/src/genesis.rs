// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Genesis Configuration.

use crate::keyring::*;
use keyring::{Ed25519Keyring, Sr25519Keyring};
use node_runtime::constants::currency::*;
use node_runtime::{
	BalancesConfig, ContractsConfig, GenesisConfig, IndicesConfig, KtonConfig, SessionConfig, StakingConfig,
	SystemConfig, WASM_BINARY,
};
use primitives::ChangesTrieConfiguration;
use sr_primitives::Perbill;

/// Create genesis runtime configuration for tests.
pub fn config(support_changes_trie: bool, code: Option<&[u8]>) -> GenesisConfig {
	GenesisConfig {
		babe: Some(Default::default()),
		contracts: Some(ContractsConfig {
			current_schedule: Default::default(),
			gas_price: 1 * MICRO,
		}),
		grandpa: Some(Default::default()),
		im_online: Some(Default::default()),
		indices: Some(IndicesConfig {
			ids: vec![alice(), bob(), charlie(), dave(), eve(), ferdie()],
		}),
		session: Some(SessionConfig {
			keys: vec![
				(alice(), to_session_keys(&Ed25519Keyring::Alice, &Sr25519Keyring::Alice)),
				(bob(), to_session_keys(&Ed25519Keyring::Bob, &Sr25519Keyring::Bob)),
				(
					charlie(),
					to_session_keys(&Ed25519Keyring::Charlie, &Sr25519Keyring::Charlie),
				),
			],
		}),
		sudo: Some(Default::default()),
		system: Some(SystemConfig {
			changes_trie_config: if support_changes_trie {
				Some(ChangesTrieConfiguration {
					digest_interval: 2,
					digest_levels: 2,
				})
			} else {
				None
			},
			code: code.map(|x| x.to_vec()).unwrap_or_else(|| WASM_BINARY.to_vec()),
		}),

		balances: Some(BalancesConfig {
			balances: vec![
				(alice(), 111 * COIN),
				(bob(), 100 * COIN),
				(charlie(), 100_000_000 * COIN),
				(dave(), 111 * COIN),
				(eve(), 101 * COIN),
				(ferdie(), 100 * COIN),
			],
			vesting: vec![],
		}),
		kton: Some(KtonConfig {
			balances: vec![
				(alice(), 111 * COIN),
				(bob(), 100 * COIN),
				(charlie(), 100_000_000 * COIN),
				(dave(), 111 * COIN),
				(eve(), 101 * COIN),
				(ferdie(), 100 * COIN),
			],
			vesting: vec![],
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			stakers: vec![
				(dave(), alice(), 111 * COIN, staking::StakerStatus::Validator),
				(eve(), bob(), 100 * COIN, staking::StakerStatus::Validator),
				(ferdie(), charlie(), 100 * COIN, staking::StakerStatus::Validator),
			],
			validator_count: 3,
			minimum_validator_count: 0,
			slash_reward_fraction: Perbill::from_percent(10),
			invulnerables: vec![alice(), bob(), charlie()],
			..Default::default()
		}),
	}
}
