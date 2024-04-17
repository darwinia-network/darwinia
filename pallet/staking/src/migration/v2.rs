// core
use core::marker::PhantomData;
// darwinia
use crate::*;
use dc_types::UNIT;
// substrate
use frame_support::traits::OnRuntimeUpgrade;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

#[frame_support::storage_alias]
type RingPool<T: Config> = StorageValue<Pallet<T>, ()>;
#[frame_support::storage_alias]
type KtonPool<T: Config> = StorageValue<Pallet<T>, ()>;
#[frame_support::storage_alias]
type MigrationStartBlock<T: Config> = StorageValue<Pallet<T>, ()>;

#[allow(missing_docs)]
#[derive(
	DebugNoBound,
	DefaultNoBound,
	PartialEqNoBound,
	EqNoBound,
	Encode,
	Decode,
	MaxEncodedLen,
	TypeInfo,
)]
#[scale_info(skip_type_params(T))]
pub struct OldLedger<T>
where
	T: Config,
{
	pub staked_ring: Balance,
	pub staked_kton: Balance,
	pub staked_deposits: BoundedVec<DepositId<T>, <T as Config>::MaxDeposits>,
	pub unstaking_ring: BoundedVec<(Balance, BlockNumberFor<T>), ConstU32<16>>,
	pub unstaking_kton: BoundedVec<(Balance, BlockNumberFor<T>), ConstU32<16>>,
	pub unstaking_deposits: BoundedVec<(DepositId<T>, BlockNumberFor<T>), ConstU32<16>>,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
struct OldExposure<AccountId> {
	commission: Perbill,
	vote: u32,
	nominators: Vec<OldIndividualExposure<AccountId>>,
}
#[cfg_attr(test, derive(Clone))]
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
struct OldIndividualExposure<AccountId> {
	who: AccountId,
	vote: u32,
}

/// Migrate darwinia-staking from v1 to v2.
pub struct MigrateToV2<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for MigrateToV2<T>
where
	T: Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		Ok(Vec::new())
	}

	fn on_runtime_upgrade() -> Weight {
		let version = StorageVersion::get::<Pallet<T>>();
		let r = 1;

		if version != 1 {
			log::warn!(
				"\
				[pallet::staking] skipping v1 to v2 migration: executed on wrong storage version.\
				Expected version 1, found {version:?}\
				",
			);

			return T::DbWeight::get().reads(r);
		}

		let mut w = 5;

		<RingPool<T>>::kill();
		<KtonPool<T>>::kill();
		<MigrationStartBlock<T>>::kill();
		<RateLimit<T>>::put(20_000_000 * UNIT);
		<Ledgers<T>>::translate::<OldLedger<T>, _>(|a, o| {
			w += 6;

			let unstaking_ring = o.unstaking_ring.into_iter().fold(0, |s, (v, _)| s + v);

			// Release the unstaking RING immediately.
			if unstaking_ring != 0 {
				let _ = <T as Config>::Ring::unstake(&a, unstaking_ring);
			}

			// Release all KTON immediately.
			let _ = <T as Config>::Kton::unstake(
				&a,
				o.staked_kton + o.unstaking_kton.into_iter().fold(0, |s, (v, _)| s + v),
			);

			// Release the unstaking deposits immediately.
			for (d, _) in o.unstaking_deposits {
				let _ = T::Deposit::unstake(&a, d);
			}

			Some(Ledger { ring: o.staked_ring, deposits: o.staked_deposits })
		});
		<ExposureCache0<T>>::translate_values::<OldExposure<T::AccountId>, _>(|o| {
			w += 1;

			Some(Exposure {
				commission: o.commission,
				vote: o.vote as _,
				nominators: o
					.nominators
					.into_iter()
					.map(|o| IndividualExposure { who: o.who, vote: o.vote as _ })
					.collect(),
			})
		});
		<ExposureCache1<T>>::translate_values::<OldExposure<T::AccountId>, _>(|o| {
			w += 1;

			Some(Exposure {
				commission: o.commission,
				vote: o.vote as _,
				nominators: o
					.nominators
					.into_iter()
					.map(|o| IndividualExposure { who: o.who, vote: o.vote as _ })
					.collect(),
			})
		});
		<ExposureCache2<T>>::translate_values::<OldExposure<T::AccountId>, _>(|o| {
			w += 1;

			Some(Exposure {
				commission: o.commission,
				vote: o.vote as _,
				nominators: o
					.nominators
					.into_iter()
					.map(|o| IndividualExposure { who: o.who, vote: o.vote as _ })
					.collect(),
			})
		});

		StorageVersion::new(2).put::<Pallet<T>>();

		T::DbWeight::get().reads_writes(r, w)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_: Vec<u8>) -> Result<(), TryRuntimeError> {
		assert_eq!(StorageVersion::get::<Pallet<T>>(), 2, "Version must be upgraded.");

		Ok(())
	}
}
