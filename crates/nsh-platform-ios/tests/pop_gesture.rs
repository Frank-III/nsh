//! Exercise the production restore decision without requiring UIKit on the host.
#[path = "../src/pop_gesture.rs"]
mod pop_gesture;

use pop_gesture::PopGestureOverrides;

#[test]
fn allowed_never_changes_an_untouched_recognizer() {
    let mut state = PopGestureOverrides::new();
    assert_eq!(state.update(1, true, false), None);
    assert_eq!(state.update(1, true, true), None);
}

#[test]
fn repeated_pauses_preserve_the_first_snapshot_per_recognizer() {
    let mut state = PopGestureOverrides::new();
    assert_eq!(state.update(1, false, true), Some(false));
    assert_eq!(state.update(2, false, false), None);
    assert_eq!(state.update(1, false, false), None);
    // UIKit may update a recognizer during a transition; keep the snapshot.
    assert_eq!(state.update(2, false, true), Some(false));
    assert_eq!(state.update(1, true, false), Some(true));
    assert_eq!(state.update(2, true, false), None);
    // Restoring relinquishes ownership to UIKit again.
    assert_eq!(state.update(1, true, false), None);
    assert_eq!(state.update(2, true, true), None);
}

#[test]
fn a_new_pause_captures_the_new_uikit_state() {
    let mut state = PopGestureOverrides::new();
    assert_eq!(state.update(1, false, true), Some(false));
    assert_eq!(state.update(1, true, false), Some(true));
    assert_eq!(state.update(1, false, false), None);
    assert_eq!(state.update(1, true, true), Some(false));
}

#[test]
fn a_replacement_recognizer_does_not_inherit_another_snapshot() {
    let mut state = PopGestureOverrides::new();
    assert_eq!(state.update(1, false, true), Some(false));
    assert_eq!(state.update(2, true, false), None);
    assert_eq!(state.update(2, false, false), None);
    assert_eq!(state.update(2, true, true), Some(false));
    assert_eq!(state.update(1, true, false), Some(true));
}
