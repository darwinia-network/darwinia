// core
use core::marker::PhantomData;
// darwinia
use crate::*;
// polkadot-sdk
use frame_support::traits::OnRuntimeUpgrade;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

type AccountId<T> = <T as frame_system::Config>::AccountId;

#[frame_support::storage_alias]
type NextExposures<T: Config> =
	StorageMap<Pallet<T>, Twox64Concat, AccountId<T>, ExposureV0<AccountId<T>>>;

#[frame_support::storage_alias]
type Exposures<T: Config> =
	StorageMap<Pallet<T>, Twox64Concat, AccountId<T>, ExposureV0<AccountId<T>>>;

#[frame_support::storage_alias]
type RewardPoints<T: Config> =
	StorageValue<Pallet<T>, (u32, BTreeMap<AccountId<T>, u32>), ValueQuery>;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
struct ExposureV0<AccountId> {
	vote: Balance,
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
		Ok(Vec::new())
	}

	fn on_runtime_upgrade() -> Weight {
		let version = StorageVersion::get::<Pallet<T>>();

		if version != 0 {
			log::warn!(
				"[pallet::staking] skipping v0 to v1 migration: executed on wrong storage version. Expected version 0, found {version:?}",
			);

			return T::DbWeight::get().reads(1);
		}

		let mut count = 4;

		let (sum, map) = <RewardPoints<T>>::take();
		<AuthoredBlocksCount<T>>::put((
			<BlockNumberFor<T>>::from(sum / 20),
			map.into_iter()
				.map(|(k, v)| (k, <BlockNumberFor<T>>::from(v / 20)))
				.collect::<BTreeMap<_, _>>(),
		));

		<Exposures<T>>::iter().drain().for_each(|(k, v)| {
			count += 1;

			<ExposureCache1<T>>::insert(
				&k,
				Exposure {
					commission: <Collators<T>>::get(&k).unwrap_or_default(),
					vote: v.vote,
					nominators: v.nominators,
				},
			);
		});
		<NextExposures<T>>::iter().drain().for_each(|(k, v)| {
			count += 1;

			<ExposureCache2<T>>::insert(
				&k,
				Exposure {
					commission: <Collators<T>>::get(&k).unwrap_or_default(),
					vote: v.vote,
					nominators: v.nominators,
				},
			);
		});

		StorageVersion::new(1).put::<Pallet<T>>();

		T::DbWeight::get().reads_writes(count, count)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_: Vec<u8>) -> Result<(), TryRuntimeError> {
		assert_eq!(StorageVersion::get::<Pallet<T>>(), 1, "Version must be upgraded.");

		// Old storages should be killed.
		assert_eq!(<Exposures<T>>::iter_keys().count(), 0);
		assert_eq!(<NextExposures<T>>::iter_keys().count(), 0);
		assert!(!<RewardPoints<T>>::exists());

		// Check the starting state is correct.
		assert_eq!(
			<ExposureCacheStates<T>>::get(),
			(CacheState::Previous, CacheState::Current, CacheState::Next)
		);

		assert_eq!(
			<ExposureCache0<T>>::iter_keys().count(),
			0,
			"Previous exposure should be empty at start."
		);

		// Check that everything decoded fine.
		<ExposureCache1<T>>::iter_keys().for_each(|k| {
			assert!(<ExposureCache1<T>>::try_get(k).is_ok(), "Can not decode V1 `Exposure`.");
		});
		<ExposureCache2<T>>::iter_keys().for_each(|k| {
			assert!(<ExposureCache2<T>>::try_get(k).is_ok(), "Can not decode V1 `Exposure`.");
		});

		Ok(())
	}
}
