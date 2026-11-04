//! The sheets touch screens show over the page (a `Menu`'s and a
//! `Sheet`'s): their backdrop is no element for screen readers and their
//! panel is modal (`accessibilityViewIsModal`), so VoiceOver reads the
//! sheet's items rather than one unlabelled full-screen button over them
//! and the page behind. The escape gesture closes them (`onCancel`).

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use nsh_base::{menu, menu_item, sheet};
use nsh_core::{
    MountItem, NodeId, Size,
    element::Root,
    platform::{Idiom, Os},
    prelude::*,
};
use nsh_engine::EngineConfig;
use nsh_testing::Headless;

/// What the platform was sent: each view's parent, the views put in each
/// view in order, the test ids, and each view's last accessibility
/// hiding and modality.
#[derive(Default)]
struct Seen {
    parents: HashMap<NodeId, NodeId>,
    stacks: HashMap<NodeId, Vec<NodeId>>,
    test_ids: HashMap<String, NodeId>,
    hidden: HashMap<NodeId, bool>,
    modal: HashMap<NodeId, bool>,
}

impl Seen {
    /// The modal view around the one with test id `id`, and the view put
    /// in its parent just before it (the backdrop).
    fn panel_and_backdrop(&self, id: &str) -> (NodeId, NodeId) {
        let mut at = self.test_ids[id];
        while !self.modal.get(&at).copied().unwrap_or(false) {
            at = *self
                .parents
                .get(&at)
                .unwrap_or_else(|| panic!("no modal view around {id}"));
        }
        let stack = &self.stacks[&self.parents[&at]];
        let i = stack.iter().position(|v| *v == at).unwrap();
        assert!(i > 0, "a backdrop under the panel: {stack:?}");
        (at, stack[i - 1])
    }
}

fn start_on_phone(root: Root) -> (Headless, Arc<Mutex<Seen>>) {
    let mut config = EngineConfig {
        viewport: Size::new(px(390.0), px(844.0)),
        ..EngineConfig::new(root)
    };
    config.environment.os = Os::Ios;
    config.environment.idiom = Idiom::Phone;
    let mut app = Headless::start(config).expect("the engine starts");
    let seen = Arc::new(Mutex::new(Seen::default()));
    let into = seen.clone();
    app.observe(move |_, batch| {
        let mut seen = into.lock().unwrap();
        for item in &batch.items {
            match item {
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        seen.test_ids.insert(test_id.to_string(), *id);
                    }
                    seen.hidden.insert(*id, props.accessibility.hidden);
                    seen.modal.insert(*id, props.accessibility.view_is_modal);
                }
                MountItem::Insert {
                    parent,
                    child,
                    before,
                } => {
                    seen.parents.insert(*child, *parent);
                    let stack = seen.stacks.entry(*parent).or_default();
                    stack.retain(|id| id != child);
                    let at = before
                        .and_then(|b| stack.iter().position(|id| *id == b))
                        .unwrap_or(stack.len());
                    stack.insert(at, *child);
                }
                _ => {}
            }
        }
    });
    app.first_render().unwrap();
    (app, seen)
}

/// The sheet's backdrop is hidden from screen readers and the panel with
/// the item `id` is modal; the page's views are not.
fn assert_sheet_semantics(seen: &Seen, id: &str) {
    let (panel, backdrop) = seen.panel_and_backdrop(id);
    assert!(seen.modal[&panel], "the panel is modal");
    assert!(
        seen.hidden.get(&backdrop).copied().unwrap_or(false),
        "the backdrop is no element for screen readers"
    );
    assert!(
        !seen
            .hidden
            .get(&seen.test_ids[id])
            .copied()
            .unwrap_or(false),
        "the items stay elements"
    );
    let page = seen.test_ids["page"];
    assert!(!seen.modal.get(&page).copied().unwrap_or(false));
    assert!(!seen.hidden.get(&page).copied().unwrap_or(false));
}

#[component]
fn MenuScreen() -> impl IntoElement {
    view().test_id("page").child(
        menu()
            .trigger(view().test_id("more").h(px(44.0)).child(text("More")))
            .child(menu_item().child(view().test_id("rename").h(px(44.0)).child(text("Rename"))))
            .child(menu_item().child(view().test_id("delete").h(px(44.0)).child(text("Delete")))),
    )
}

#[test]
fn on_a_phone_a_menu_sheet_is_modal_for_screen_readers() {
    let (mut app, seen) = start_on_phone(Root::of::<MenuScreen>());
    app.press("id:more").unwrap();
    app.wait_for("Rename").unwrap();
    assert_sheet_semantics(&seen.lock().unwrap(), "rename");
}

#[component]
fn SheetScreen() -> impl IntoElement {
    view().test_id("page").child(
        sheet().open(|| Some(true)).child(
            view()
                .test_id("body")
                .h(px(200.0))
                .child(text("Sheet body")),
        ),
    )
}

#[test]
fn on_a_phone_a_sheet_is_modal_for_screen_readers() {
    let (mut app, seen) = start_on_phone(Root::of::<SheetScreen>());
    app.wait_for("Sheet body").unwrap();
    assert_sheet_semantics(&seen.lock().unwrap(), "body");
}
