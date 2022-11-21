// darwinia
use crate::*;

impl Processor {
	pub fn process_balances(
		&mut self,
		ring_locks: &mut Map<Vec<BalanceLock>>,
		kton_locks: &mut Map<Vec<BalanceLock>>,
	) -> &mut Self {
		log::info!("take solo balance locks");
		self.solo_state
			.take::<Vec<BalanceLock>, _>(
				b"Balances",
				b"Locks",
				ring_locks,
				get_blake2_128_concat_suffix,
			)
			.take::<Vec<BalanceLock>, _>(
				b"Kton",
				b"Locks",
				kton_locks,
				get_blake2_128_concat_suffix,
			);

		// ---
		// Currently, there are only fee-market locks.
		// I suggest shutting down the fee-market before merging.
		// So, we could ignore the para balance locks migration.
		// ---

		log::info!("adjust solo balance lock decimals");
		ring_locks.iter_mut().for_each(|(_, v)| v.iter_mut().for_each(|l| l.amount *= GWEI));
		kton_locks.iter_mut().for_each(|(_, v)| v.iter_mut().for_each(|l| l.amount *= GWEI));

		self
	}
}
