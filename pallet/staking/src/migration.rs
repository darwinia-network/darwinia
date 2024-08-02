//! Pallet migrations.

// darwinia
use crate::*;
// polkadot-sdk
use sp_core::H160;

/// Migrate `StakingRewardDistribution` contract.
///
/// https://github.com/darwinia-network/KtonDAO/blob/722bdf62942868de2eeaf19bc70d7a165fc031af/src/Owned.sol#L5.
/// https://github.com/darwinia-network/KtonDAO/blob/045b5b59d56b426cb8b06b9da912d0a3ad0a636d/src/staking/KtonDAOVault.sol#L36.
pub fn migrate_staking_reward_distribution_contract<T>(kton_staking_contract: T::AccountId)
where
	T: Config + darwinia_ethtx_forwarder::Config,
	T::AccountId: Into<H160>,
{
	let treasury = <T as Config>::Treasury::get().into();
	let ksc = kton_staking_contract.into();
	// 0x000000000Ae5DB7BDAf8D071e680452e33d91Dd5.
	let ksc_old = H160([
		0, 0, 0, 0, 10, 229, 219, 123, 218, 248, 208, 113, 230, 128, 69, 46, 51, 217, 29, 213,
	]);

	#[allow(deprecated)]
	darwinia_ethtx_forwarder::quick_forward_transact::<T>(
		treasury,
		Function {
			name: "nominateNewOwner".into(),
			inputs: vec![Param {
				name: "_owner".into(),
				kind: ParamType::Address,
				internal_type: None,
			}],
			outputs: Vec::new(),
			constant: None,
			state_mutability: StateMutability::Payable,
		},
		&[Token::Address(ksc)],
		ksc_old,
		0.into(),
		1_000_000.into(),
	);
	#[allow(deprecated)]
	darwinia_ethtx_forwarder::quick_forward_transact::<T>(
		treasury,
		Function {
			name: "acceptOwnershipFromOldDistribution".into(),
			inputs: Vec::new(),
			outputs: Vec::new(),
			constant: None,
			state_mutability: StateMutability::Payable,
		},
		&[],
		ksc,
		0.into(),
		1_000_000.into(),
	);
}
