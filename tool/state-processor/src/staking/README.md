### Process steps
- take solo `Staking::Bonded`, `Staking::Ledger`, `Staking::RingPool`, `Staking::KtonPool` and `Staking::LivingTime`
- adjust decimals and block number, convert ledger, adjust unstaking duration then set `AccountMigration::Ledgers` and `AccountMigration::Deposits`
- set `Staking::RingPool` and `Staking::KtonPool`
