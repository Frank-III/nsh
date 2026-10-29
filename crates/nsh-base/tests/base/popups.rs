//! The presence of `Popover`, a desktop `Menu` with its `Submenu` and
//! `Tooltip`: opened, a popup has its starting look in its first frame;
//! closed, it stays with its ending look for as long as its transitions
//! then run, and `onClose` comes after; opened again on its way out it is
//! taken back; with reduced motion, or no ending look, it goes at once and
//! nothing more is sent. On a manual clock: time moves only as the test
//! says. On touch screens a popup is above the backdrop that takes the
//! presses outside it.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use nsh_base::{MenuEvent, PopoverEvent, menu, menu_item, popover, sheet, submenu, tooltip};
use nsh_core::{
    EventKind, EventPayload, MountItem, NodeId, ROOT_ID, Size,
    element::Root,
    platform::{Idiom, Os},
    prelude::*,
};
use nsh_engine::EngineConfig;
use nsh_testing::Headless;

/// What the platform was sent: each view's parent and test id, the
/// opacities of its styles in order, whether it takes presses, whether
/// it is hidden from assistive technologies, and the root view's children
/// in their z order (the last on top).
#[derive(Default)]
struct Seen {
    parents: HashMap<NodeId, NodeId>,
    root_children: Vec<NodeId>,
    test_ids: HashMap<String, NodeId>,
    styles: HashMap<NodeId, Vec<f32>>,
    inert: HashMap<NodeId, bool>,
    hidden: HashMap<NodeId, bool>,
    frames: HashMap<NodeId, Bounds<Points>>,
    content_sizes: HashMap<NodeId, Size<Points>>,
}

impl Seen {
    /// The view around the one with test id `id` (the popup part around
    /// the app's content).
    fn popup(&self, id: &str) -> NodeId {
        self.parents[&self.test_ids[id]]
    }
}

struct App {
    app: Headless,
    seen: Arc<Mutex<Seen>>,
}

impl App {
    /// The opacities the popup part around `id` was sent.
    fn popup_styles(&self, id: &str) -> Vec<f32> {
        let seen = self.seen.lock().unwrap();
        seen.styles
            .get(&seen.popup(id))
            .cloned()
            .unwrap_or_default()
    }

    /// The opacity the popup part around `id` was last sent.
    fn popup_opacity(&self, id: &str) -> f32 {
        *self.popup_styles(id).last().expect("styled")
    }

    /// Whether the popup part around `id` takes no presses.
    fn popup_inert(&self, id: &str) -> bool {
        let seen = self.seen.lock().unwrap();
        seen.inert.get(&seen.popup(id)).copied().unwrap_or(false)
    }
}

/// `root` on a Mac, on a manual clock; `reduce_motion`: the user asked for
/// less motion.
fn start(root: Root, reduce_motion: bool) -> App {
    start_on(root, reduce_motion, Os::MacOs, Idiom::Desktop)
}

/// [`start`] on `os` and `idiom`.
fn start_on(root: Root, reduce_motion: bool, os: Os, idiom: Idiom) -> App {
    let mut config = EngineConfig {
        viewport: Size::new(px(390.0), px(844.0)),
        clock: nsh_engine::Clock::manual(std::time::SystemTime::now()),
        ..EngineConfig::new(root)
    };
    config.environment.os = os;
    config.environment.idiom = idiom;
    config.environment.reduce_motion = reduce_motion;
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
                    seen.inert
                        .insert(*id, props.pointer_events == PointerEvents::None);
                    seen.hidden.insert(*id, props.accessibility.hidden);
                }
                MountItem::Insert {
                    parent,
                    child,
                    before,
                } => {
                    seen.parents.insert(*child, *parent);
                    seen.root_children.retain(|c| c != child);
                    if *parent == ROOT_ID {
                        let at = before
                            .and_then(|b| seen.root_children.iter().position(|&c| c == b))
                            .unwrap_or(seen.root_children.len());
                        seen.root_children.insert(at, *child);
                    }
                }
                MountItem::Remove { id } => {
                    seen.root_children.retain(|c| c != id);
                }
                MountItem::ScrollContent { id, size } => {
                    seen.content_sizes.insert(*id, *size);
                }
                MountItem::Frame { id, frame } => {
                    seen.frames.insert(*id, *frame);
                }
                MountItem::ViewStyle { id, style } => {
                    seen.styles.entry(*id).or_default().push(style.opacity);
                }
                _ => {}
            }
        }
    });
    app.first_render().unwrap();
    App { app, seen }
}

#[derive(Clone, PartialEq, Props)]
struct PopoverProps {
    /// `popupStartingClass` / `popupEndingClass` set.
    looks: bool,
    /// The trigger at the bottom of the window, with no room below it
    /// (what the screen shows stays above it: the driver sees only views
    /// inside the window).
    low: bool,
}

/// A popover whose popup fades for 200 ms between its look and its
/// starting and ending looks, with its lifecycle and the side it went on
/// on screen.
#[component]
fn PopoverScreen(props: PopoverProps) -> impl IntoElement {
    let life = signal(Vec::<&str>::new());
    let side = signal(None::<Placement>);
    let looks = props.looks;
    view()
        .pl(px(40.0))
        .child(text(move || format!("Life: {}", life.get().join(" "))))
        .child(text(move || format!("Side: {:?}", side.get())))
        .child(view().h(px(if props.low { 770.0 } else { 60.0 })))
        .child(
            popover()
                .popup_class("transition-opacity duration-200")
                .popup_starting_class(looks.then(|| SharedString::from("opacity-0")))
                .popup_ending_class(looks.then(|| SharedString::from("opacity-0")))
                .on_open(move || life.update(|l| l.push("open")))
                .on_close(move || life.update(|l| l.push("close")))
                .on_position(move |e: &PopoverEvent| {
                    if let PopoverEvent::Position(placement, ..) = e {
                        side.set(Some(*placement));
                    }
                })
                .trigger(
                    view()
                        .test_id("trigger")
                        .w(px(100.0))
                        .h(px(30.0))
                        .child(text("Details")),
                )
                .child(view().test_id("body").h(px(40.0)).child(text("Body"))),
        )
}

fn popover_app(looks: bool, reduce_motion: bool) -> App {
    start(
        Root::new::<PopoverScreen>(move || PopoverProps { looks, low: false }),
        reduce_motion,
    )
}

#[test]
fn a_popover_has_its_starting_look_in_its_first_frame() {
    let mut app = popover_app(true, false);
    app.app.press("id:trigger").unwrap();
    app.app.wait_for("Body").unwrap();
    assert_eq!(
        app.popup_styles("body").first(),
        Some(&0.0),
        "made with its starting look"
    );
    app.app.run_for(Duration::from_millis(50)).unwrap();
    assert_eq!(
        app.popup_opacity("body"),
        1.0,
        "its look from its first frame on"
    );
}

#[test]
fn a_popover_stays_with_its_ending_look_until_its_exit_ran() {
    let mut app = popover_app(true, false);
    app.app.press("id:trigger").unwrap();
    app.app.run_for(Duration::from_millis(50)).unwrap();
    app.app.wait_for("Life: open").unwrap();
    app.app.wait_for("Side: Some(Bottom)").unwrap();
    app.app.press("id:trigger").unwrap();
    app.app.run_for(Duration::from_millis(100)).unwrap();
    assert!(app.app.shows("Body"), "kept while it goes");
    assert_eq!(app.popup_opacity("body"), 0.0, "with its ending look");
    assert!(app.popup_inert("body"), "taking no presses");
    assert!(app.app.shows("Life: open"), "not closed yet");
    app.app.run_for(Duration::from_millis(150)).unwrap();
    assert!(!app.app.shows("Body"), "gone once its 200 ms ran");
    app.app.wait_for("Life: open close").unwrap();
}

#[test]
fn reopening_a_popover_on_its_way_out_takes_it_back() {
    let mut app = popover_app(true, false);
    app.app.press("id:trigger").unwrap();
    app.app.run_for(Duration::from_millis(50)).unwrap();
    app.app.press("id:trigger").unwrap();
    app.app.run_for(Duration::from_millis(50)).unwrap();
    app.app.press("id:trigger").unwrap();
    app.app.run_for(Duration::from_millis(500)).unwrap();
    assert!(app.app.shows("Body"), "an old exit does not take it away");
    assert_eq!(app.popup_opacity("body"), 1.0, "back to its look");
    assert!(
        app.app.shows("Life: open"),
        "never gone: neither closed nor opened again"
    );
}

#[test]
fn with_reduced_motion_a_popover_goes_at_once() {
    let mut app = popover_app(true, true);
    app.app.press("id:trigger").unwrap();
    app.app.wait_for("Body").unwrap();
    assert_eq!(app.popup_opacity("body"), 1.0, "no starting look");
    app.app.press("id:trigger").unwrap();
    assert!(!app.app.shows("Body"));
    app.app.wait_for("Life: open close").unwrap();
}

#[test]
fn a_popover_without_looks_goes_at_once_and_sends_nothing_more() {
    let mut app = popover_app(false, false);
    app.app.press("id:trigger").unwrap();
    app.app.run_for(Duration::from_millis(50)).unwrap();
    let styled = app.popup_styles("body").len();
    app.app.press("id:trigger").unwrap();
    assert!(!app.app.shows("Body"), "gone in the turn that closed it");
    app.app.wait_for("Life: open close").unwrap();
    assert_eq!(
        app.popup_styles("body").len(),
        styled,
        "no style sent to the popup as it closes"
    );
}

#[test]
fn a_popover_reports_the_side_it_went_on() {
    let mut app = start(
        Root::new::<PopoverScreen>(|| PopoverProps {
            looks: false,
            low: true,
        }),
        false,
    );
    app.app.press("id:trigger").unwrap();
    app.app.wait_for("Side: Some(Top)").unwrap();
}

/// The overlays shown in the root view, bottom to top: its children after
/// the app's own view.
fn shown_overlays(app: &App) -> Vec<NodeId> {
    app.seen
        .lock()
        .unwrap()
        .root_children
        .iter()
        .skip(1)
        .copied()
        .collect()
}

/// The touch screens' popover backdrop: the overlay below the popup
/// around `body`, the only other one shown.
fn backdrop(app: &App) -> NodeId {
    let overlays = shown_overlays(app);
    let popup = app.seen.lock().unwrap().popup("body");
    assert_eq!(overlays.len(), 2, "the backdrop and the popup");
    assert_eq!(
        overlays.last(),
        Some(&popup),
        "the popup on top, so presses in it reach its content"
    );
    overlays[0]
}

impl App {
    /// Whether view `id` takes no presses.
    fn inert(&self, id: NodeId) -> bool {
        self.seen
            .lock()
            .unwrap()
            .inert
            .get(&id)
            .copied()
            .unwrap_or(false)
    }

    /// Whether view `id` is hidden from assistive technologies.
    fn hidden(&self, id: NodeId) -> bool {
        self.seen
            .lock()
            .unwrap()
            .hidden
            .get(&id)
            .copied()
            .unwrap_or(false)
    }

    /// A press on view `id`, as a platform sends it whatever takes the
    /// touches (a screen reader's activation).
    fn press_view(&mut self, id: NodeId) {
        self.app
            .engine()
            .control()
            .dispatch_event(id, EventKind::Press, EventPayload::None);
        self.app.run_for(Duration::from_millis(50)).unwrap();
    }
}

#[derive(Clone, PartialEq, Props)]
struct TouchPopoverProps {
    /// `closeOnPressOutside`.
    close_on_press_outside: bool,
}

/// A popover whose popup fades out for 200 ms, with its open changes on
/// screen.
#[component]
fn TouchPopoverScreen(props: TouchPopoverProps) -> impl IntoElement {
    let changes = signal(Vec::<String>::new());
    view()
        .pl(px(40.0))
        .child(text(move || {
            format!("Changes: {}", changes.get().join(", "))
        }))
        .child(view().h(px(60.0)))
        .child(
            popover()
                .close_on_press_outside(props.close_on_press_outside)
                .popup_class("transition-opacity duration-200")
                .popup_ending_class(Some(SharedString::from("opacity-0")))
                .on_open_change(move |e: &PopoverEvent| {
                    if let PopoverEvent::OpenChange(open, reason) = e {
                        changes.update(|c| c.push(format!("{open} {reason:?}")));
                    }
                })
                .trigger(
                    view()
                        .test_id("trigger")
                        .w(px(100.0))
                        .h(px(30.0))
                        .child(text("Details")),
                )
                .child(view().test_id("body").h(px(40.0)).child(text("Body"))),
        )
}

fn touch_popover_app(os: Os, close_on_press_outside: bool) -> App {
    let mut app = start_on(
        Root::new::<TouchPopoverScreen>(move || TouchPopoverProps {
            close_on_press_outside,
        }),
        false,
        os,
        Idiom::Phone,
    );
    app.app.press("id:trigger").unwrap();
    app.app.wait_for("Body").unwrap();
    app.app.run_for(Duration::from_millis(50)).unwrap();
    app
}

#[test]
fn on_a_phone_a_popover_is_above_its_backdrop() {
    for os in [Os::Ios, Os::Android] {
        let mut app = touch_popover_app(os, true);
        let below = backdrop(&app);
        assert!(!app.inert(below), "{os:?}: takes the presses outside");
        assert!(!app.hidden(below), "{os:?}: as it did before");
        app.press_view(below);
        assert!(
            app.app
                .shows("Changes: true TriggerPress, false PressOutside"),
            "{os:?}: a press on it closes the popover"
        );
        // On its way out: presses go through it, and screen readers do not
        // see it.
        assert!(app.app.shows("Body"), "{os:?}: kept while it goes");
        assert_eq!(backdrop(&app), below, "{os:?}: the same backdrop");
        assert!(app.inert(below), "{os:?}: inert while it goes");
        assert!(app.hidden(below), "{os:?}: hidden while it goes");
        app.press_view(below);
        assert!(
            app.app
                .shows("Changes: true TriggerPress, false PressOutside"),
            "{os:?}: no change from a press while it goes"
        );
        // Opened again on its way out: still on top, taking presses again.
        app.app.press("id:trigger").unwrap();
        app.app.run_for(Duration::from_millis(50)).unwrap();
        assert!(app.app.shows("Body"));
        let again = backdrop(&app);
        assert!(!app.inert(again), "{os:?}: takes presses once opened again");
        assert!(!app.hidden(again), "{os:?}: seen once opened again");
    }
}

#[test]
fn on_a_phone_a_popover_that_stays_on_presses_outside_lets_them_through() {
    for os in [Os::Ios, Os::Android] {
        let mut app = touch_popover_app(os, false);
        let below = backdrop(&app);
        assert!(app.inert(below), "{os:?}: presses go through it");
        assert!(app.hidden(below), "{os:?}: no element for screen readers");
        app.press_view(below);
        assert!(
            app.app.shows("Changes: true TriggerPress"),
            "{os:?}: a press on it does not close the popover"
        );
        assert!(app.app.shows("Body"), "{os:?}: still open");
    }
}

/// An open sheet.
#[component]
fn SheetScreen() -> impl IntoElement {
    view().child(
        sheet()
            .default_open(true)
            .child(view().test_id("body").h(px(200.0)).child(text("Body"))),
    )
}

#[test]
fn on_a_phone_a_sheet_is_above_its_backdrop() {
    let mut app = start_on(Root::of::<SheetScreen>(), false, Os::Ios, Idiom::Phone);
    app.app.wait_for("Body").unwrap();
    let overlays = shown_overlays(&app);
    let panel = app.seen.lock().unwrap().popup("body");
    assert_eq!(overlays.len(), 2, "the backdrop and the panel");
    assert_eq!(overlays.last(), Some(&panel), "the panel on top");
}

/// A desktop menu with a submenu, both fading for 200 ms on their way out.
#[component]
fn MenuScreen() -> impl IntoElement {
    let closed = signal(false);
    let side = signal(None::<Placement>);
    view()
        .pt(px(100.0))
        .pl(px(40.0))
        .child(
            menu()
                .popup_class("transition-opacity duration-200")
                .popup_ending_class("opacity-0")
                .on_close(move || closed.set(true))
                .on_position(move |e: &MenuEvent| {
                    if let MenuEvent::Position(placement, ..) = e {
                        side.set(Some(*placement));
                    }
                })
                .trigger(
                    view()
                        .test_id("file")
                        .w(px(80.0))
                        .h(px(30.0))
                        .child(text("File")),
                )
                .child(menu_item().child(view().test_id("new").h(px(30.0)).child(text("New"))))
                .child(
                    submenu()
                        .trigger(view().test_id("share").h(px(30.0)).child(text("Share")))
                        .child(
                            menu_item()
                                .child(view().test_id("mail").h(px(30.0)).child(text("Mail"))),
                        ),
                ),
        )
        .child(text(move || format!("Closed: {}", closed.get())))
        .child(text(move || format!("Side: {:?}", side.get())))
}

#[test]
fn a_desktop_menu_and_its_submenu_go_with_their_exit() {
    let mut app = start(Root::of::<MenuScreen>(), false);
    app.app.press("id:file").unwrap();
    app.app.wait_for("Side: Some(Bottom)").unwrap();
    app.app.press("id:share").unwrap();
    app.app.wait_for("Mail").unwrap();
    app.app.press("Mail").unwrap();
    app.app.run_for(Duration::from_millis(100)).unwrap();
    assert!(
        app.app.shows("New") && app.app.shows("Mail"),
        "kept while they go"
    );
    assert!(app.app.shows("Closed: false"), "not closed yet");
    app.app.run_for(Duration::from_millis(150)).unwrap();
    assert!(!app.app.shows("New") && !app.app.shows("Mail"));
    app.app.wait_for("Closed: true").unwrap();
}

#[derive(Clone, PartialEq, Props)]
struct TooltipProps {}

/// A tooltip the app shows and hides, fading for 200 ms on its way out.
#[component]
fn TooltipScreen(_props: TooltipProps) -> impl IntoElement {
    let open = signal(true);
    let closed = signal(false);
    view()
        .pt(px(100.0))
        .pl(px(40.0))
        .child(
            view()
                .test_id("toggle")
                .w(px(80.0))
                .h(px(30.0))
                .on_press(move || open.set(!open.get_untracked()))
                .tooltip(
                    tooltip()
                        .open(move || Some(open.get()))
                        .ending_class("transition-opacity duration-200 opacity-0")
                        .on_close(move || closed.set(true))
                        .child(text("Tip")),
                )
                .child(text("Toggle")),
        )
        .child(text(move || format!("Closed: {}", closed.get())))
}

#[test]
fn a_tooltip_stays_with_its_ending_look_until_its_exit_ran() {
    let mut app = start(Root::new::<TooltipScreen>(|| TooltipProps {}), false);
    app.app.wait_for("Tip").unwrap();
    app.app.press("id:toggle").unwrap();
    app.app.run_for(Duration::from_millis(100)).unwrap();
    assert!(app.app.shows("Tip"), "kept while it goes");
    assert!(app.app.shows("Closed: false"), "not closed yet");
    app.app.run_for(Duration::from_millis(150)).unwrap();
    assert!(!app.app.shows("Tip"));
    app.app.wait_for("Closed: true").unwrap();
}

#[component]
fn MaxHeightPopoverScreen() -> impl IntoElement {
    view().pt(px(200.0)).child(
        popover()
            .popup_class("w-[240px]")
            .trigger(view().test_id("trigger").h(px(44.0)).child(text("Open")))
            .child(
                scroll()
                    .test_id("list")
                    .max_h(px(300.0))
                    .children((0..30).map(|_| view().h(px(44.0)).child(text("Row")))),
            ),
    )
}

#[test]
fn a_popover_respects_its_scroll_childs_max_height() {
    for os in [Os::Ios, Os::Android] {
        let mut app = start_on(
            Root::of::<MaxHeightPopoverScreen>(),
            false,
            os,
            Idiom::Phone,
        );
        app.app.press("id:trigger").unwrap();
        app.app.run_for(Duration::from_millis(50)).unwrap();
        let id = app.seen.lock().unwrap().test_ids["list"];
        let bounds = app.seen.lock().unwrap().frames[&id];
        let content = app.seen.lock().unwrap().content_sizes[&id];
        assert_eq!(content.height, px(1320.0), "{os:?}: 30 full-height rows");
        assert_eq!(bounds.size.height, px(300.0), "{os:?}");
    }
}
