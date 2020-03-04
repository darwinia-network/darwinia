//! Mock file for phragmen.

#![cfg(test)]

use sp_runtime::{assert_eq_error_rate, traits::Member, PerThing, Perbill};
use sp_std::collections::btree_map::BTreeMap;

use crate::{elect, PhragmenAssignment, PhragmenResult, Power, Votes};

pub(crate) type AccountId = u64;

pub(crate) type _PhragmenAssignment<A> = (A, f64);
pub(crate) type _SupportMap<A> = BTreeMap<A, _Support<A>>;

#[derive(Default, Debug)]
pub(crate) struct _Candidate<A> {
	who: A,
	score: f64,
	approval_stake: f64,
	elected: bool,
}

#[derive(Default, Debug)]
pub(crate) struct _Voter<A> {
	who: A,
	edges: Vec<_Edge<A>>,
	budget: f64,
	load: f64,
}

#[derive(Default, Debug)]
pub(crate) struct _Edge<A> {
	who: A,
	load: f64,
	candidate_index: usize,
}

#[derive(Default, Debug, PartialEq)]
pub(crate) struct _Support<A> {
	pub own: f64,
	pub total: f64,
	pub others: Vec<_PhragmenAssignment<A>>,
}

#[derive(Debug, Clone)]
pub(crate) struct _PhragmenResult<A: Clone> {
	pub winners: Vec<(A, Votes)>,
	pub assignments: Vec<(A, Vec<_PhragmenAssignment<A>>)>,
}

pub(crate) fn create_power_of(stakes: &[(AccountId, Power)]) -> Box<dyn Fn(&AccountId) -> Power> {
	let mut storage = BTreeMap::<AccountId, Power>::new();
	stakes.iter().for_each(|s| {
		storage.insert(s.0, s.1);
	});
	let power_of = move |who: &AccountId| -> Power { storage.get(who).unwrap().to_owned() };
	Box::new(power_of)
}

pub(crate) fn auto_generate_self_voters<A: Clone>(candidates: &[A]) -> Vec<(A, Vec<A>)> {
	candidates.iter().map(|c| (c.clone(), vec![c.clone()])).collect()
}

pub(crate) fn check_assignments(assignments: Vec<(AccountId, Vec<PhragmenAssignment<AccountId, Perbill>>)>) {
	for (_, a) in assignments {
		let sum: u32 = a.iter().map(|(_, p)| p.deconstruct()).sum();
		assert_eq_error_rate!(sum, Perbill::ACCURACY, 5);
	}
}

pub(crate) fn run_and_compare(
	candidates: Vec<AccountId>,
	voters: Vec<(AccountId, Vec<AccountId>)>,
	power_of: Box<dyn Fn(&AccountId) -> Power>,
	to_elect: usize,
	min_to_elect: usize,
) {
	// run fixed point code.
	let PhragmenResult { winners, assignments } =
		elect::<_, Perbill, _>(to_elect, min_to_elect, candidates.clone(), voters.clone(), &power_of).unwrap();

	// run float poc code.
	let truth_value = elect_float(to_elect, min_to_elect, candidates, voters, &power_of).unwrap();

	assert_eq!(winners, truth_value.winners);

	for (nominator, assigned) in assignments.clone() {
		if let Some(float_assignments) = truth_value.assignments.iter().find(|x| x.0 == nominator) {
			for (candidate, per_thingy) in assigned {
				if let Some(float_assignment) = float_assignments.1.iter().find(|x| x.0 == candidate) {
					assert_eq_error_rate!(
						Perbill::from_fraction(float_assignment.1).deconstruct(),
						per_thingy.deconstruct(),
						1,
					);
				} else {
					panic!("candidate mismatch. This should never happen.")
				}
			}
		} else {
			panic!("nominator mismatch. This should never happen.")
		}
	}

	check_assignments(assignments);
}

pub(crate) fn elect_float<A, FS>(
	candidate_count: usize,
	minimum_candidate_count: usize,
	initial_candidates: Vec<A>,
	initial_voters: Vec<(A, Vec<A>)>,
	power_of: FS,
) -> Option<_PhragmenResult<A>>
where
	A: Default + Ord + Member + Copy,
	for<'r> FS: Fn(&'r A) -> Power,
{
	let mut elected_candidates: Vec<(A, Votes)>;
	let mut assigned: Vec<(A, Vec<_PhragmenAssignment<A>>)>;
	let mut c_idx_cache = BTreeMap::<A, usize>::new();
	let num_voters = initial_candidates.len() + initial_voters.len();
	let mut voters: Vec<_Voter<A>> = Vec::with_capacity(num_voters);

	let mut candidates = initial_candidates
		.into_iter()
		.enumerate()
		.map(|(idx, who)| {
			c_idx_cache.insert(who.clone(), idx);
			_Candidate {
				who,
				..Default::default()
			}
		})
		.collect::<Vec<_Candidate<A>>>();

	if candidates.len() < minimum_candidate_count {
		return None;
	}

	voters.extend(initial_voters.into_iter().map(|(who, votes)| {
		let voter_stake = power_of(&who) as f64;
		let mut edges: Vec<_Edge<A>> = Vec::with_capacity(votes.len());
		for v in votes {
			if let Some(idx) = c_idx_cache.get(&v) {
				candidates[*idx].approval_stake = candidates[*idx].approval_stake + voter_stake;
				edges.push(_Edge {
					who: v.clone(),
					candidate_index: *idx,
					..Default::default()
				});
			}
		}
		_Voter {
			who,
			edges,
			budget: voter_stake,
			load: 0f64,
		}
	}));

	let to_elect = candidate_count.min(candidates.len());
	elected_candidates = Vec::with_capacity(candidate_count);
	assigned = Vec::with_capacity(candidate_count);

	for _round in 0..to_elect {
		for c in &mut candidates {
			if !c.elected {
				c.score = 1.0 / c.approval_stake;
			}
		}
		for n in &voters {
			for e in &n.edges {
				let c = &mut candidates[e.candidate_index];
				if !c.elected && !(c.approval_stake == 0f64) {
					c.score += n.budget * n.load / c.approval_stake;
				}
			}
		}

		if let Some(winner) = candidates
			.iter_mut()
			.filter(|c| !c.elected)
			.min_by(|x, y| x.score.partial_cmp(&y.score).unwrap_or(sp_std::cmp::Ordering::Equal))
		{
			winner.elected = true;
			for n in &mut voters {
				for e in &mut n.edges {
					if e.who == winner.who {
						e.load = winner.score - n.load;
						n.load = winner.score;
					}
				}
			}

			elected_candidates.push((winner.who.clone(), winner.approval_stake as Votes));
		} else {
			break;
		}
	}

	for n in &mut voters {
		let mut assignment = (n.who.clone(), vec![]);
		for e in &mut n.edges {
			if let Some(c) = elected_candidates.iter().cloned().map(|(c, _)| c).find(|c| *c == e.who) {
				if c != n.who {
					let ratio = e.load / n.load;
					assignment.1.push((e.who.clone(), ratio));
				}
			}
		}
		if assignment.1.len() > 0 {
			assigned.push(assignment);
		}
	}

	Some(_PhragmenResult {
		winners: elected_candidates,
		assignments: assigned,
	})
}

pub(crate) fn build_support_map_float<FS>(
	result: &mut _PhragmenResult<AccountId>,
	power_of: FS,
) -> _SupportMap<AccountId>
where
	for<'r> FS: Fn(&'r AccountId) -> Power,
{
	let mut supports = <_SupportMap<AccountId>>::new();
	result
		.winners
		.iter()
		.map(|(e, _)| (e, power_of(e) as f64))
		.for_each(|(e, s)| {
			let item = _Support {
				own: s,
				total: s,
				..Default::default()
			};
			supports.insert(e.clone(), item);
		});

	for (n, assignment) in result.assignments.iter_mut() {
		for (c, r) in assignment.iter_mut() {
			let nominator_stake = power_of(n) as f64;
			let other_stake = nominator_stake * *r;
			if let Some(support) = supports.get_mut(c) {
				support.total = support.total + other_stake;
				support.others.push((n.clone(), other_stake));
			}
			*r = other_stake;
		}
	}
	supports
}

pub(crate) fn equalize_float<A, FS>(
	mut assignments: Vec<(A, Vec<_PhragmenAssignment<A>>)>,
	supports: &mut _SupportMap<A>,
	tolerance: f64,
	iterations: usize,
	power_of: FS,
) where
	for<'r> FS: Fn(&'r A) -> Power,
	A: Ord + Clone + std::fmt::Debug,
{
	for _i in 0..iterations {
		let mut max_diff = 0.0;
		for (voter, assignment) in assignments.iter_mut() {
			let voter_budget = power_of(&voter);
			let diff = do_equalize_float(voter, voter_budget, assignment, supports, tolerance);
			if diff > max_diff {
				max_diff = diff;
			}
		}

		if max_diff < tolerance {
			break;
		}
	}
}

pub(crate) fn do_equalize_float<A>(
	voter: &A,
	budget_balance: Power,
	elected_edges: &mut Vec<_PhragmenAssignment<A>>,
	support_map: &mut _SupportMap<A>,
	tolerance: f64,
) -> f64
where
	A: Ord + Clone,
{
	let budget = budget_balance as f64;
	if elected_edges.is_empty() {
		return 0.0;
	}

	let stake_used = elected_edges.iter().fold(0.0, |s, e| s + e.1);

	let backed_stakes_iter = elected_edges
		.iter()
		.filter_map(|e| support_map.get(&e.0))
		.map(|e| e.total);

	let backing_backed_stake = elected_edges
		.iter()
		.filter(|e| e.1 > 0.0)
		.filter_map(|e| support_map.get(&e.0))
		.map(|e| e.total)
		.collect::<Vec<f64>>();

	let mut difference;
	if backing_backed_stake.len() > 0 {
		let max_stake = backing_backed_stake
			.iter()
			.max_by(|x, y| x.partial_cmp(&y).unwrap_or(sp_std::cmp::Ordering::Equal))
			.expect("vector with positive length will have a max; qed");
		let min_stake = backed_stakes_iter
			.min_by(|x, y| x.partial_cmp(&y).unwrap_or(sp_std::cmp::Ordering::Equal))
			.expect("iterator with positive length will have a min; qed");

		difference = max_stake - min_stake;
		difference = difference + budget - stake_used;
		if difference < tolerance {
			return difference;
		}
	} else {
		difference = budget;
	}

	// Undo updates to support
	elected_edges.iter_mut().for_each(|e| {
		if let Some(support) = support_map.get_mut(&e.0) {
			support.total = support.total - e.1;
			support.others.retain(|i_support| i_support.0 != *voter);
		}
		e.1 = 0.0;
	});

	elected_edges.sort_unstable_by(|x, y| {
		support_map
			.get(&x.0)
			.and_then(|x| support_map.get(&y.0).and_then(|y| x.total.partial_cmp(&y.total)))
			.unwrap_or(sp_std::cmp::Ordering::Equal)
	});

	let mut cumulative_stake = 0.0;
	let mut last_index = elected_edges.len() - 1;
	elected_edges.iter_mut().enumerate().for_each(|(idx, e)| {
		if let Some(support) = support_map.get_mut(&e.0) {
			let stake = support.total;
			let stake_mul = stake * (idx as f64);
			let stake_sub = stake_mul - cumulative_stake;
			if stake_sub > budget {
				last_index = idx.checked_sub(1).unwrap_or(0);
				return;
			}
			cumulative_stake = cumulative_stake + stake;
		}
	});

	let last_stake = elected_edges[last_index].1;
	let split_ways = last_index + 1;
	let excess = budget + cumulative_stake - last_stake * (split_ways as f64);
	elected_edges.iter_mut().take(split_ways).for_each(|e| {
		if let Some(support) = support_map.get_mut(&e.0) {
			e.1 = excess / (split_ways as f64) + last_stake - support.total;
			support.total = support.total + e.1;
			support.others.push((voter.clone(), e.1));
		}
	});

	difference
}
