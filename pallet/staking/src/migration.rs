//! Pallet migrations.

/// Migration version 1.
pub mod v1;
/// Migration version 2.
pub mod v2;

// crates.io
use ethabi::Token;
// self
use crate::*;

/// Migrate `StakingRewardDistribution` contract.
///
/// https://github.com/darwinia-network/KtonDAO/blob/722bdf62942868de2eeaf19bc70d7a165fc031af/src/Owned.sol#L5.
/// https://github.com/darwinia-network/KtonDAO/blob/045b5b59d56b426cb8b06b9da912d0a3ad0a636d/src/staking/KtonDAOVault.sol#L36.
pub fn migrate_staking_reward_distribution_contract<T>()
where
	T: Config + darwinia_ethtx_forwarder::Config,
	T::RuntimeOrigin: Into<Result<ForwardEthOrigin, T::RuntimeOrigin>> + From<ForwardEthOrigin>,
	<T as frame_system::Config>::AccountId: Into<H160>,
{
	// Treasury pallet account.
	let sender =
		H160([109, 111, 100, 108, 100, 97, 47, 116, 114, 115, 114, 121, 0, 0, 0, 0, 0, 0, 0, 0]);
	let krd_contract = T::KtonRewardDistributionContract::get().into();
	// 0x000000000Ae5DB7BDAf8D071e680452e33d91Dd5.
	let krd_contract_old = H160([
		0, 0, 0, 0, 10, 229, 219, 123, 218, 248, 208, 113, 230, 128, 69, 46, 51, 217, 29, 213,
	]);

	{
		#[allow(deprecated)]
		let func = Function {
			name: "nominateNewOwner".into(),
			inputs: vec![Param {
				name: "_owner".into(),
				kind: ParamType::Address,
				internal_type: None,
			}],
			outputs: Vec::new(),
			constant: None,
			state_mutability: StateMutability::Payable,
		};
		let input = match func.encode_input(&[Token::Address(krd_contract)]) {
			Ok(i) => i,
			Err(e) => {
				log::error!("failed to encode input due to {e:?}");

				return;
			},
		};
		let req = ForwardRequest {
			tx_type: TxType::LegacyTransaction,
			action: TransactionAction::Call(krd_contract_old),
			value: U256::zero(),
			input,
			gas_limit: U256::from(1_000_000),
		};

		if let Err(e) = <darwinia_ethtx_forwarder::Pallet<T>>::forward_transact(
			ForwardEthOrigin::ForwardEth(sender).into(),
			req,
		) {
			log::error!("failed to call `nominateNewOwner` due to {e:?}");
		}
	}
	{
		#[allow(deprecated)]
		let func = Function {
			name: "acceptOwnershipFromOldDistribution".into(),
			inputs: Vec::new(),
			outputs: Vec::new(),
			constant: None,
			state_mutability: StateMutability::Payable,
		};
		let input = match func.encode_input(&[]) {
			Ok(i) => i,
			Err(e) => {
				log::error!("failed to encode input due to {e:?}");

				return;
			},
		};
		let req = ForwardRequest {
			tx_type: TxType::LegacyTransaction,
			action: TransactionAction::Call(krd_contract),
			value: U256::zero(),
			input,
			gas_limit: U256::from(1_000_000),
		};

		if let Err(e) = <darwinia_ethtx_forwarder::Pallet<T>>::forward_transact(
			ForwardEthOrigin::ForwardEth(sender).into(),
			req,
		) {
			log::error!("failed to call `nominateNewOwner` due to {e:?}");
		}
	}
}
