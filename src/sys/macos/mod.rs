use std::{
    borrow::Borrow,
    collections::{HashMap, VecDeque},
    io,
    iter::{self, Once},
    ptr,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Instant,
};

use libc::pid_t;
use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::{NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace};
use objc2_application_services::{
    kAXTrustedCheckOptionPrompt, AXError, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
    AXUIElement,
};
use objc2_core_foundation::{kCFBooleanTrue, kCFRunLoopDefaultMode, CFDictionary, CFRunLoop};

use crate::{
    protocol::{WindowError, WindowEvent},
    sys::platform::ffi::CFRetainedSafe,
};

pub use self::{application::Application, window::Window};
use self::{
    application::{ExistingWindowsBehavior, WindowIterator},
    workspace::{AppEvent, AppEventKind, ExistingAppsBehavior, WorkspaceWatcher},
};

mod application;
mod ffi;
mod window;
mod workspace;

const TIMEOUT_STEPS: u32 = 10;

pub type WindowHandle = CFRetainedSafe<AXUIElement>;

// TODO: various properties of windows
// https://github.com/nikitabobko/AeroSpace/blob/0569bb0d663ebf732c2ea12cc168d4ff60378394/src/util/accessibility.swift#L24
// interesting: https://github.com/nikitabobko/AeroSpace/blob/0569bb0d663ebf732c2ea12cc168d4ff60378394/src/util/accessibility.swift#L296

// TODO: thread info: https://github.com/koekeishiya/yabai/issues/1583#issuecomment-1578557111

// TODO: use AXUIElementCopyAttributeNames to get a list of supported attributes for the window
// and can use AXUIElementCopyAttributeValues to get multiple values at once

// NOTE: info about api
// https://github.com/gshen7/macOSNotes

// NOTE: useful info about window ids:
// https://stackoverflow.com/questions/7422666/uniquely-identify-active-window-on-os-x
// https://stackoverflow.com/questions/311956/getting-a-unique-id-for-a-window-of-another-application/312099#312099

#[allow(dead_code)]
#[derive(Debug)]
pub enum WatcherState {
    Registering(pid_t),
    Registered(application::AppWatcher),
}

/// A system event from an app or window.
///
/// All event types are stored in the same enum so that they can be joined into the same queue and
/// somewhat properly ordered.
#[derive(Debug)]
pub enum Event {
    Window {
        pid: pid_t,
        event: Result<WindowEvent, WindowError>,
    },
    App(AppEvent),
}

#[derive(Debug)]
pub struct Watcher {
    workspace_watcher: WorkspaceWatcher,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    app_watchers: HashMap<pid_t, WatcherState>,
    // When an [`AppEventKind::Terminated`] event is received, we need to emit [`WindowEvent::Closed`]
    // events for every open window in that app. To maintain order, we use this pending queue to frontload
    // events in the channel.
    pending: VecDeque<Result<WindowEvent, WindowError>>,
}

impl Watcher {
    pub fn new() -> Result<Watcher, WindowError> {
        let (sender, receiver) = mpsc::channel();
        let workspace_watcher =
            WorkspaceWatcher::new(sender.clone(), ExistingAppsBehavior::TriggerExisting);

        Ok(Watcher {
            workspace_watcher,
            sender,
            receiver,
            app_watchers: HashMap::new(),
            pending: VecDeque::new(),
        })
    }

    /// Blocks until the next window event, pumping the main thread's run loop as needed.
    ///
    /// Must be called on the main thread because detecting new app launches/terminations relies
    /// on `NSWorkspace`, which only delivers updates while the process's actual main thread has
    /// an active run loop.
    ///
    /// We are at the mercy of macOS in terms of event ordering, so we can only report events in the
    /// order we observe them. Proper ordering is not and can not be guaranteed.
    pub fn next_request(&mut self) -> Result<WindowEvent, WindowError> {
        assert!(
            MainThreadMarker::new().is_some(),
            "`next_request` must be called on the main thread"
        );

        loop {
            // Some things to know:
            // * CFRunLoopInMode caches events internally when they happen. Calling the function will execute the callback for one event.
            // * We drain before pumping to handle outstanding events (e.g. failing to register, new app added, etc.), as well as
            //   events we queue ourselves (e.g. closing the windows of a terminated app), which have no run loop source to wake us.
            if let Some(event) = self.try_next_request_no_pump() {
                return event;
            }

            unsafe {
                // Possible errors:
                // * kCFRunLoopRunFinished: Impossible to occur, there will always be the app watcher.
                // * kCFRunLoopRunStopped: Can only occur if the user calls it, but who cares about them.
                // * kCFRunLoopRunTimedOut: It would take millions of years for the interval to timeout.
                // * kCFRunLoopRunHandledSource: AKA success.
                CFRunLoop::run_in_mode(kCFRunLoopDefaultMode, f64::MAX, true);
            }
        }
    }

    /// Attempts to return a pending value on this `Watcher` without blocking.
    ///
    /// Like [`next_request`], but never pumps any run loop. Meant for callers who already run
    /// their own run loop and just want to drain this `Watcher`'s event queue as part of that.
    /// Because nothing here drives the `CFRunLoop`, catching new app launches/terminations depends
    /// on your run loop still pumping the **main thread** somewhere. If it isn't, those specific
    /// events won't arrive.
    ///
    /// Unlike [`next_request`], this isn't required to be called on the main thread specifically
    /// since the run loop isn't pumped here.
    ///
    /// [`next_request`]: Watcher::next_request
    pub fn try_next_request_no_pump(&mut self) -> Option<Result<WindowEvent, WindowError>> {
        loop {
            if let Some(event) = self.pending.pop_front() {
                return Some(event);
            }

            // It can only error w/ disconnected if the sender is disconnected, but that's not possible because we
            // always have a reference to the sender within this struct. If it errors with empty then there's simply
            // no event ready yet.
            match self.receiver.try_recv().ok()? {
                Event::Window { pid, event } => {
                    if let Ok(event) = &event {
                        if let Some(WatcherState::Registered(watcher)) =
                            self.app_watchers.get_mut(&pid)
                        {
                            watcher.handle_event(event);
                        }
                    }

                    return Some(event);
                }
                // If handling the app event emits new closed events in the pending queue, the loop will cover it.
                Event::App(event) => self.handle_app_event(event),
            }
        }
    }

    fn handle_app_event(&mut self, event: AppEvent) {
        match event.kind {
            AppEventKind::Launched | AppEventKind::Existing => {
                self.app_watchers
                    .insert(event.pid, WatcherState::Registering(event.pid));

                let source = self.workspace_watcher.context().source.clone();
                let sender = self.sender.clone();
                // Even though the accessibility API doesn't require observers to be registered to
                // the main thread's run loop, we do it anyway to stay consistent with `WorkspaceWatcher`,
                // which does require it.
                let thread_loop = CFRetainedSafe(CFRunLoop::main().unwrap());

                // We spawn a new thread because some applications can take a long time to respond to AX operations or it is taking a long
                // time for the app to initialize.
                thread::spawn(move || {
                    let app = Application::new(event.pid);

                    // Read more on why we do this in `Application::should_wait`.
                    let start = Instant::now();
                    while app.should_wait() && Instant::now().duration_since(start) <= app.timeout()
                    {
                        thread::sleep(app.timeout() / TIMEOUT_STEPS);
                    }

                    // If it passed the timeout and it's still not valid then unfortunately we are going to have to pass.
                    if app.should_wait() {
                        return;
                    }

                    let existing_windows = if matches!(event.kind, AppEventKind::Launched) {
                        // When an app is launched and windows are created, we aren't quick enough to receive
                        // an event for them, so we trigger `WindowEvent::Opened` for all existing windows.
                        ExistingWindowsBehavior::TriggerExisting
                    } else {
                        ExistingWindowsBehavior::Skip
                    };
                    let app_event_kind = match app.watch(sender.clone(), existing_windows) {
                        Ok(watcher) => {
                            watcher.run_on_thread(&thread_loop);
                            AppEventKind::Registered(watcher)
                        }
                        Err(err) => AppEventKind::FailedToRegister(err),
                    };

                    let _ = sender.send(Event::App(AppEvent {
                        kind: app_event_kind,
                        pid: app.pid(),
                    }));

                    source.signal();
                    thread_loop.wake_up();
                });
            }
            AppEventKind::Terminated => {
                if let Some(WatcherState::Registered(watcher)) =
                    self.app_watchers.remove(&event.pid)
                {
                    // Trigger [`WindowEvent::Closed`] for any open windows we know about since macOS doesn't do that for us.
                    self.pending
                        .extend(watcher.into_closed_events().into_iter().map(Ok));
                }
            }
            AppEventKind::Registered(watcher) => {
                // If it already exists in the hash map, then it MUST be WatcherState::Registering, which is the only acceptable case.
                // If it doesn't exist in the hash map, then it must've been terminated already.
                // It can't be WatcherState::Registered because it's not possible for the launch notification to be sent twice.
                // TODO: verify the latter case or implement check
                match self.app_watchers.get_mut(&event.pid) {
                    Some(entry) => {
                        *entry = WatcherState::Registered(watcher);
                    }
                    None => self
                        .pending
                        .extend(watcher.into_closed_events().into_iter().map(Ok)),
                }
            }
            AppEventKind::FailedToRegister(err) => self.pending.push_back(Err(err)),
        }
    }
}

pub fn trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

pub fn request_trust() -> Result<bool, WindowError> {
    let options = unsafe {
        CFDictionary::new(
            None,
            [kAXTrustedCheckOptionPrompt as *const _ as *const _].as_mut_ptr(),
            [kCFBooleanTrue.unwrap() as *const _ as *const _].as_mut_ptr(),
            1,
            ptr::null(),
            ptr::null(),
        )
    };
    match options {
        Some(options) => Ok(unsafe { AXIsProcessTrustedWithOptions(Some(&options)) }),
        None => Err(WindowError::OsError(io::Error::other(
            "failed to create `CFDictionary` for accessibility trust options",
        ))),
    }
}

pub fn focused_window() -> Result<Option<Window>, WindowError> {
    match NSWorkspace::sharedWorkspace().frontmostApplication() {
        Some(app) => {
            let focused_pid = app.processIdentifier();
            for window in Application::new(focused_pid).iter_windows()? {
                let window = window?;
                if window.is_focused()? {
                    return Ok(Some(window));
                }
            }

            Ok(None)
        }
        None => Ok(None),
    }
}

pub fn iter_windows() -> impl Iterator<Item = Result<Window, WindowError>> {
    iter_windows_with_app_iter(iter_apps())
}

#[inline]
fn iter_windows_with_app_iter(
    app_iter: impl Iterator<Item = impl Borrow<Application>>,
) -> impl Iterator<Item = Result<Window, WindowError>> {
    app_iter.flat_map(|app| {
        app.borrow()
            .iter_windows()
            .map(WindowIteratorOrErr::WindowIterator)
            .unwrap_or_else(|err| WindowIteratorOrErr::Err(iter::once(Err(err))))
    })
}

fn iter_apps() -> impl Iterator<Item = Application> {
    filter_apps(
        NSWorkspace::sharedWorkspace()
            .runningApplications()
            .into_iter(),
    )
    .map(|app| Application::new(app.processIdentifier()))
}

fn filter_apps(
    apps: impl Iterator<Item = Retained<NSRunningApplication>>,
) -> impl Iterator<Item = Retained<NSRunningApplication>> {
    apps
        // TODO: need to do more filtering, check out yabai, they have pretty extensive filtering
        // https://github.com/koekeishiya/yabai/issues/439
        // https://github.com/koekeishiya/yabai/blob/60380a1f18ebaa503fda29a72647fd8f5f5ce43b/src/process_manager.c#L14-L61
        // https://github.com/koekeishiya/yabai/commit/82727a2c22a9ed82e51223e554de39636e21061f#
        //
        // NOTE: ideally we'd include ::Accessory activation policy apps, but most (if not all) of them are irrelevant
        //       and cause significant slow downs
        .filter(|app| app.activationPolicy() == NSApplicationActivationPolicy::Regular)
        .filter(|app| {
            // TODO: can get pid from app on main branch of objc2, waiting for release
            let pid = app.processIdentifier();
            // if it's -1 then the app isn't associated with a process
            pid != -1
        })
}

#[derive(Debug)]
enum WindowIteratorOrErr {
    WindowIterator(WindowIterator),
    Err(Once<Result<Window, WindowError>>),
}

impl Iterator for WindowIteratorOrErr {
    type Item = Result<Window, WindowError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            WindowIteratorOrErr::WindowIterator(iter) => iter.next(),
            WindowIteratorOrErr::Err(iter) => iter.next(),
        }
    }
}

impl From<AXError> for WindowError {
    // https://developer.apple.com/documentation/applicationservices/axerror?language=objc
    fn from(value: AXError) -> Self {
        match value {
            AXError::APIDisabled => WindowError::NotTrusted,
            AXError::IllegalArgument => WindowError::InvalidInternalArgument,
            AXError::InvalidUIElementObserver => WindowError::InvalidHandle,
            AXError::InvalidUIElement => WindowError::InvalidHandle,
            AXError::NotImplemented => WindowError::Unsupported,
            // attempt to retrieve unsupported attribute
            AXError::NoValue => WindowError::Unsupported,
            AXError::AttributeUnsupported => WindowError::Unsupported,
            AXError::ParameterizedAttributeUnsupported => WindowError::Unsupported,
            AXError::ActionUnsupported => WindowError::Unsupported,
            AXError::NotificationUnsupported => WindowError::Unsupported,
            err @ (
                // because this event shouldn't be possible (it's handled manually) and there is no enum variant for it, we label it as an arbitrary error
                AXError::NotificationAlreadyRegistered
                // same here
                | AXError::NotificationNotRegistered
                // no idea when this could occur, it's not documented
                | AXError::NotEnoughPrecision
                // called when the accessibility API timeout is reached
                // TODO: give this a WindowError::TimeoutReached error so the user can retry or ack?
                | AXError::CannotComplete
                | AXError::Failure | _
            ) => WindowError::OsError(io::Error::other(format!(
                "accessibility API returned {err:?}",
            ))),
        }
    }
}
