#![cfg(test)]

use sp_runtime::Perbill;
use substrate_test_utils::assert_eq_uvec;

use crate::{elect, mock::*, PhragmenResult};

#[test]
fn phragmen_poc_works() {
	let candidates = vec![1, 2, 3];
	let voters = vec![(10, vec![1, 2]), (20, vec![1, 3]), (30, vec![2, 3])];

	let PhragmenResult { winners, assignments } = elect::<_, _>(
		2,
		2,
		candidates,
		voters,
		create_stake_of(&[(10, 10), (20, 20), (30, 30)]),
		1_000_000_000,
	)
	.unwrap();

	assert_eq_uvec!(winners, vec![(2, 40), (3, 50)]);
	assert_eq_uvec!(
		assignments,
		vec![
			(10, vec![(2, Perbill::from_percent(100))]),
			(20, vec![(3, Perbill::from_percent(100))]),
			(
				30,
				vec![(2, Perbill::from_percent(100 / 2)), (3, Perbill::from_percent(100 / 2))]
			),
		]
	);
}

#[test]
fn phragmen_poc_2_works() {
	let candidates = vec![10, 20, 30];
	let voters = vec![(2, vec![10, 20, 30]), (4, vec![10, 20, 40])];
	let stake_of = create_stake_of(&[(10, 1000), (20, 1000), (30, 1000), (40, 1000), (2, 500), (4, 500)]);

	run_and_compare(candidates, voters, stake_of, 2, 2);
}

#[test]
fn phragmen_poc_3_works() {
	let candidates = vec![10, 20, 30];
	let voters = vec![(2, vec![10, 20, 30]), (4, vec![10, 20, 40])];
	let stake_of = create_stake_of(&[(10, 1000), (20, 1000), (30, 1000), (2, 50), (4, 1000)]);

	run_and_compare(candidates, voters, stake_of, 2, 2);
}
