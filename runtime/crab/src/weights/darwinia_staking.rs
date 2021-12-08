#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_staking.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_staking::WeightInfo for WeightInfo<T> {
	// Storage: Staking Bonded (r:1 w:1)
	// Storage: Staking Ledger (r:1 w:1)
	// Storage: Staking CurrentEra (r:1 w:0)
	// Storage: Staking HistoryDepth (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: Staking Payee (r:0 w:1)
	fn bond() -> Weight {
		(71_638_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: Staking Bonded (r:1 w:0)
	// Storage: Staking Ledger (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	fn bond_extra() -> Weight {
		(54_546_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn deposit_extra() -> Weight {
		(54_546_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn claim_mature_deposits() -> Weight {
		(54_546_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn try_claim_deposits_with_punish() -> Weight {
		(54_546_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Staking Ledger (r:1 w:1)
	// Storage: Staking Nominators (r:1 w:0)
	// Storage: Staking Validators (r:1 w:0)
	// Storage: Staking CurrentEra (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn unbond() -> Weight {
		(58_492_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Staking Ledger (r:1 w:0)
	// Storage: Staking MinValidatorBond (r:1 w:0)
	// Storage: Staking Validators (r:1 w:1)
	// Storage: Staking MaxValidatorsCount (r:1 w:0)
	// Storage: Staking Nominators (r:1 w:0)
	// Storage: Staking CounterForValidators (r:1 w:1)
	fn validate() -> Weight {
		(31_702_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Staking Ledger (r:1 w:0)
	// Storage: Staking Nominators (r:1 w:1)
	fn kick(k: u32, ) -> Weight {
		(8_188_000 as Weight)
			// Standard Error: 13_000
			.saturating_add((18_241_000 as Weight).saturating_mul(k as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(k as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(k as Weight)))
	}
	// Storage: Staking Ledger (r:1 w:0)
	// Storage: Staking MinNominatorBond (r:1 w:0)
	// Storage: Staking Nominators (r:1 w:1)
	// Storage: Staking MaxNominatorsCount (r:1 w:0)
	// Storage: Staking Validators (r:2 w:0)
	// Storage: Staking CurrentEra (r:1 w:0)
	// Storage: Staking CounterForNominators (r:1 w:1)
	fn nominate(n: u32, ) -> Weight {
		(37_988_000 as Weight)
			// Standard Error: 5_000
			.saturating_add((5_184_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(n as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Staking Ledger (r:1 w:0)
	// Storage: Staking Validators (r:1 w:0)
	// Storage: Staking Nominators (r:1 w:0)
	fn chill() -> Weight {
		(16_559_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
	}
	// Storage: Staking Ledger (r:1 w:0)
	// Storage: Staking Payee (r:0 w:1)
	fn set_payee() -> Weight {
		(11_664_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Staking Bonded (r:1 w:1)
	// Storage: Staking Ledger (r:2 w:2)
	fn set_controller() -> Weight {
		(25_130_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Staking ValidatorCount (r:0 w:1)
	fn set_validator_count() -> Weight {
		(2_588_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Staking ForceEra (r:0 w:1)
	fn force_no_eras() -> Weight {
		(2_855_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Staking ForceEra (r:0 w:1)
	fn force_new_era() -> Weight {
		(2_807_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Staking ForceEra (r:0 w:1)
	fn force_new_era_always() -> Weight {
		(2_897_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Staking Invulnerables (r:0 w:1)
	fn set_invulnerables(v: u32, ) -> Weight {
		(3_276_000 as Weight)
			// Standard Error: 0
			.saturating_add((55_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Staking Bonded (r:1 w:1)
	// Storage: Staking SlashingSpans (r:1 w:0)
	// Storage: Staking Validators (r:1 w:0)
	// Storage: Staking Nominators (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: Staking Ledger (r:0 w:1)
	// Storage: Staking Payee (r:0 w:1)
	// Storage: Staking SpanSlash (r:0 w:2)
	fn force_unstake(s: u32, ) -> Weight {
		(57_619_000 as Weight)
			// Standard Error: 6_000
			.saturating_add((2_526_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
	}
	// Storage: Staking UnappliedSlashes (r:1 w:1)
	fn cancel_deferred_slash(s: u32, ) -> Weight {
		(6_603_662_000 as Weight)
			// Standard Error: 449_000
			.saturating_add((39_782_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Staking CurrentEra (r:1 w:0)
	// Storage: Staking HistoryDepth (r:1 w:0)
	// Storage: Staking ErasValidatorReward (r:1 w:0)
	// Storage: Staking Bonded (r:2 w:0)
	// Storage: Staking Ledger (r:1 w:1)
	// Storage: Staking ErasStakersClipped (r:1 w:0)
	// Storage: Staking ErasRewardPoints (r:1 w:0)
	// Storage: Staking ErasValidatorPrefs (r:1 w:0)
	// Storage: Staking Payee (r:2 w:0)
	// Storage: System Account (r:2 w:2)
	fn payout_stakers_dead_controller(n: u32, ) -> Weight {
		(124_439_000 as Weight)
			// Standard Error: 32_000
			.saturating_add((49_128_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(n as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(n as Weight)))
	}
	// Storage: Staking CurrentEra (r:1 w:0)
	// Storage: Staking HistoryDepth (r:1 w:0)
	// Storage: Staking ErasValidatorReward (r:1 w:0)
	// Storage: Staking Bonded (r:2 w:0)
	// Storage: Staking Ledger (r:2 w:2)
	// Storage: Staking ErasStakersClipped (r:1 w:0)
	// Storage: Staking ErasRewardPoints (r:1 w:0)
	// Storage: Staking ErasValidatorPrefs (r:1 w:0)
	// Storage: Staking Payee (r:2 w:0)
	// Storage: System Account (r:2 w:2)
	// Storage: Balances Locks (r:2 w:2)
	fn payout_stakers_alive_staked(n: u32, ) -> Weight {
		(134_507_000 as Weight)
			// Standard Error: 24_000
			.saturating_add((61_661_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(T::DbWeight::get().reads(11 as Weight))
			.saturating_add(T::DbWeight::get().reads((5 as Weight).saturating_mul(n as Weight)))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
			.saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(n as Weight)))
	}
	// Storage: Staking Ledger (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn rebond(l: u32, ) -> Weight {
		(47_910_000 as Weight)
			// Standard Error: 3_000
			.saturating_add((71_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Staking CurrentEra (r:1 w:0)
	// Storage: Staking HistoryDepth (r:1 w:1)
	// Storage: Staking ErasStakersClipped (r:0 w:2)
	// Storage: Staking ErasValidatorPrefs (r:0 w:2)
	// Storage: Staking ErasValidatorReward (r:0 w:1)
	// Storage: Staking ErasRewardPoints (r:0 w:1)
	// Storage: Staking ErasStakers (r:0 w:2)
	// Storage: Staking ErasTotalStake (r:0 w:1)
	// Storage: Staking ErasStartSessionIndex (r:0 w:1)
	fn set_history_depth(e: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 75_000
			.saturating_add((32_986_000 as Weight).saturating_mul(e as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
			.saturating_add(T::DbWeight::get().writes((7 as Weight).saturating_mul(e as Weight)))
	}
	// Storage: System Account (r:1 w:1)
	// Storage: Staking Bonded (r:1 w:1)
	// Storage: Staking SlashingSpans (r:1 w:1)
	// Storage: Staking Validators (r:1 w:1)
	// Storage: Staking CounterForValidators (r:1 w:1)
	// Storage: Staking Nominators (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: Staking Ledger (r:0 w:1)
	// Storage: Staking Payee (r:0 w:1)
	// Storage: Staking SpanSlash (r:0 w:1)
	fn reap_stash(s: u32, ) -> Weight {
		(69_437_000 as Weight)
			// Standard Error: 4_000
			.saturating_add((2_491_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
	}
	// Storage: Staking CounterForNominators (r:1 w:0)
	// Storage: Staking CounterForValidators (r:1 w:0)
	// Storage: Staking SlashingSpans (r:1 w:0)
	// Storage: Staking Validators (r:2 w:0)
	// Storage: Staking Bonded (r:101 w:0)
	// Storage: Staking Ledger (r:101 w:0)
	// Storage: Staking Nominators (r:101 w:0)
	// Storage: Staking ValidatorCount (r:1 w:0)
	// Storage: System BlockWeight (r:1 w:1)
	// Storage: Staking MinimumValidatorCount (r:1 w:0)
	// Storage: Staking CurrentEra (r:1 w:1)
	// Storage: Staking HistoryDepth (r:1 w:0)
	// Storage: Staking ErasStakersClipped (r:0 w:1)
	// Storage: Staking ErasValidatorPrefs (r:0 w:1)
	// Storage: Staking ErasStakers (r:0 w:1)
	// Storage: Staking ErasTotalStake (r:0 w:1)
	// Storage: Staking ErasStartSessionIndex (r:0 w:1)
	fn new_era(v: u32, n: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 733_000
			.saturating_add((301_800_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 36_000
			.saturating_add((46_777_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(v as Weight)))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(n as Weight)))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
			.saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(v as Weight)))
	}
	// Storage: Staking Validators (r:501 w:0)
	// Storage: Staking Bonded (r:1500 w:0)
	// Storage: Staking Ledger (r:1500 w:0)
	// Storage: Staking SlashingSpans (r:21 w:0)
	// Storage: Staking Nominators (r:1001 w:0)
	fn get_npos_voters(v: u32, n: u32, s: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 97_000
			.saturating_add((23_667_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 97_000
			.saturating_add((26_908_000 as Weight).saturating_mul(n as Weight))
			// Standard Error: 3_312_000
			.saturating_add((50_232_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(v as Weight)))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(n as Weight)))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(s as Weight)))
	}
	// Storage: Staking Validators (r:501 w:0)
	fn get_npos_targets(v: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 27_000
			.saturating_add((9_729_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(v as Weight)))
	}
	// Storage: Staking MinValidatorBond (r:0 w:1)
	// Storage: Staking MaxValidatorsCount (r:0 w:1)
	// Storage: Staking ChillThreshold (r:0 w:1)
	// Storage: Staking MaxNominatorsCount (r:0 w:1)
	// Storage: Staking MinNominatorBond (r:0 w:1)
	fn set_staking_limits() -> Weight {
		(6_532_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: Staking Ledger (r:1 w:0)
	// Storage: Staking ChillThreshold (r:1 w:0)
	// Storage: Staking Nominators (r:1 w:0)
	// Storage: Staking Validators (r:1 w:1)
	// Storage: Staking MaxValidatorsCount (r:1 w:0)
	// Storage: Staking CounterForValidators (r:1 w:1)
	// Storage: Staking MinValidatorBond (r:1 w:0)
	fn chill_other() -> Weight {
		(53_642_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
