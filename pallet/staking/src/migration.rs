//! Pallet migrations.

// core
use core::marker::PhantomData;
// darwinia
use crate::*;
// substrate
use frame_support::traits::OnRuntimeUpgrade;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// Migration version 1.
pub mod v1 {
	// darwinia
	use super::*;
	// substrate
	use frame_support::storage::child::KillStorageResult;

	type AccountId<T> = <T as frame_system::Config>::AccountId;

	#[frame_support::storage_alias]
	type NextExposures<T: Config> =
		StorageMap<Pallet<T>, Twox64Concat, AccountId<T>, Exposure<AccountId<T>>>;

	#[frame_support::storage_alias]
	type RewardPoints<T: Config> =
		StorageValue<Pallet<T>, (u32, BTreeMap<AccountId<T>, u32>), ValueQuery>;

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
	struct ExposureV0<AccountId> {
		vote: Vote,
		nominators: Vec<IndividualExposure<AccountId>>,
	}

	/// Migrate darwinia-staking from v0 to v1.
	pub struct MigrateToV1<T>(PhantomData<T>);
	impl<T> OnRuntimeUpgrade for MigrateToV1<T>
	where
		T: Config,
	{
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			ensure!(StorageVersion::get::<Pallet<T>>() == 0, "Can only upgrade from version 0.");

			Ok(Vec::new())
		}

		fn on_runtime_upgrade() -> Weight {
			let version = StorageVersion::get::<Pallet<T>>();

			if version != 0 {
				log::warn!(
					"[pallet::staking] skipping v0 to v1 migration: executed on wrong storage version. Expected version 1, found {version:?}",
				);

				return T::DbWeight::get().reads(1);
			}

			let mut count = 4;

			<Exposures<T>>::translate::<ExposureV0<_>, _>(|c, v| {
				count += 1;

				Some(Exposure {
					commission: <Collators<T>>::get(c).unwrap_or_default(),
					vote: v.vote,
					nominators: v.nominators,
				})
			});
			#[allow(deprecated)]
			{
				count += match <NextExposures<T>>::remove_all(None) {
					KillStorageResult::AllRemoved(w) => w,
					KillStorageResult::SomeRemaining(w) => w,
				} as u64;
			}
			let (sum, map) = <RewardPoints<T>>::take();
			<AuthoredBlocksCount<T>>::put((
				<BlockNumberFor<T>>::from(sum / 20),
				map.into_iter()
					.map(|(k, v)| (k, <BlockNumberFor<T>>::from(v / 20)))
					.collect::<BTreeMap<_, _>>(),
			));

			StorageVersion::new(1).put::<Pallet<T>>();

			T::DbWeight::get().reads_writes(count, count)
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_: Vec<u8>) -> Result<(), TryRuntimeError> {
			ensure!(StorageVersion::get::<Pallet<T>>() == 1, "Version must be upgraded.");

			// Check that everything decoded fine.
			for k in <Exposures<T>>::iter_keys() {
				ensure!(<Exposures<T>>::try_get(k).is_ok(), "Can not decode V1 `Exposure`.");
			}

			Ok(())
		}
	}
}
