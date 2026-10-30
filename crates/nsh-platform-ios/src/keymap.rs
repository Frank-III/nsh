//! The app's keyboard shortcuts (`nsh/keymap`) on a hardware keyboard
//! (an iPad's, a Mac's under Catalyst, the simulator's): the table the
//! engine mounts ([`MountItem::Keymap`](nsh_core::MountItem::Keymap)),
//! matched against every key press on the spot.
//!
//! UIKit hands a hardware key to the first responder as a press
//! (`pressesBegan:withEvent:`), which passes what it does not handle up the
//! responder chain. The text inputs (`input::Field`, `input::Area`) look at
//! the table before UIKit's text system does, so a binding that fires in
//! text (`when: "always"`, or a keystroke that edits no text) takes the key
//! there and the input never sees it: no caret move, no line break, no
//! typed character. Presses nothing handled reach the surface's root view
//! (`surface::RootView`), which matches them as the desktops do when the
//! focus is on none of the app's inputs: a press from a hosted view (a web
//! view, a native component) counts as in text. The root and navigation
//! controllers' content roots become first responder only while no control
//! owns the keyboard. Window attachment, detachment and editing-end
//! notifications restore that fallback after UIKit completes a focus transfer.
//!
//! A key that fires a binding is consumed until it is released (its
//! release is consumed too) and reported to JS with the root's
//! `KeyCommand` event; JS runs the handler afterwards. Keys are named by
//! their USB HID usage (their place on the US layout) for the keys that
//! type nothing, and by the character they type without modifiers
//! otherwise, the US layout's of the same key when that is no ASCII, as on
//! the desktops. UIKit sends a held key's press once: a binding that
//! repeats (`repeat: true`) fires again on a timer of its own while the key
//! is held, at the system's usual pace. Fallback navigation keys use priority
//! UIKeyCommands instead: one dispatch per action, with no release timer.

use std::{
    cell::{Cell, RefCell},
    sync::Arc,
    time::Duration,
};

use nsh_core::{
    EventKind, EventPayload, PayloadValue, ROOT_ID,
    keymap::{Key, KeyMatch, Keymap, Keystroke, Modifiers},
};
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send,
    rc::{Retained, Weak},
    runtime::{AnyClass, AnyObject},
    sel,
};
use objc2_foundation::{NSArray, NSObject, NSString};
use objc2_ui_kit::{UIResponder, UIView, UIViewControllerTransitionCoordinator, UIWindow};

use crate::target::EventTarget;

// Diagnostic branch only. NSLog is variadic and is not bound by objc2-foundation.
#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    fn NSLog(format: *const NSString, ...);
}

fn debug_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        let enabled = std::env::var("NSH_KEY_DEBUG").is_ok_and(|v| v == "1");
        if enabled {
            debug_log("enabled NSH_KEY_DEBUG=1");
        }
        enabled
    })
}

fn debug_log(message: &str) {
    let format = NSString::from_str("nsh-keys: %@");
    let message = NSString::from_str(message);
    // SAFETY: constant format with exactly one NSString object argument.
    unsafe { NSLog(&*format, &*message as *const NSString) };
}

fn debug_class(object: Option<&AnyObject>) -> String {
    object.map_or_else(
        || "nil".into(),
        |o| o.class().name().to_string_lossy().into_owned(),
    )
}

fn debug_context(view: &UIView) -> String {
    let Some(window) = view.window() else {
        return "firstResponder=no-window focusedItem=no-window".into();
    };
    // SAFETY: same private UIWindow query as synth::first_responder, guarded
    // for OS versions without it. This branch is for local diagnostics only.
    let first: Option<Retained<AnyObject>> = unsafe {
        let supported: bool = msg_send![&window, respondsToSelector: sel!(firstResponder)];
        if supported {
            msg_send![&window, firstResponder]
        } else {
            None
        }
    };
    let focused: Option<Retained<AnyObject>> = unsafe {
        // SAFETY: optional public UIKit focus API, guarded for availability.
        AnyClass::get(c"UIFocusSystem").and_then(|class| {
            let supported: bool =
                msg_send![class, respondsToSelector: sel!(focusSystemForEnvironment:)];
            if !supported {
                return None;
            }
            let system: Option<Retained<AnyObject>> =
                msg_send![class, focusSystemForEnvironment: &*window];
            system.and_then(|system| msg_send![&system, focusedItem])
        })
    };
    format!(
        "firstResponder={} focusedItem={}",
        debug_class(first.as_deref()),
        debug_class(focused.as_deref())
    )
}

pub(crate) fn debug_commands(view: &UIView, commands: Option<&NSArray<AnyObject>>) {
    if !debug_enabled() {
        return;
    }
    let entries: Vec<_> = commands
        .into_iter()
        .flat_map(|a| a.iter())
        .map(|command| {
            // SAFETY: each returned element is a UIKeyCommand.
            let (input, flags): (Option<Retained<NSString>>, isize) = unsafe {
                (
                    msg_send![&command, input],
                    msg_send![&command, modifierFlags],
                )
            };
            format!("input={:?} flags={flags:#x}", input.map(|s| s.to_string()))
        })
        .collect();
    debug_log(&format!(
        "keyCommands who={} isFirstResponder={} count={} commands=[{}] {}",
        view.class().name().to_string_lossy(),
        view.isFirstResponder(),
        entries.len(),
        entries.join(", "),
        debug_context(view)
    ));
}

pub(crate) fn debug_presses(view: &UIView, presses: &AnyObject, consumed: bool) {
    if !debug_enabled() {
        return;
    }
    let entries: Vec<_> = keys(presses)
        .iter()
        .map(|key| {
            // SAFETY: UIKey properties, as in keystroke().
            let (characters, flags): (Option<Retained<NSString>>, isize) =
                unsafe { (msg_send![key, characters], msg_send![key, modifierFlags]) };
            format!(
                "code={} characters={:?} flags={flags:#x}",
                code(key),
                characters.map(|s| s.to_string())
            )
        })
        .collect();
    debug_log(&format!(
        "pressesBegan who={} isFirstResponder={} keys=[{}] consumed={} {}",
        view.class().name().to_string_lossy(),
        view.isFirstResponder(),
        entries.join(", "),
        consumed,
        debug_context(view)
    ));
}

/// How long a key is held before it repeats, and the time between repeats:
/// iOS's defaults for a hardware keyboard.
const REPEAT_DELAY: Duration = Duration::from_millis(400);
const REPEAT_INTERVAL: Duration = Duration::from_millis(100);

/// `UIKeyModifierFlags`.
const SHIFT: isize = 1 << 17;
const CONTROL: isize = 1 << 18;
const ALTERNATE: isize = 1 << 19;
const COMMAND: isize = 1 << 20;

/// A surface's table, and the keys it consumed that are still down.
#[derive(Default)]
pub(crate) struct KeymapState {
    table: RefCell<Option<Arc<Keymap>>>,
    /// Keys whose press the table consumed, by HID usage, with the binding
    /// that repeats while they are held, if any.
    held: RefCell<Vec<(isize, Option<u32>)>>,
    /// Bumped by every press and release: a repeat timer of an older
    /// generation stops.
    generation: Cell<u64>,
}

impl KeymapState {
    /// `MountItem::Keymap`: replaces the table.
    pub(crate) fn set(&self, keymap: Arc<Keymap>) {
        *self.table.borrow_mut() = (!keymap.is_empty()).then_some(keymap);
    }

    /// Whether the table has bindings: presses need no look otherwise.
    fn active(&self) -> bool {
        self.table.borrow().is_some() || !self.held.borrow().is_empty()
    }
}

/// The presses of a `UIPress` set, with their keys (`UIKey`); presses of
/// no key (a remote's, a game controller's) are left out.
fn keys(presses: &AnyObject) -> Vec<Retained<AnyObject>> {
    // SAFETY: `presses` is the `NSSet<UIPress *>` UIKit passes; `key` is
    // UIPress's (iOS 13.4, below the deployment target).
    unsafe {
        let all: Retained<NSArray<AnyObject>> = msg_send![presses, allObjects];
        all.iter()
            .filter_map(|press| msg_send![&*press, key])
            .collect()
    }
}

/// `UIKey.keyCode`: the key's USB HID usage.
fn code(key: &AnyObject) -> isize {
    // SAFETY: UIKey's property.
    unsafe { msg_send![key, keyCode] }
}

/// The keystroke of `key` (a `UIKey`), by its key on the US layout.
fn keystroke(key: &AnyObject) -> Option<Keystroke> {
    // SAFETY: UIKey's properties.
    let (flags, typed): (isize, Option<Retained<NSString>>) = unsafe {
        (
            msg_send![key, modifierFlags],
            msg_send![key, charactersIgnoringModifiers],
        )
    };
    let modifiers = Modifiers {
        ctrl: flags & CONTROL != 0,
        alt: flags & ALTERNATE != 0,
        shift: flags & SHIFT != 0,
        cmd: flags & COMMAND != 0,
    };
    let usage = u16::try_from(code(key)).ok()?;
    let key = match Key::from_hid_usage(usage)? {
        // A key that types a character: the one it types without
        // modifiers. UIKit may leave Shift applied (`?` for Shift+/): then
        // the key's own, which the US layout names.
        us @ Key::Char(c) if c != ' ' => match typed.and_then(|s| s.to_string().chars().next()) {
            Some(c) if c.is_ascii_alphabetic() => Key::Char(c.to_ascii_lowercase()),
            Some(c) if c.is_ascii_graphic() && !modifiers.shift => Key::Char(c),
            _ => us,
        },
        named => named,
    };
    Some(Keystroke { modifiers, key })
}

/// `pressesBegan:withEvent:` of a responder of the surface: whether the
/// table consumed `presses` (then the responder neither handles them nor
/// passes them on). `editing`: a text input or hosted view has the focus.
pub(crate) fn began(
    target: &EventTarget,
    presses: &AnyObject,
    editing: bool,
    priority: bool,
) -> bool {
    let state = target.keymap();
    if !state.active() {
        return false;
    }
    let mut consumed = false;
    for key in keys(presses) {
        let code = code(&key);
        // UIKit sends a key's press once: one already held lost its
        // release to a responder elsewhere (the focus moved), so it is
        // pressed anew.
        state.held.borrow_mut().retain(|(held, _)| *held != code);
        let Some(stroke) = keystroke(&key) else {
            continue;
        };
        if priority && command_input(stroke.key).is_some() {
            continue;
        }
        let Some(table) = state.table.borrow().clone() else {
            continue;
        };
        // The dev host's Cmd+R reloads, whatever the app binds.
        if cfg!(feature = "dev") && stroke.reloads(nsh_core::platform::Os::Ios) {
            continue;
        }
        // Presses come once: no auto-repeat to match (`KeyMatch::Held`).
        let KeyMatch::Fire(id) = table.find(stroke, editing, false) else {
            continue;
        };
        fire(target, id);
        let repeats = table.bindings().iter().any(|b| b.id == id && b.repeat);
        state.held.borrow_mut().push((code, repeats.then_some(id)));
        if repeats {
            let generation = state.generation.get() + 1;
            state.generation.set(generation);
            schedule(target, code, generation, REPEAT_DELAY);
        }
        consumed = true;
    }
    consumed
}

/// `pressesEnded:` / `pressesCancelled:`: whether `presses` are releases
/// of keys the table consumed (then they are consumed too).
pub(crate) fn ended(target: &EventTarget, presses: &AnyObject) -> bool {
    let state = target.keymap();
    if state.held.borrow().is_empty() {
        return false;
    }
    let mut consumed = false;
    for key in keys(presses) {
        let code = code(&key);
        let mut held = state.held.borrow_mut();
        if let Some(i) = held.iter().position(|(h, _)| *h == code) {
            held.remove(i);
            state.generation.set(state.generation.get() + 1);
            consumed = true;
        }
    }
    consumed
}

/// Reports binding `id` to JS.
fn fire(target: &EventTarget, id: u32) {
    target.dispatch(
        ROOT_ID,
        EventKind::KeyCommand,
        EventPayload::Value(PayloadValue::Number(f64::from(id))),
    );
}

/// Fires the repeating binding of held key `code` again after `delay`,
/// and on, while it is held and nothing else was pressed.
fn schedule(target: &EventTarget, code: isize, generation: u64, delay: Duration) {
    let mtm = MainThreadMarker::from(target);
    let weak = dispatch2::MainThreadBound::new(objc2::rc::Weak::from(target), mtm);
    let when = dispatch2::DispatchTime::NOW.time(delay.as_nanos() as i64);
    dispatch2::DispatchQueue::main()
        .after(when, move || {
            let mtm = MainThreadMarker::new().expect("main queue");
            let Some(target) = weak.into_inner(mtm).load() else {
                return;
            };
            let state = target.keymap();
            if state.generation.get() != generation {
                return;
            }
            let binding = state
                .held
                .borrow()
                .iter()
                .find(|(held, _)| *held == code)
                .and_then(|(_, binding)| *binding);
            // The binding may have gone with its component.
            let bound = state
                .table
                .borrow()
                .as_ref()
                .is_some_and(|t| t.bindings().iter().any(|b| Some(b.id) == binding));
            if let (Some(id), true) = (binding, bound) {
                fire(&target, id);
                schedule(&target, code, generation, REPEAT_INTERVAL);
            }
        })
        .ok();
}

/// The event target of text input `view` (its delegate).
fn input_target(view: &UIView) -> Option<Retained<EventTarget>> {
    // SAFETY: both input classes have a `delegate`, set to the target.
    let delegate: Option<Retained<AnyObject>> = unsafe { msg_send![view, delegate] };
    delegate?.downcast::<EventTarget>().ok()
}

/// `pressesBegan:withEvent:` of text input `view`: whether the table
/// consumed `presses`. While an input method composes, its keys are its
/// own (choosing a candidate, committing).
pub(crate) fn input_began(view: &UIView, presses: &AnyObject) -> bool {
    if crate::input::marked_text(view).is_some() {
        return false;
    }
    input_target(view).is_some_and(|target| began(&target, presses, true, false))
}

/// `pressesEnded:` / `pressesCancelled:` of text input `view`.
pub(crate) fn input_ended(view: &UIView, presses: &AnyObject) -> bool {
    input_target(view).is_some_and(|target| ended(&target, presses))
}

/// `pressesBegan:withEvent:` of the surface's root view, which gets the
/// presses no responder inside it handled: whether the table consumed
/// them. The app's text inputs matched theirs already; a press from
/// another view taking text (a web view's, a native component's) is in
/// text.
pub(crate) fn root_began(target: &EventTarget, presses: &AnyObject, priority: bool) -> bool {
    use objc2::ProtocolType;
    // SAFETY: as in `keys`; `responder` is UIPress's.
    let responder: Option<Retained<AnyObject>> = unsafe {
        let all: Retained<NSArray<AnyObject>> = msg_send![presses, allObjects];
        all.iter()
            .next()
            .and_then(|press| msg_send![&*press, responder])
    };
    let editing = match responder.as_deref() {
        Some(r)
            if r.downcast_ref::<crate::input::Field>().is_some()
                || r.downcast_ref::<crate::input::Area>().is_some() =>
        {
            return false;
        }
        Some(r) => {
            let text = <dyn objc2_ui_kit::UIKeyInput>::protocol()
                .is_some_and(|p| r.class().conforms_to(p));
            // The app's own texts take keys only to select (`TextView`).
            text && r.downcast_ref::<crate::native_view::NshView>().is_none()
                && r.downcast_ref::<crate::selectable_view::AreaResponder>()
                    .is_none()
        }
        None => false,
    };
    let priority = priority
        || responder
            .as_deref()
            .and_then(|r| r.downcast_ref::<UIView>())
            .is_some_and(is_fallback);
    began(target, presses, editing, priority)
}

/// UIKit's focus navigation keys need priority commands on iPadOS.
fn command_input(key: Key) -> Option<Retained<NSString>> {
    use objc2_ui_kit::*;
    // SAFETY: UIKit's exported immutable NSString constants.
    Some(unsafe {
        match key {
            Key::Up => UIKeyInputUpArrow.retain(),
            Key::Down => UIKeyInputDownArrow.retain(),
            Key::Left => UIKeyInputLeftArrow.retain(),
            Key::Right => UIKeyInputRightArrow.retain(),
            Key::Escape => UIKeyInputEscape.retain(),
            Key::Home => UIKeyInputHome.retain(),
            Key::End => UIKeyInputEnd.retain(),
            Key::PageUp => UIKeyInputPageUp.retain(),
            Key::PageDown => UIKeyInputPageDown.retain(),
            Key::Tab => NSString::from_str("\t"),
            _ => return None,
        }
    })
}

fn command_flags(modifiers: Modifiers) -> isize {
    (if modifiers.shift { SHIFT } else { 0 })
        | (if modifiers.ctrl { CONTROL } else { 0 })
        | (if modifiers.alt { ALTERNATE } else { 0 })
        | (if modifiers.cmd { COMMAND } else { 0 })
}

pub(crate) fn commands(
    view: &UIView,
    target: &EventTarget,
    modal: bool,
) -> Option<Retained<NSArray<AnyObject>>> {
    if !view.isFirstResponder() {
        return None;
    }
    let table = target.keymap().table.borrow().clone()?;
    let mut strokes = Vec::new();
    let mut commands = Vec::new();
    for binding in table.bindings() {
        let stroke = binding.keystroke;
        if strokes.contains(&stroke)
            || (modal && stroke.key == Key::Escape && stroke.modifiers == Modifiers::default())
        {
            continue;
        }
        let Some(input) = command_input(stroke.key) else {
            continue;
        };
        strokes.push(stroke);
        // SAFETY: same UIKeyCommand factory and priority setter as input.rs.
        let command: Retained<AnyObject> = unsafe {
            let command: Retained<AnyObject> = msg_send![
                AnyClass::get(c"UIKeyCommand").expect("UIKeyCommand"),
                keyCommandWithInput: &*input,
                modifierFlags: command_flags(stroke.modifiers),
                action: sel!(nshKeyCommand:)
            ];
            let _: () = msg_send![&command, setWantsPriorityOverSystemBehavior: true];
            command
        };
        commands.push(command);
    }
    Some(NSArray::from_retained_slice(&commands))
}

pub(crate) fn command(view: &UIView, target: &EventTarget, sender: &AnyObject) {
    if debug_enabled() {
        debug_log(&format!(
            "command reached who={} isFirstResponder={} {}",
            view.class().name().to_string_lossy(),
            view.isFirstResponder(),
            debug_context(view)
        ));
    }
    if !view.isFirstResponder() {
        return;
    }
    // SAFETY: the action's sender is a UIKeyCommand.
    let (input, flags): (Option<Retained<NSString>>, isize) =
        unsafe { (msg_send![sender, input], msg_send![sender, modifierFlags]) };
    let Some(input) = input else {
        return;
    };
    let Some(table) = target.keymap().table.borrow().clone() else {
        return;
    };
    let stroke = table.bindings().iter().map(|b| b.keystroke).find(|s| {
        command_flags(s.modifiers) == flags && command_input(s.key).is_some_and(|s| s == input)
    });
    if let Some(stroke) = stroke
        && let KeyMatch::Fire(id) = table.find(stroke, false, false)
    {
        // Commands have no paired release. Never start a held-key timer here.
        fire(target, id);
    }
}

/// A navigation controller's content root, outside the surface's responder chain.
pub(crate) struct CommandViewIvars {
    target: Retained<EventTarget>,
    window: RefCell<Option<Weak<UIWindow>>>,
    modal: bool,
}

define_class!(
    #[unsafe(super(UIView, UIResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = CommandViewIvars]
    pub(crate) struct CommandView;

    impl CommandView {
        #[unsafe(method_id(keyCommands))]
        fn key_commands(&self) -> Option<Retained<NSArray<AnyObject>>> {
            let result = commands(self, &self.ivars().target, self.ivars().modal);
            debug_commands(self, result.as_deref());
            result
        }

        #[unsafe(method(nshKeyCommand:))]
        fn key_command(&self, sender: &AnyObject) {
            command(self, &self.ivars().target, sender);
        }

        #[unsafe(method(canBecomeFirstResponder))]
        fn can_become_first_responder(&self) -> bool { true }

        #[unsafe(method(didMoveToWindow))]
        fn did_move_to_window(&self) {
            // SAFETY: calling the superclass implementation.
            let _: () = unsafe { msg_send![super(self), didMoveToWindow] };
            let window = self.window();
            let old = self.ivars().window.replace(window.as_ref().map(Weak::from_retained));
            if let Some(window) = window.or_else(|| old.and_then(|w| w.load())) {
                restore(&window);
            }
        }

        #[unsafe(method(pressesBegan:withEvent:))]
        fn presses_began(&self, presses: &AnyObject, event: Option<&AnyObject>) {
            // ModalController's UIKeyCommand owns unmodified Escape.
            let escape = self.ivars().modal && keys(presses).iter().any(|key| {
                keystroke(key).is_some_and(|s| s.key == Key::Escape && s.modifiers == Modifiers::default())
            });
            let consumed = !escape && root_began(&self.ivars().target, presses, self.isFirstResponder());
            debug_presses(self, presses, consumed);
            if !consumed {
                // SAFETY: calling the superclass implementation.
                let _: () = unsafe { msg_send![super(self), pressesBegan: presses, withEvent: event] };
            }
        }

        #[unsafe(method(pressesEnded:withEvent:))]
        fn presses_ended(&self, presses: &AnyObject, event: Option<&AnyObject>) {
            if !ended(&self.ivars().target, presses) {
                // SAFETY: calling the superclass implementation.
                let _: () = unsafe { msg_send![super(self), pressesEnded: presses, withEvent: event] };
            }
        }

        #[unsafe(method(pressesCancelled:withEvent:))]
        fn presses_cancelled(&self, presses: &AnyObject, event: Option<&AnyObject>) {
            if !ended(&self.ivars().target, presses) {
                // SAFETY: calling the superclass implementation.
                let _: () = unsafe { msg_send![super(self), pressesCancelled: presses, withEvent: event] };
            }
        }
    }
);

impl CommandView {
    pub(crate) fn new(target: &EventTarget, modal: bool) -> Retained<Self> {
        let this = Self::alloc(MainThreadMarker::from(target)).set_ivars(CommandViewIvars {
            target: target.retain(),
            window: RefCell::new(None),
            modal,
        });
        // SAFETY: UIView's designated initializer.
        unsafe { msg_send![super(this), initWithFrame: objc2_core_foundation::CGRect::ZERO] }
    }
}

/// Restore only an empty keyboard focus, after UIKit finishes transferring it.
/// A fallback responder takes no text and does not request accessibility focus.
pub(crate) fn restore(window: &UIWindow) {
    if debug_enabled() {
        debug_log(&format!("restore scheduled {}", debug_context(window)));
    }
    restore_after(window, Duration::ZERO);
    // Editing-end can precede UIKit releasing the hardware keyboard focus.
    restore_after(window, Duration::from_millis(100));
}

fn restore_after(window: &UIWindow, delay: Duration) {
    let window =
        dispatch2::MainThreadBound::new(Weak::from(window), MainThreadMarker::from(window));
    let when = dispatch2::DispatchTime::NOW.time(delay.as_nanos() as i64);
    dispatch2::DispatchQueue::main()
        .after(when, move || {
            let mtm = MainThreadMarker::new().expect("main queue");
            let Some(window) = window.into_inner(mtm).load() else {
                if debug_enabled() { debug_log("restore skipped window=gone occupied=unknown candidate=none becomeFirstResponder=not-called"); }
                return;
            };
            fn occupied(view: &UIView) -> Option<Retained<UIResponder>> {
                if view.isFirstResponder() && !is_fallback(view) {
                    return Some(view.retain().into_super());
                }
                if let Some(r) = view.nextResponder()
                    && r.isFirstResponder()
                    && r.downcast_ref::<UIView>().is_none_or(|v| !is_fallback(v))
                {
                    return Some(r);
                }
                view.subviews().iter().find_map(|v| occupied(&v))
            }
            fn candidate(view: &UIView) -> Option<Retained<UIView>> {
                if view.window().is_none() || view.isHidden() || view.alpha() == 0.0 {
                    return None;
                }
                view.subviews()
                    .to_vec()
                    .iter()
                    .rev()
                    .find_map(|v| candidate(v))
                    .or_else(|| is_fallback(view).then(|| view.retain()))
            }
            let Some(mut controller) = window.rootViewController() else {
                if debug_enabled() {
                    let owner = occupied(&window);
                    debug_log(&format!("restore skipped controller=nil occupied={} occupiedClass={} candidate=none becomeFirstResponder=not-called {}",
                        owner.is_some(), debug_class(owner.as_deref().map(|r| r.as_ref())), debug_context(&window)));
                }
                return;
            };
            while let Some(presented) = controller.presentedViewController() {
                controller = presented;
            }
            if delay.is_zero()
                && let Some(coordinator) = controller.transitionCoordinator()
            {
                let weak = Weak::from_retained(&window);
                let block = block2::RcBlock::new(
                    move |_context: std::ptr::NonNull<
                        objc2::runtime::ProtocolObject<
                            dyn objc2_ui_kit::UIViewControllerTransitionCoordinatorContext,
                        >,
                    >| {
                        if let Some(window) = weak.load() {
                            restore_after(&window, Duration::from_millis(1));
                        }
                    },
                );
                coordinator.animateAlongsideTransition_completion(None, Some(&block));
            }
            let owner = occupied(&window);
            let candidate = if owner.is_none() && !controller.isBeingDismissed() {
                controller.view().and_then(|v| candidate(&v))
            } else { None };
            let became = candidate.as_ref().and_then(|view| {
                (!view.isFirstResponder()).then(|| view.becomeFirstResponder())
            });
            if debug_enabled() {
                debug_log(&format!("restore delay_ms={} occupied={} occupiedClass={} dismissing={} candidate={} becomeFirstResponder={:?} {}",
                    delay.as_millis(), owner.is_some(), debug_class(owner.as_deref().map(|r| r.as_ref())),
                    controller.isBeingDismissed(), debug_class(candidate.as_deref().map(|v| v.as_ref())), became, debug_context(&window)));
            }
            if became == Some(false) {
                log::debug!("keyboard fallback could not become first responder: {:?}", candidate);
            }
        })
        .ok();
}

/// A responder that takes keys only while no control needs the keyboard.
pub(crate) fn is_fallback(view: &UIView) -> bool {
    view.downcast_ref::<CommandView>().is_some()
        || view.downcast_ref::<crate::surface::RootView>().is_some()
}
