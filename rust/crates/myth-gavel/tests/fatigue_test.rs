use myth_common::{Enforcement, SessionId};
use myth_gavel::FatigueTracker;

#[test]
fn advisory_caps_at_two_then_downgrades_to_note() {
    let mut ft = FatigueTracker::new();
    let sid = SessionId::new();
    assert_eq!(ft.register(sid, Enforcement::Advisory), Enforcement::Advisory);
    assert_eq!(ft.register(sid, Enforcement::Advisory), Enforcement::Advisory);
    assert_eq!(ft.register(sid, Enforcement::Advisory), Enforcement::Note);
    assert_eq!(ft.register(sid, Enforcement::Advisory), Enforcement::Note);
}

#[test]
fn caution_caps_at_three_then_downgrades_to_note() {
    let mut ft = FatigueTracker::new();
    let sid = SessionId::new();
    for _ in 0..3 {
        assert_eq!(ft.register(sid, Enforcement::Caution), Enforcement::Caution);
    }
    assert_eq!(ft.register(sid, Enforcement::Caution), Enforcement::Note);
}

#[test]
fn warn_caps_at_two_then_downgrades_to_caution() {
    let mut ft = FatigueTracker::new();
    let sid = SessionId::new();
    assert_eq!(ft.register(sid, Enforcement::Warn), Enforcement::Warn);
    assert_eq!(ft.register(sid, Enforcement::Warn), Enforcement::Warn);
    assert_eq!(ft.register(sid, Enforcement::Warn), Enforcement::Caution);
}

#[test]
fn strike_and_seal_pass_through_indefinitely() {
    let mut ft = FatigueTracker::new();
    let sid = SessionId::new();
    for _ in 0..100 {
        assert_eq!(ft.register(sid, Enforcement::Strike), Enforcement::Strike);
        assert_eq!(ft.register(sid, Enforcement::Seal), Enforcement::Seal);
    }
}

#[test]
fn sessions_are_independent() {
    let mut ft = FatigueTracker::new();
    let a = SessionId::new();
    let b = SessionId::new();
    for _ in 0..2 {
        assert_eq!(ft.register(a, Enforcement::Advisory), Enforcement::Advisory);
    }
    // a exhausted, but b should still have its full quota.
    assert_eq!(ft.register(a, Enforcement::Advisory), Enforcement::Note);
    assert_eq!(ft.register(b, Enforcement::Advisory), Enforcement::Advisory);
}

#[test]
fn dismiss_and_note_never_touch_counters() {
    let mut ft = FatigueTracker::new();
    let sid = SessionId::new();
    for _ in 0..50 {
        assert_eq!(ft.register(sid, Enforcement::Dismiss), Enforcement::Dismiss);
        assert_eq!(ft.register(sid, Enforcement::Note), Enforcement::Note);
    }
    let snap = ft.snapshot(sid).unwrap();
    assert_eq!(snap.advisory, 0);
    assert_eq!(snap.caution, 0);
    assert_eq!(snap.warn, 0);
}
