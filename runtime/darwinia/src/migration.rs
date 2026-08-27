// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

// darwinia
use crate::*;

const TARGET_COLLATOR_COUNT: u32 = 10;
#[cfg(feature = "try-runtime")]
const PREVIOUS_COLLATOR_COUNT: u32 = 20;

pub struct CustomOnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		log::info!("pre");

		let collator_count = darwinia_staking::CollatorCount::<Runtime>::get();
		assert!(
			collator_count == PREVIOUS_COLLATOR_COUNT || collator_count == TARGET_COLLATOR_COUNT,
			"expected collator count to be {PREVIOUS_COLLATOR_COUNT} or {TARGET_COLLATOR_COUNT}, got {collator_count}",
		);

		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		assert_eq!(darwinia_staking::CollatorCount::<Runtime>::get(), TARGET_COLLATOR_COUNT,);

		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrate()
	}
}

fn migrate() -> frame_support::weights::Weight {
	darwinia_staking::CollatorCount::<Runtime>::put(TARGET_COLLATOR_COUNT);

	<Runtime as frame_system::Config>::DbWeight::get().writes(1)
}

#[cfg(all(test, feature = "try-runtime"))]
mod tests {
	use super::*;
	use frame_support::traits::OnRuntimeUpgrade;
	use sp_runtime::BuildStorage;

	#[test]
	fn collator_count_migration_is_idempotent() {
		let mut ext: sp_io::TestExternalities =
			frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap().into();

		ext.execute_with(|| {
			darwinia_staking::CollatorCount::<Runtime>::put(PREVIOUS_COLLATOR_COUNT);

			for _ in 0..2 {
				let state = CustomOnRuntimeUpgrade::pre_upgrade().unwrap();
				CustomOnRuntimeUpgrade::on_runtime_upgrade();
				CustomOnRuntimeUpgrade::post_upgrade(state).unwrap();
			}

			assert_eq!(darwinia_staking::CollatorCount::<Runtime>::get(), TARGET_COLLATOR_COUNT);
		});
	}
}
