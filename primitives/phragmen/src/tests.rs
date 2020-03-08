//! Tests for phragmen.

use sp_runtime::Perbill;
use substrate_test_utils::assert_eq_uvec;

use crate::{mock::*, *};

type Output = Perbill;

#[test]
fn float_phragmen_poc_works() {
	let candidates = vec![1, 2, 3];
	let voters = vec![(10, vec![1, 2]), (20, vec![1, 3]), (30, vec![2, 3])];
	let power_of = create_power_of(&[(10, 10), (20, 20), (30, 30), (1, 0), (2, 0), (3, 0)]);
	let mut phragmen_result = elect_float(2, 2, candidates, voters, &power_of).unwrap();
	let winners = phragmen_result.clone().winners;
	let assignments = phragmen_result.clone().assignments;

	assert_eq_uvec!(winners, vec![(2, 40), (3, 50)]);
	assert_eq_uvec!(
		assignments,
		vec![
			(10, vec![(2, 1.0)]),
			(20, vec![(3, 1.0)]),
			(30, vec![(2, 0.5), (3, 0.5)]),
		]
	);

	let mut support_map = build_support_map_float(&mut phragmen_result, &power_of);

	assert_eq!(
		support_map.get(&2).unwrap(),
		&_Support {
			own: 0.0,
			total: 25.0,
			others: vec![(10u64, 10.0), (30u64, 15.0)]
		}
	);
	assert_eq!(
		support_map.get(&3).unwrap(),
		&_Support {
			own: 0.0,
			total: 35.0,
			others: vec![(20u64, 20.0), (30u64, 15.0)]
		}
	);

	equalize_float(phragmen_result.assignments, &mut support_map, 0.0, 2, power_of);

	assert_eq!(
		support_map.get(&2).unwrap(),
		&_Support {
			own: 0.0,
			total: 30.0,
			others: vec![(10u64, 10.0), (30u64, 20.0)]
		}
	);
	assert_eq!(
		support_map.get(&3).unwrap(),
		&_Support {
			own: 0.0,
			total: 30.0,
			others: vec![(20u64, 20.0), (30u64, 10.0)]
		}
	);
}

#[test]
fn phragmen_poc_works() {
	let candidates = vec![1, 2, 3];
	let voters = vec![(10, vec![1, 2]), (20, vec![1, 3]), (30, vec![2, 3])];

	let PhragmenResult { winners, assignments } = elect::<_, Output, _>(
		2,
		2,
		candidates,
		voters,
		create_power_of(&[(10, 10), (20, 20), (30, 30)]),
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
	let power_of = create_power_of(&[(10, 1_000), (20, 1_000), (30, 1_000), (40, 1_000), (2, 500), (4, 500)]);

	run_and_compare(candidates, voters, power_of, 2, 2);
}

#[test]
fn phragmen_poc_3_works() {
	let candidates = vec![10, 20, 30];
	let voters = vec![(2, vec![10, 20, 30]), (4, vec![10, 20, 40])];
	let power_of = create_power_of(&[(10, 1_000), (20, 1_000), (30, 1_000), (2, 50), (4, 1_000)]);

	run_and_compare(candidates, voters, power_of, 2, 2);
}

#[test]
fn phragmen_accuracy_on_large_scale_only_validators() {
	// because of this particular situation we had per_u128 and now rational128. In practice, a
	// candidate can have the maximum amount of tokens, and also supported by the maximum.
	let candidates = vec![1, 2, 3, 4, 5];
	let power_of = create_power_of(&[
		(1, (u32::max_value() - 1).into()),
		(2, (u32::max_value() - 4).into()),
		(3, (u32::max_value() - 5).into()),
		(4, (u32::max_value() - 3).into()),
		(5, (u32::max_value() - 2).into()),
	]);

	let PhragmenResult { winners, assignments } = elect::<_, Output, _>(
		2,
		2,
		candidates.clone(),
		auto_generate_self_voters(&candidates),
		power_of,
	)
	.unwrap();

	assert_eq_uvec!(winners, vec![(1, 4_294_967_294_u64), (5, 4_294_967_293_u64)]);
	assert_eq!(assignments.len(), 2);
	check_assignments(assignments);
}

#[test]
fn phragmen_accuracy_on_large_scale_validators_and_nominators() {
	let candidates = vec![1, 2, 3, 4, 5];
	let mut voters = vec![(13, vec![1, 3, 5]), (14, vec![2, 4])];
	voters.extend(auto_generate_self_voters(&candidates));
	let power_of = create_power_of(&[
		(1, (u32::max_value() - 1).into()),
		(2, (u32::max_value() - 4).into()),
		(3, (u32::max_value() - 5).into()),
		(4, (u32::max_value() - 3).into()),
		(5, (u32::max_value() - 2).into()),
		(13, (u32::max_value() - 10).into()),
		(14, u32::max_value().into()),
	]);

	let PhragmenResult { winners, assignments } = elect::<_, Output, _>(2, 2, candidates, voters, power_of).unwrap();

	assert_eq_uvec!(winners, vec![(2, 8_589_934_586_u64), (1, 8_589_934_579_u64)]);
	assert_eq!(
		assignments,
		vec![
			(13, vec![(1, Perbill::one())]),
			(14, vec![(2, Perbill::one())]),
			(1, vec![(1, Perbill::one())]),
			(2, vec![(2, Perbill::one())]),
		]
	);
	check_assignments(assignments);
}

#[test]
fn phragmen_accuracy_on_small_scale_self_vote() {
	let candidates = vec![40, 10, 20, 30];
	let voters = auto_generate_self_voters(&candidates);
	let power_of = create_power_of(&[(40, 0), (10, 1), (20, 2), (30, 1)]);

	let PhragmenResult {
		winners,
		assignments: _,
	} = elect::<_, Output, _>(3, 3, candidates, voters, power_of).unwrap();

	assert_eq_uvec!(winners, vec![(20, 2), (10, 1), (30, 1)]);
}

#[test]
fn phragmen_accuracy_on_small_scale_no_self_vote() {
	let candidates = vec![40, 10, 20, 30];
	let voters = vec![(1, vec![10]), (2, vec![20]), (3, vec![30]), (4, vec![40])];
	let power_of = create_power_of(&[
		(40, 1000), // don't care
		(10, 1000), // don't care
		(20, 1000), // don't care
		(30, 1000), // don't care
		(4, 0),
		(1, 1),
		(2, 2),
		(3, 1),
	]);

	let PhragmenResult {
		winners,
		assignments: _,
	} = elect::<_, Output, _>(3, 3, candidates, voters, power_of).unwrap();

	assert_eq_uvec!(winners, vec![(20, 2), (10, 1), (30, 1)]);
}

#[test]
fn phragmen_large_scale_test() {
	let candidates = vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24];
	let mut voters = vec![(50, vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24])];
	voters.extend(auto_generate_self_voters(&candidates));
	let power_of = create_power_of(&[
		(2, 1),
		(4, 10),
		(6, 100),
		(8, 10_100),
		(10, 10_200),
		(12, 10_300),
		(14, 40_000),
		(16, 41_000),
		(18, 1_800_000),
		(20, 2_000_000),
		(22, 50_100_000),
		(24, 50_200_000),
		(50, 99_000_000),
	]);

	let PhragmenResult { winners, assignments } = elect::<_, Output, _>(2, 2, candidates, voters, power_of).unwrap();

	assert_eq_uvec!(winners, vec![(24, 149_200_000_u64), (22, 149_100_000_u64)]);
	check_assignments(assignments);
}

#[test]
fn phragmen_large_scale_test_2() {
	let nom_budget: u64 = 999_999_990;
	let c_budget: u64 = 1;

	let candidates = vec![2, 4];
	let mut voters = vec![(50, vec![2, 4])];
	voters.extend(auto_generate_self_voters(&candidates));

	let power_of = create_power_of(&[(2, c_budget as _), (4, c_budget as _), (50, nom_budget as _)]);

	let PhragmenResult { winners, assignments } = elect::<_, Output, _>(2, 2, candidates, voters, power_of).unwrap();

	assert_eq_uvec!(winners, vec![(2, 999_999_991_u64), (4, 999_999_991_u64)]);
	assert_eq!(
		assignments,
		vec![
			(
				50,
				vec![
					(2, Perbill::from_parts(500_000_001)),
					(4, Perbill::from_parts(499_999_999))
				]
			),
			(2, vec![(2, Perbill::one())]),
			(4, vec![(4, Perbill::one())]),
		],
	);
	check_assignments(assignments);
}

#[test]
fn phragmen_linear_equalize() {
	let candidates = vec![11, 21, 31, 41, 51, 61, 71];
	let voters = vec![
		(2, vec![11]),
		(4, vec![11, 21]),
		(6, vec![21, 31]),
		(8, vec![31, 41]),
		(110, vec![41, 51]),
		(120, vec![51, 61]),
		(130, vec![61, 71]),
	];
	let power_of = create_power_of(&[
		(11, 1000),
		(21, 1000),
		(31, 1000),
		(41, 1000),
		(51, 1000),
		(61, 1000),
		(71, 1000),
		(2, 2000),
		(4, 1000),
		(6, 1000),
		(8, 1000),
		(110, 1000),
		(120, 1000),
		(130, 1000),
	]);

	run_and_compare(candidates, voters, power_of, 2, 2);
}

#[test]
fn elect_has_no_entry_barrier() {
	let candidates = vec![10, 20, 30];
	let voters = vec![(1, vec![10]), (2, vec![20])];
	let power_of = create_power_of(&[(1, 10), (2, 10)]);

	let PhragmenResult {
		winners,
		assignments: _,
	} = elect::<_, Output, _>(3, 3, candidates, voters, power_of).unwrap();

	// 30 is elected with stake 0. The caller is responsible for stripping this.
	assert_eq_uvec!(winners, vec![(10, 10), (20, 10), (30, 0),]);
}

#[test]
fn minimum_to_elect_is_respected() {
	let candidates = vec![10, 20, 30];
	let voters = vec![(1, vec![10]), (2, vec![20])];
	let power_of = create_power_of(&[(1, 10), (2, 10)]);

	let maybe_result = elect::<_, Output, _>(10, 10, candidates, voters, power_of);

	assert!(maybe_result.is_none());
}

// FIXME
// #[test]
// fn self_votes_should_be_kept() {
// 	let candidates = vec![5, 10, 20, 30];
// 	let voters = vec![
// 		(5, vec![5]),
// 		(10, vec![10]),
// 		(20, vec![20]),
// 		(1, vec![10, 20])
// 	];
// 	let stake_of = create_stake_of(&[
// 		(5, 5),
// 		(10, 10),
// 		(20, 20),
// 		(1, 8),
// 	]);
//
// 	let result = elect::<_, _, _, TestCurrencyToVote, Output>(
// 		2,
// 		2,
// 		candidates,
// 		voters,
// 		&stake_of,
// 	).unwrap();
//
// 	assert_eq!(result.winners, vec![(20, 28), (10, 18)]);
// 	assert_eq!(
// 		result.assignments,
// 		vec![
// 			(10, vec![(10, Perbill::from_percent(100))]),
// 			(20, vec![(20, Perbill::from_percent(100))]),
// 			(1, vec![
// 				(10, Perbill::from_percent(50)),
// 				(20, Perbill::from_percent(50))
// 			]
// 			)
// 		],
// 	);
//
// 	let mut supports = build_support_map::<
// 		Balance,
// 		AccountId,
// 		_,
// 		TestCurrencyToVote,
// 		Output,
// 	>(
// 		&result.winners.into_iter().map(|(who, _)| who).collect(),
// 		&result.assignments,
// 		&stake_of
// 	);
//
// 	assert_eq!(supports.get(&5u64), None);
// 	assert_eq!(
// 		supports.get(&10u64).unwrap(),
// 		&Support { total: 14u128, voters: vec![(10u64, 10u128), (1u64, 4u128)] },
// 	);
// 	assert_eq!(
// 		supports.get(&20u64).unwrap(),
// 		&Support { total: 24u128, voters: vec![(20u64, 20u128), (1u64, 4u128)] },
// 	);
//
// 	let assignments = result.assignments;
// 	let mut staked_assignments
// 		: Vec<(AccountId, Vec<PhragmenStakedAssignment<AccountId>>)>
// 		= Vec::with_capacity(assignments.len());
// 	for (n, assignment) in assignments.iter() {
// 		let mut staked_assignment
// 			: Vec<PhragmenStakedAssignment<AccountId>>
// 			= Vec::with_capacity(assignment.len());
// 		let stake = stake_of(&n);
// 		for (c, per_thing) in assignment.iter() {
// 			let vote_stake = *per_thing * stake;
// 			staked_assignment.push((c.clone(), vote_stake));
// 		}
// 		staked_assignments.push((n.clone(), staked_assignment));
// 	}
//
// 	equalize::<Balance, AccountId, TestCurrencyToVote, _>(staked_assignments, &mut supports, 0, 2usize, &stake_of);
//
// 	assert_eq!(
// 		supports.get(&10u64).unwrap(),
// 		&Support { total: 18u128, voters: vec![(10u64, 10u128), (1u64, 8u128)] },
// 	);
// 	assert_eq!(
// 		supports.get(&20u64).unwrap(),
// 		&Support { total: 20u128, voters: vec![(20u64, 20u128)] },
// 	);
// }
