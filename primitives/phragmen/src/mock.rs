#![cfg(test)]

use std::collections::BTreeMap;

use sp_runtime::{assert_eq_error_rate, traits::Member, Perbill};

use crate::{elect, PhragmenAssignment, PhragmenResult};

pub(crate) type AccountId = u64;
pub(crate) type Power = u32;

pub(crate) type _PhragmenAssignment<A> = (A, f64);

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
	pub winners: Vec<(A, Power)>,
	pub assignments: Vec<(A, Vec<_PhragmenAssignment<A>>)>,
}

pub(crate) fn create_stake_of(stakes: &[(AccountId, Power)]) -> Box<dyn Fn(&AccountId) -> Power> {
	let mut storage = BTreeMap::<AccountId, Power>::new();
	stakes.iter().for_each(|s| {
		storage.insert(s.0, s.1);
	});
	let stake_of = move |who: &AccountId| -> Power { storage.get(who).unwrap().to_owned() };
	Box::new(stake_of)
}

pub(crate) fn elect_float<A, FS>(
	candidate_count: usize,
	minimum_candidate_count: usize,
	initial_candidates: Vec<A>,
	initial_voters: Vec<(A, Vec<A>)>,
	stake_of: FS,
) -> Option<_PhragmenResult<A>>
where
	A: Default + Ord + Member + Copy,
	for<'r> FS: Fn(&'r A) -> Power,
{
	let mut elected_candidates: Vec<(A, Power)>;
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
		let voter_stake = stake_of(&who) as f64;
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

			elected_candidates.push((winner.who.clone(), winner.approval_stake as Power));
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

pub(crate) fn check_assignments(assignments: Vec<(AccountId, Vec<PhragmenAssignment<AccountId>>)>) {
	for (_, a) in assignments {
		let sum: u32 = a.iter().map(|(_, p)| p.deconstruct()).sum();
		assert_eq_error_rate!(sum, Perbill::accuracy(), 5);
	}
}

pub(crate) fn run_and_compare(
	candidates: Vec<AccountId>,
	voters: Vec<(AccountId, Vec<AccountId>)>,
	stake_of: Box<dyn Fn(&AccountId) -> Power>,
	to_elect: usize,
	min_to_elect: usize,
) {
	// run fixed point code.
	let PhragmenResult { winners, assignments } = elect::<_, _>(
		to_elect,
		min_to_elect,
		candidates.clone(),
		voters.clone(),
		&stake_of,
		1_000_000_000,
	)
	.unwrap();

	// run float poc code.
	let truth_value = elect_float(to_elect, min_to_elect, candidates, voters, &stake_of).unwrap();

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
