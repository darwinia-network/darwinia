# CHANGELOG(v2.0.0.alpha.3)

## Core

Some concepts should have some explaination for the changing from substrate

### power

power is a mixture of ring and kton.

+ For *RING*: `power = ring_ratio * POWER_COUNT / 2`
+ For *KTON*: `power = kton_ratio * POWER_COUNT / 2`

### rebond

The darwinia style `rebond` implementation.


### withdraw

What should happen after all balances being unbonded?(the locked balance)


## Moudle
+ delete `withdraw_unbond`
+ delete `slashable_balance_of`

## Structs

### Exposure

A snapshot of the stake backing a single validator in the system.

> darwinia

```rust
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Exposure<AccountId, RingBalance, KtonBalance>
where
	RingBalance: HasCompact,
	KtonBalance: HasCompact,
{
	#[codec(compact)]
	pub own_ring_balance: RingBalance,
	#[codec(compact)]
	pub own_kton_balance: KtonBalance,
	pub own_power: Power,
	pub total_power: Power,
	pub others: Vec<IndividualExposure<AccountId, RingBalance, KtonBalance>>,
}
```

> substrate

```rust
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Exposure<AccountId, Balance: HasCompact> {
	/// The total balance backing this validator.
	#[codec(compact)]
	pub total: Balance,
	/// The validator's own stash that is exposed.
	#[codec(compact)]
	pub own: Balance,
	/// The portions of nominators stashes that are exposed.
	pub others: Vec<IndividualExposure<AccountId, Balance>>,
}
```

### IndividualExposure

The amount of exposure (to slashing) than an individual nominator has.

> darwinia

```rust
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct IndividualExposure<AccountId, RingBalance, KtonBalance>
where
	RingBalance: HasCompact,
	KtonBalance: HasCompact,
{
	who: AccountId,
	#[codec(compact)]
	ring_balance: RingBalance,
	#[codec(compact)]
	kton_balance: KtonBalance,
	power: Power,
}
```

> substrate
```rust
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct IndividualExposure<AccountId, Balance: HasCompact> {
	/// The stash account of the nominator in question.
	who: AccountId,
	/// Amount of funds exposed.
	#[codec(compact)]
	value: Balance,
}
```


### StakingLedger

The ledger of a (bonded) stash.

+ annotated `rebond`

> darwinia
```rust
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct StakingLedger<AccountId, RingBalance, KtonBalance, BlockNumber, Timestamp>
where
	RingBalance: HasCompact,
	KtonBalance: HasCompact,
{
	pub stash: AccountId,
  #[codec(compact)]
	pub active_ring: RingBalance,
  #[codec(compact)]
	pub active_deposit_ring: RingBalance,
	#[codec(compact)]
	pub active_kton: KtonBalance,
	pub deposit_items: Vec<TimeDepositItem<RingBalance, Timestamp>>,
	pub ring_staking_lock: StakingLock<RingBalance, BlockNumber>,
	pub kton_staking_lock: StakingLock<KtonBalance, BlockNumber>,
}
```

> substrate

```rust
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingLedger<AccountId, Balance: HasCompact> {
	pub stash: AccountId,
	/// The total amount of the stash's balance that we are currently accounting for.
	/// It's just `active` plus all the `unlocking` balances.
	#[codec(compact)]
	pub total: Balance,
	/// The total amount of the stash's balance that will be at stake in any forthcoming
	/// rounds.
	#[codec(compact)]
	pub active: Balance,
	/// Any balance that is becoming free, which may eventually be transferred out
	/// of the stash (assuming it doesn't get slashed first).
	pub unlocking: Vec<UnlockChunk<Balance>>,
}
```
