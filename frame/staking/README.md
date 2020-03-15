# Staking

The Staking module is the means by which a set of network maintainers (known as _authorities_
in some contexts and _validators_ in others) are chosen based upon those who voluntarily place
funds under deposit. Under deposit, those funds are rewarded under normal operation but are
held at pain of _slash_ (expropriation) should the staked maintainer be found not to be
discharging its duties properly.

## Terminology

- **Staking**: The process of locking up funds for some time, placing them at risk of slashing
(loss) in order to become a rewarded maintainer of the network.
- **Validating**: The process of running a node to actively maintain the network, either by
producing blocks or guaranteeing finality of the chain.
- **Nominating**: The process of placing staked funds behind one or more validators in order to
share in any reward, and punishment, they take.
- **Stash account**: The account holding an owner's funds used for staking.
- **Controller account**: The account that controls an owner's funds for staking.
- **Era**: A (whole) number of sessions, which is the period that the validator set (and each
validator's active nominator set) is recalculated and where rewards are paid out.
- **Slash**: The punishment of a staker by reducing its funds.

## FAQ

### Q1: What is the relationship between stash and controller?

Stash account holding an owner's funds used for staking, controller account controls an owner's funds for staking.

### Q2: What does staker mean?

Almost any interaction with the Staking module requires a process of **bonding** (also known
as being a *staker*). To become *bonded*, a fund-holding account known as the *stash account*,
which holds some or all of the funds that become frozen in place as part of the staking process,
is paired with an active **controller** account, which issues instructions on how they shall be
used.

### Q3: What are the differents from BlockNumber, Era, Session and TimeStamp?

We config the relationships manually, for example: 

```rust
pub fn start_session(session_index: SessionIndex) {
	for i in Session::current_index()..session_index {
		Staking::on_finalize(System::block_number());
		System::set_block_number((i + 1).into());
		Timestamp::set_timestamp(System::block_number() * 1000);
		Session::on_initialize(System::block_number());
	}

	assert_eq!(Session::current_index(), session_index);
}
```

| Unit        | Value    |
|-------------|----------|
| BlockNumber | 4        |
| Session     | 3        |
| Timestamp   | 3 * 1000 |
| Era         | 1        |

### Q4: What is the process of rewrad?

```rust
// 1. Insert stash account into Payment map.
Payee::<Test>::insert(11, RewardDestination::Controller);

// 2. Add reward points to validators using their stash account ID.
Staking::reward_by_ids(vec![(11, 50)]);

// 3. Make all validator and nominator request their payment
make_all_reward_payment(0); // 0 means 0 era.
```

**What happend exactly?**

`make_all_reward_payment` triggers reward process:

+ `make_all_reward_payment`
  + reward nominators
    + payout from nominators to controller
  + reward validators
    + payout from validators to controller



#### [Sequential Phragmén’s method.][0]




[0]: https://research.web3.foundation/en/latest/polkadot/NPoS/4.%20Sequential%20Phragm%C3%A9n%E2%80%99s%20method.html
