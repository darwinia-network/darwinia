
#![cfg(test)]
use sr_io::{with_externalities, TestExternalities};
use primitives::H256;
use sr_primitives::traits::{BlakeTwo256, IdentityLookup, OnFinalize, Header as HeaderT};
use sr_primitives::testing::Header;
use support::{assert_ok, impl_outer_origin, parameter_types};
use std::cell::RefCell;
use finality_tracker::*;

#[derive(Clone, PartialEq, Debug)]
pub struct StallEvent {
    at: u64,
    further_wait: u64,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

impl_outer_origin! {
		pub enum Origin for Test {}
	}

thread_local! {
		static NOTIFICATIONS: RefCell<Vec<StallEvent>> = Default::default();
	}

pub struct StallTracker;
impl OnFinalizationStalled<u64> for StallTracker {
    fn on_stalled(further_wait: u64, _median: u64) {
        let now = System::block_number();
        NOTIFICATIONS.with(|v| v.borrow_mut().push(StallEvent { at: now, further_wait }));
    }
}


impl system::Trait for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<u64>;
    type Header = Header;
    type Event = ();
}
parameter_types! {
		pub const WindowSize: u64 = 11;
		pub const ReportLatency: u64 = 100;
	}
impl finality_tracker::Trait for Test {
    type OnFinalizationStalled = StallTracker;
    type WindowSize = WindowSize;
    type ReportLatency = ReportLatency;
}

type System = system::Module<Test>;
type FinalityTracker = finality_tracker::Module<Test>;



#[test]
fn notifies_when_stalled() {
    let t = system::GenesisConfig::default().build_storage::<Test>().unwrap().0;
    with_externalities(&mut TestExternalities::new(t), || {
        let mut parent_hash = System::parent_hash();
        for i in 2..106 {
            System::initialize(&i, &parent_hash, &Default::default(), &Default::default());
            FinalityTracker::on_finalize(i);
            let hdr = System::finalize();
            parent_hash = hdr.hash();
        }

        assert_eq!(
            NOTIFICATIONS.with(|n| n.borrow().clone()),
            vec![StallEvent { at: 105, further_wait: 10 }]
        )
    });
}

#[test]
fn recent_notifications_prevent_stalling() {
    let t = system::GenesisConfig::default().build_storage::<Test>().unwrap().0;
    with_externalities(&mut TestExternalities::new(t), || {
        let mut parent_hash = System::parent_hash();
        for i in 2..106 {
            System::initialize(&i, &parent_hash, &Default::default(), &Default::default());
            assert_ok!(FinalityTracker::dispatch(
					Call::final_hint(i-1),
					Origin::NONE,
				));
            FinalityTracker::on_finalize(i);
            let hdr = System::finalize();
            parent_hash = hdr.hash();
        }

        assert!(NOTIFICATIONS.with(|n| n.borrow().is_empty()));
    });
}
