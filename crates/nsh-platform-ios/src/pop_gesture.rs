//! Saves UIKit's enabled state separately for each recognizer NSH pauses.

pub(crate) struct PopGestureOverrides<G> {
    saved: Vec<(G, bool)>,
}

impl<G: PartialEq> PopGestureOverrides<G> {
    pub(crate) fn new() -> Self {
        Self { saved: Vec::new() }
    }

    /// Returns an enabled value to write only when NSH must change it.
    pub(crate) fn update(&mut self, gesture: G, allowed: bool, enabled: bool) -> Option<bool> {
        let index = self.saved.iter().position(|(g, _)| *g == gesture);
        if allowed {
            let (_, original) = self.saved.swap_remove(index?);
            (original != enabled).then_some(original)
        } else {
            if index.is_none() {
                self.saved.push((gesture, enabled));
            }
            enabled.then_some(false)
        }
    }
}
