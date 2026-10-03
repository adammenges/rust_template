//! Main-thread AppKit owner; callbacks only coalesce intents, never borrow GPUI.
//! Target/action structure adapted from HEX (MIT), see licenses/HEX-MIT.txt.
use crate::identity;
use anyhow::{Context, Result};
use objc2::{
    DefinedClass, MainThreadOnly, msg_send,
    rc::Retained,
    runtime::{AnyObject, NSObjectProtocol, Sel},
    sel,
};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSImage, NSMenu, NSMenuItem, NSStatusBar,
    NSStatusItem,
};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectNSDelayedPerforming, NSString};
use std::{cell::Cell, rc::Rc};

struct TargetIvars {
    intents: Rc<Cell<u8>>,
}
objc2::define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = TargetIvars]
    struct StatusTarget;
    unsafe impl NSObjectProtocol for StatusTarget {}
    impl StatusTarget {
        #[unsafe(method(openApp:))]
        fn open_app(&self, _: &AnyObject) { self.ivars().intents.set(self.ivars().intents.get() | super::OPEN); }
        #[unsafe(method(quitApp:))]
        fn quit_app(&self, _: &AnyObject) { self.ivars().intents.set(self.ivars().intents.get() | super::QUIT); }
    }
);
pub struct StatusItem {
    item: Retained<NSStatusItem>,
    _menu: Retained<NSMenu>,
    _target: Retained<StatusTarget>,
    intents: Rc<Cell<u8>>,
}
impl StatusItem {
    pub fn install() -> Result<Self> {
        let mtm = MainThreadMarker::new().context("Menu bar requires the main thread")?;
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
        let intents = Rc::new(Cell::new(0));
        let target = StatusTarget::alloc(mtm).set_ivars(TargetIvars {
            intents: intents.clone(),
        });
        // SAFETY: NSObject initialization and target/action selectors are declared above;
        // all retained objects stay alive in this main-thread owner until the item is removed.
        let target: Retained<StatusTarget> = unsafe { msg_send![super(target), init] };
        let menu = NSMenu::initWithTitle(
            NSMenu::alloc(mtm),
            &NSString::from_str(identity::DISPLAY_NAME),
        );
        add(
            &menu,
            &target,
            &format!("Open {}", identity::DISPLAY_NAME),
            sel!(openApp:),
            mtm,
        );
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        add(
            &menu,
            &target,
            &format!("Quit {}", identity::DISPLAY_NAME),
            sel!(quitApp:),
            mtm,
        );
        let item = NSStatusBar::systemStatusBar().statusItemWithLength(-2.0);
        let button = item.button(mtm).context("Menu-bar button unavailable")?;
        if let Some(image) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str("square.stack"),
            Some(&NSString::from_str(identity::DISPLAY_NAME)),
        ) {
            image.setTemplate(true);
            button.setImage(Some(&image));
        } else {
            button.setTitle(&NSString::from_str(identity::DISPLAY_NAME));
        }
        item.setMenu(Some(&menu));
        Ok(Self {
            item,
            _menu: menu,
            _target: target,
            intents,
        })
    }
    pub fn take_actions(&self) -> u8 {
        self.intents.replace(0)
    }
}
impl Drop for StatusItem {
    fn drop(&mut self) {
        NSStatusBar::systemStatusBar().removeStatusItem(&self.item);
    }
}
fn add(menu: &NSMenu, target: &StatusTarget, title: &str, selector: Sel, mtm: MainThreadMarker) {
    // SAFETY: selectors match the StatusTarget methods and target lifetime is retained by owner.
    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &NSString::from_str(title),
            Some(selector),
            &NSString::new(),
        )
    };
    unsafe {
        item.setTarget(Some(target));
    }
    menu.addItem(&item);
}

// GPUI's AppKit delegate does not implement applicationShouldTerminate:. Install
// only that missing method; preserve GPUI's delegate, ivars, and all other methods.
// AppKit termination otherwise exits the process without returning from run().
thread_local! {
    static TERMINATION: std::cell::RefCell<Option<TerminationState>> = const { std::cell::RefCell::new(None) };
}
struct TerminationState {
    requested: Rc<Cell<bool>>,
    ready: Rc<Cell<bool>>,
    pending: Rc<Cell<bool>>,
}
pub struct NativeApplication {
    requested: Rc<Cell<bool>>,
    ready: Rc<Cell<bool>>,
    pending: Rc<Cell<bool>>,
}
unsafe extern "C" {
    fn class_addMethod(
        class: *const objc2::runtime::AnyClass,
        selector: Sel,
        implementation: unsafe extern "C" fn(),
        types: *const std::ffi::c_char,
    ) -> bool;
}
extern "C" fn should_terminate(_: &AnyObject, _: Sel, _: &NSApplication) -> usize {
    TERMINATION.with(|state| {
        let state = state.borrow();
        match state.as_ref() {
            Some(state) if !state.ready.get() => {
                tracing::info!(event = "native_quit_deferred");
                state.requested.set(true);
                state.pending.set(true);
                2
            } // NSTerminateLater
            _ => 1, // NSTerminateNow
        }
    })
}
impl NativeApplication {
    pub fn install() -> Result<Self> {
        let mtm = MainThreadMarker::new().context("Native application requires main thread")?;
        let app = NSApplication::sharedApplication(mtm);
        let delegate = app
            .delegate()
            .context("GPUI application delegate unavailable")?;
        let delegate_object: &AnyObject = (*delegate).as_ref();
        let requested = Rc::new(Cell::new(false));
        let ready = Rc::new(Cell::new(false));
        let pending = Rc::new(Cell::new(false));
        // SAFETY: 64-bit AppKit uses NSUInteger for NSApplicationTerminateReply.
        // The IMP has Objective-C's self/_cmd/sender ABI and encoding Q@:@.
        // We add a missing method, never replace GPUI's implementation. It only
        // accesses main-thread cells; it performs no UI borrows or blocking work.
        let added = unsafe {
            let imp = std::mem::transmute::<
                extern "C" fn(&AnyObject, Sel, &NSApplication) -> usize,
                unsafe extern "C" fn(),
            >(should_terminate);
            class_addMethod(
                delegate_object.class(),
                sel!(applicationShouldTerminate:),
                imp,
                c"Q@:@".as_ptr(),
            )
        };
        anyhow::ensure!(
            added,
            "GPUI delegate already owns applicationShouldTerminate; review the quit adapter for this framework version"
        );
        TERMINATION.with(|state| {
            *state.borrow_mut() = Some(TerminationState {
                requested: requested.clone(),
                ready: ready.clone(),
                pending: pending.clone(),
            })
        });
        // Refresh AppKit's optional-delegate-method cache without replacing its owner.
        app.setDelegate(None);
        app.setDelegate(Some(&delegate));
        Ok(Self {
            requested,
            ready,
            pending,
        })
    }
    pub fn request_termination(&self) {
        // Schedule through the Cocoa run loop, not libdispatch's serial main
        // queue. AppKit's NSTerminateLater nested loop must not hold that queue
        // while GPUI's foreground poller needs it to finish asynchronous drain.
        if let Some(mtm) = MainThreadMarker::new() {
            let app = NSApplication::sharedApplication(mtm);
            // SAFETY: terminate: is an NSApplication selector accepting an optional sender.
            unsafe {
                app.performSelector_withObject_afterDelay(sel!(terminate:), None, 0.0);
            }
        }
    }
    pub fn take_quit_request(&self) -> bool {
        self.requested.replace(false)
    }
    pub fn allow_termination(&self) {
        self.ready.set(true);
    }
    /// Call outside a GPUI App borrow: replying invokes GPUI's quit callback synchronously.
    pub fn reply_if_pending(&self) {
        if self.pending.replace(false)
            && let Some(mtm) = MainThreadMarker::new()
        {
            NSApplication::sharedApplication(mtm).replyToApplicationShouldTerminate(true);
        }
    }
}
impl Drop for NativeApplication {
    fn drop(&mut self) {
        TERMINATION.with(|state| state.borrow_mut().take());
    }
}
