### Process steps
- take solo `Staking::Ledger`, `Staking::RingPool`, `Staking::KtonPool` and `Staking::LivingTime`
- clean empty ledger
- adjust decimals and block number, convert ledger, adjust unstaking duration then set `AccountMigration::Ledgers` and `AccountMigration::Deposits`
- set `DarwiniaStaking::RingPool` and `DarwiniaStaking::KtonPool`
- set `DarwiniaStaking::ElapsedTime`
