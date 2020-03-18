# CHANGELOG(v2.0.0.alpha.3)

## Core

Some concepts should have some explaination for the changing from substrate

### power

power is a mixture of ring and kton.

+ *RING*: `power = ring_ratio * POWER_COUNT / 2`
+ *KTON*: `power = kton_ratio * POWER_COUNT / 2`

We use `currency_to_power` and `power_of` to calculcate `power`.

### rebond

We doesn't support `rebond` currently now.

### withdraw

What should happen after all balances being unbonded?(the locked balance)


## Moudle
### delete `withdraw_unbond`

+ **withdraw_unbond**: Remove all associated data of a stash account from the staking system.

Darwinia has `active_balance` and `active_deposit_balance`, we calculate `normal_balance` by `active_balance - active_deposit_balance`, the `normal_balance` is **free to transfer**, so we don't need the `withdraw_unbond` function actually.

### delete `slashable_balance_of`

+ **slashable_balance_of**: The total balance that can be slashed from a stash account as of right now.

We use `power_of` and `stake_of` instead of `slashable_balance_of`:

+ **power_of**: The total power that can be slashed from a stash account as of right now.
+ **stake_of**: The `active_ring` and `active_kton` from a stash account.

**For if an account is slashale:**

Just use `power_of`, if the return `power` is zero, the target account is not slashable.

**For the amount of slashable balances:**

The slashable balances actually mean `active-ring` and `active-kton` in darwinia's staking
process, we can use `Staking::ledger(controller)` to get a `StakingLedger` which contains
the `active-ring` and `active-kton` the `controller` have.

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

Currently we don't have this requirement.

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
