// darwinia
use crate::*;

type Locks = Vec<BalanceLock>;

impl<S> Processor<S> {
	pub fn process_balances(&mut self) -> (u128, u128) {
		// Balances storage items.
		// https://github.dev/darwinia-network/substrate/blob/darwinia-v0.12.5/frame/balances/src/lib.rs#L486

		let mut solo_ring_total_issuance = u128::default();
		let mut solo_kton_total_issuance = u128::default();
		let mut solo_ring_locks = <Map<Locks>>::default();
		let mut solo_kton_locks = <Map<Locks>>::default();
		let mut para_ring_total_issuance = u128::default();

		log::info!("take solo `Balances::TotalIssuance`, `Kton::TotalIssuance`, `Balances::Locks` and `Kton::Locks`");
		self.solo_state
			.take_value(b"Balances", b"TotalIssuance", "", &mut solo_ring_total_issuance)
			.take_value(b"Kton", b"TotalIssuance", "", &mut solo_kton_total_issuance)
			.take_map(b"Balances", b"Locks", &mut solo_ring_locks, get_hashed_key)
			.take_map(b"Kton", b"Locks", &mut solo_kton_locks, get_hashed_key);

		log::info!("prune solo balance locks");
		prune(&mut solo_ring_locks);
		prune(&mut solo_kton_locks);

		log::info!("adjust solo total issuances decimals");
		solo_ring_total_issuance.adjust();
		solo_kton_total_issuance.adjust();

		log::info!("take para `Balances::TotalIssuance` and `Balances::Locks`");
		self.para_state.take_value(
			b"Balances",
			b"TotalIssuance",
			"",
			&mut para_ring_total_issuance,
		);

		if self.para_state.exists(b"Balances", b"Locks") {
			log::error!("check para `Balances::Locks`, it isn't empty");
		}

		(solo_ring_total_issuance + para_ring_total_issuance, solo_kton_total_issuance)
	}
}

fn prune(locks: &mut Map<Locks>) {
	// https://github.dev/darwinia-network/darwinia-common/blob/6a9392cfb9fe2c99b1c2b47d0c36125d61991bb7/frame/staking/src/primitives.rs#L39
	const STAKING: [u8; 8] = *b"da/staki";
	// https://github.dev/darwinia-network/darwinia/blob/2d1c1436594b2c397d450e317c35eb16c71105d6/runtime/crab/src/pallets/elections_phragmen.rs#L8
	const PHRAGMEN_ELECTION: [u8; 8] = *b"phrelect";
	// https://github.dev/paritytech/substrate/blob/19162e43be45817b44c7d48e50d03f074f60fbf4/frame/democracy/src/lib.rs#L190
	const DEMOCRACY: [u8; 8] = *b"democrac";
	// https://github.dev/paritytech/substrate/blob/19162e43be45817b44c7d48e50d03f074f60fbf4/frame/vesting/src/lib.rs#L86
	const VESTING: [u8; 8] = *b"vesting ";
	const RELAY_AUTHORITY: [u8; 8] = *b"ethrauth";
	// https://github.dev/darwinia-network/darwinia/blob/2d1c1436594b2c397d450e317c35eb16c71105d6/runtime/crab/src/pallets/fee_market.rs#L35
	const FEE_MARKET_0: [u8; 8] = *b"da/feelf";
	// https://github.dev/darwinia-network/darwinia/blob/2d1c1436594b2c397d450e317c35eb16c71105d6/runtime/crab/src/pallets/fee_market.rs#L36
	const FEE_MARKET_1: [u8; 8] = *b"da/feecp";
	// https://github.dev/darwinia-network/darwinia/blob/2d1c1436594b2c397d450e317c35eb16c71105d6/runtime/darwinia/src/pallets/fee_market.rs#L37
	const FEE_MARKET_2: [u8; 8] = *b"da/feedp";

	locks.retain(|k, v| {
		v.retain(|l| match l.id {
			STAKING | PHRAGMEN_ELECTION | DEMOCRACY | VESTING | RELAY_AUTHORITY | FEE_MARKET_0
			| FEE_MARKET_1 | FEE_MARKET_2 => false,
			id => {
				log::error!(
					"pruned unknown lock id({}) of account({})",
					String::from_utf8_lossy(&id),
					get_last_64(k)
				);

				false
			},
		});

		!v.is_empty()
	});
}
