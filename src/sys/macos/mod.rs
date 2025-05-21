use std::{
    borrow::Borrow,
    collections::HashMap,
    ffi::c_void,
    iter::{self, Once},
    ptr,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, ThreadId},
    time::{Duration, Instant},
};

use objc2::{
    declare_class, msg_send, msg_send_id, mutability, rc::Id, runtime::AnyObject, ClassType,
    DeclaredClass,
};
use objc2_app_kit::{NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace};
use objc2_foundation::{
    ns_string, NSArray, NSDictionary, NSKeyValueChangeKey, NSKeyValueChangeNewKey,
    NSKeyValueChangeOldKey, NSKeyValueObservingOptions, NSObject, NSString,
};

use crate::{
    protocol::{WindowError, WindowEvent},
    sys::platform::ffi::{CFRunLoopGetCurrent, CFRunLoopSourceSignal, CFRunLoopWakeUp},
};

pub use self::{application::Application, window::Window};
use self::{
    application::WindowIterator,
    ffi::{
        kAXTrustedCheckOptionPrompt, kCFAllocatorDefault, kCFBooleanTrue, kCFRunLoopDefaultMode,
        pid_t, AXIsProcessTrusted, AXIsProcessTrustedWithOptions, AXUIElementRef,
        CFDictionaryCreate, CFRelease, CFRunLoopAddSource, CFRunLoopRunInMode,
        CFRunLoopSourceContext, CFRunLoopSourceCreate, CFRunLoopSourceRef, CGWindowID,
        NSRunningApplication_processIdentifier,
    },
};

mod application;
mod ffi;
mod window;

const TIMEOUT_STEPS: u32 = 10;

pub type WindowHandle = AXUIElementRef;

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

#[derive(Debug)]
pub enum WatcherState {
    Registering(pid_t),
    Registered(application::Watcher),
}

#[derive(Debug)]
pub struct Watcher {
    app_watcher: AppWatcher,
    sender: Sender<Result<WindowEvent, WindowError>>,
    receiver: Receiver<Result<WindowEvent, WindowError>>,
    watchers: HashMap<pid_t, WatcherState>,
    thread_id: ThreadId,
}

// The run loop must be ran on the thread the watchers are created.
impl !Send for Watcher {}
impl !Sync for Watcher {}

impl Watcher {
    pub fn new() -> Result<Watcher, WindowError> {
        // Start the app watcher so we never miss any new apps while registering existing apps.
        let app_watcher = AppWatcher::new();

        for app in iter_apps() {
            app_watcher
                .context
                .sender
                .send(AppEvent {
                    kind: AppEventKind::Launched,
                    pid: app.pid(),
                })
                // Sender + Receiver always exist.
                .unwrap();
        }

        let (sender, receiver) = mpsc::channel();
        Ok(Watcher {
            app_watcher,
            sender,
            receiver,
            watchers: HashMap::new(),
            thread_id: thread::current().id(),
        })
    }

    // TODO: same as below, but orders the output
    pub fn next_request_buffered_ordered(
        &self,
        interval: Duration,
    ) -> Result<WindowEvent, WindowError> {
        todo!()
    }

    // TODO: this function will call CFRunLoopInMode w/ interval seconds, it returns a list of events >= interval age
    pub fn next_request_buffered(&self, interval: Duration) -> Result<WindowEvent, WindowError> {
        todo!()
    }

    // TODO: This function MUST be called on the thread its watchers were created (preferably main thread for perf/responsiveness?)
    //
    // TODO: the user may run their own run loop somewhere else, this code would interfere with that
    //       it may be wise to separate the run loop logic from the receiver logic
    pub fn next_request(&mut self) -> Result<WindowEvent, WindowError> {
        assert!(
            thread::current().id() == self.thread_id,
            "can only get next request on the same thread the `Watcher` was created"
        );

        if let Some(event) = self.app_watcher.next_request() {
            self.handle_app_event(event)?;
        }

        // Some things to know:
        // * CFRunLoopInMode caches events internally when they happen. Calling the function will execute the callback for one event.
        // * The if statement below is to handle outstanding events (e.g. failing to register, new app added, etc.).
        if let Ok(event) = self.receiver.try_recv() {
            return event;
        }

        loop {
            // TODO: it is impossible to get a timestamp for when an event occurs
            //       this function should run the loop to completion each call and return a vector of events
            //       this way, the next time this function is called, you know those events are guaranteed to happen after the last
            //       vector of events. It provides some sense of ordering and the vector will only occassionally have >1 element
            unsafe {
                // Possible errors:
                // * kCFRunLoopRunFinished: Impossible to occur, there will always be the app watcher.
                // * kCFRunLoopRunStopped: Can only occur if the user calls it, but who cares about them.
                // * kCFRunLoopRunTimedOut: It would take millions of years for the interval to timeout.
                // * kCFRunLoopRunHandledSource: AKA success.
                CFRunLoopRunInMode(kCFRunLoopDefaultMode, f64::MAX, true as u8);
            }

            // Handle registering/deregistering launched/terminated apps.
            if let Some(event) = self.app_watcher.next_request() {
                self.handle_app_event(event)?;
                // Since CFRunLoopInMode only processes one event at a time, skip checking for window events.
                continue;
            }

            // It can only error w/ disconnected if the sender is disconnected, but that's not possible because we
            // always have a reference to the sender within this struct. If it errors with empty then we skip to the
            // next iteration.
            if let Ok(event) = self.receiver.try_recv() {
                return event;
            }
        }
    }

    fn handle_app_event(&mut self, event: AppEvent) -> Result<(), WindowError> {
        match event.kind {
            AppEventKind::Launched => {
                self.watchers
                    .insert(event.pid, WatcherState::Registering(event.pid));

                // We spawn a new thread because some applications can take a long time to respond to AX operations or it is taking a long
                // time for the app to initialize.
                let sender = self.sender.clone();
                let app_sender = self.app_watcher.context.sender.clone();
                let source = self.app_watcher.context.source.clone();
                let thread_loop = CFRunLoopSourceRef(unsafe { CFRunLoopGetCurrent() });
                // TODO: kinda hacky, we aren't responsible for ownership of the run loop
                thread_loop.increment_ref_count();

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

                    match app.watch(sender.clone()) {
                        Ok(watcher) => {
                            let thread_loop = thread_loop;
                            watcher.run_on_thread(thread_loop.0);

                            let _ = app_sender.send(AppEvent {
                                kind: AppEventKind::Registered(watcher),
                                pid: app.pid(),
                            });
                            unsafe {
                                let source = source;
                                CFRunLoopSourceSignal(source.0);
                                CFRunLoopWakeUp(thread_loop.0);
                            }
                        }
                        Err(err) => {
                            // TODO: in this case, return a struct dedicated to reconnecting the failed watcher that the user can handle
                            let _ = sender.send(Err(err));
                        }
                    }
                });
            }
            AppEventKind::Terminated => {
                self.watchers.remove(&event.pid);
            }
            AppEventKind::Registered(watcher) => {
                // If it already exists in the hash map, then it MUST be WatcherState::Registering, which is the only acceptable case.
                // If it doesn't exist in the hash map, then it must've been terminated already.
                // It can't be WatcherState::Registered because it's not possible for the launch notification to be sent twice.
                // TODO: verify the latter case or implement check
                if self.watchers.contains_key(&event.pid) {
                    self.watchers
                        .insert(event.pid, WatcherState::Registered(watcher));
                }
            }
        }

        Ok(())
    }
}

pub fn trusted() -> bool {
    unsafe { AXIsProcessTrusted() != 0 }
}

pub fn request_trust() -> Result<bool, WindowError> {
    let options = unsafe {
        CFDictionaryCreate(
            kCFAllocatorDefault,
            [kAXTrustedCheckOptionPrompt as *const _].as_mut_ptr(),
            [kCFBooleanTrue as *const _].as_mut_ptr(),
            1,
            ptr::null(),
            ptr::null(),
        )
    };
    match options.is_null() {
        true => Err(WindowError::ArbitraryFailure),
        false => {
            let result = unsafe { AXIsProcessTrustedWithOptions(options) != 0 };
            unsafe {
                CFRelease(options as *const _);
            }
            Ok(result)
        }
    }
}

pub fn focused_window() -> Result<Option<Window>, WindowError> {
    match unsafe { NSWorkspace::sharedWorkspace().frontmostApplication() } {
        Some(app) => {
            let focused_pid = unsafe { NSRunningApplication_processIdentifier(&app) };
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
    filter_apps(unsafe { NSWorkspace::sharedWorkspace().runningApplications() }.into_iter())
        .map(|app| Application::new(unsafe { NSRunningApplication_processIdentifier(&app) }))
}

fn filter_apps(
    apps: impl Iterator<Item = Id<NSRunningApplication>>,
) -> impl Iterator<Item = Id<NSRunningApplication>> {
    apps
        // TODO: need to do more filtering, check out yabai, they have pretty extensive filtering
        // https://github.com/koekeishiya/yabai/issues/439
        // https://github.com/koekeishiya/yabai/blob/60380a1f18ebaa503fda29a72647fd8f5f5ce43b/src/process_manager.c#L14-L61
        // https://github.com/koekeishiya/yabai/commit/82727a2c22a9ed82e51223e554de39636e21061f#
        //
        // NOTE: ideally we'd include ::Accessory activation policy apps, but most (if not all) of them are irrelevant
        //       and cause significant slow downs
        .filter(|app| unsafe { app.activationPolicy() } == NSApplicationActivationPolicy::Regular)
        .filter(|app| {
            // TODO: can get pid from app on main branch of objc2, waiting for release
            let pid = unsafe { NSRunningApplication_processIdentifier(app) };
            // if it's -1 then the app isn't associated with a process
            pid != -1
        })
}

#[derive(Debug)]
pub enum AppEventKind {
    Launched,
    Terminated,
    Registered(application::Watcher),
}

#[derive(Debug)]
pub struct AppEvent {
    kind: AppEventKind,
    pid: pid_t,
}

declare_class!(
    #[derive(Debug)]
    struct AppWatcherInner;

    unsafe impl ClassType for AppWatcherInner {
        type Super = NSObject;
        type Mutability = mutability::Immutable;
        const NAME: &'static str = "TODO_AppWatcher";
    }

    impl DeclaredClass for AppWatcherInner {}

    unsafe impl AppWatcherInner {
        #[allow(non_snake_case)]
        #[method(observeValueForKeyPath:ofObject:change:context:)]
        unsafe fn observeValueForKeyPath_ofObject_change_context(
            &self,
            key_path: *mut NSString,
            object: *mut AnyObject,
            change: *mut NSDictionary<NSKeyValueChangeKey, NSArray<NSRunningApplication>>,
            context: *mut c_void
        ) {
            let context = context as *mut Context;
            let mut sent = false;

            if let Some(new_apps)= (*change).get_retained(NSKeyValueChangeNewKey) {
                for app in filter_apps(new_apps.into_iter()) {
                    let _ = (*context).sender.send(AppEvent {
                        kind: AppEventKind::Launched,
                        pid: NSRunningApplication_processIdentifier(&app)
                    });

                    sent = true;
                }
            }

            if let Some(old_apps) = (*change).get_retained(NSKeyValueChangeOldKey) {
                for app in filter_apps(old_apps.into_iter()) {
                    let _ = (*context).sender.send(AppEvent {
                        kind: AppEventKind::Terminated,
                        pid: NSRunningApplication_processIdentifier(&app)
                    });

                    sent = true;
                }
            }

            if sent {
                CFRunLoopSourceSignal((*context).source.0);
                CFRunLoopWakeUp(CFRunLoopGetCurrent());
            }
        }
    }
);

#[derive(Debug)]
pub struct Context {
    sender: Sender<AppEvent>,
    // The reason we create a "dummy" source is because registering a KVO (AKA AppWatcherInner) does not trigger
    // a source as being "processed" thus not prompting CFRunLoopInMode to return.
    source: CFRunLoopSourceRef,
}

// TODO: kqueues also exist, but I'm not sure if it provides any advantages
#[derive(Debug)]
pub struct AppWatcher {
    inner: Id<AppWatcherInner>,
    context: Box<Context>,
    receiver: Receiver<AppEvent>,
}

unsafe extern "C" fn test(info: *mut ::std::os::raw::c_void) {
    println!("signaled");
}

impl AppWatcher {
    pub fn new() -> AppWatcher {
        let source = unsafe {
            CFRunLoopSourceRef(CFRunLoopSourceCreate(
                kCFAllocatorDefault,
                -1,
                &mut CFRunLoopSourceContext {
                    version: 0,
                    info: ptr::null_mut(),
                    retain: None,
                    release: None,
                    copyDescription: None,
                    equal: None,
                    hash: None,
                    schedule: None,
                    cancel: None,
                    perform: None,
                } as *mut _,
            ))
        };

        unsafe {
            CFRunLoopAddSource(CFRunLoopGetCurrent(), source.0, kCFRunLoopDefaultMode);
        }

        let (sender, receiver) = mpsc::channel();
        let context = Box::into_raw(Box::new(Context { sender, source }));

        let inner: Id<AppWatcherInner> = unsafe { msg_send_id![AppWatcherInner::alloc(), init] };
        unsafe {
            let _: () = msg_send![
                &NSWorkspace::sharedWorkspace(),
                addObserver: &*inner,
                forKeyPath: ns_string!("runningApplications"),
                options: NSKeyValueObservingOptions::NSKeyValueObservingOptionNew.0 | NSKeyValueObservingOptions::NSKeyValueObservingOptionOld.0,
                context: context as *const c_void
            ];
        }

        AppWatcher {
            inner,
            context: unsafe { Box::from_raw(context) },
            receiver,
        }
    }

    // Assumes the run loop is being ran.
    pub fn next_request(&self) -> Option<AppEvent> {
        // Impossible for disconnected error, only possible for empty error, in which case it should be an option.
        self.receiver.try_recv().ok()
    }
}

impl Drop for AppWatcher {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![
                &NSWorkspace::sharedWorkspace(),
                removeObserver: &*self.inner,
                forKeyPath: ns_string!("runningApplications"),
                context: &*self.context as *const _ as *const c_void
            ];
        }
    }
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

impl WindowError {
    // https://developer.apple.com/documentation/applicationservices/axerror?language=objc
    pub(self) fn from_ax_error(code: i32) -> WindowError {
        match code {
            ffi::kAXErrorAPIDisabled => WindowError::NotTrusted,
            ffi::kAXErrorIllegalArgument => WindowError::InvalidInternalArgument,
            ffi::kAXErrorInvalidUIElementObserver => WindowError::InvalidHandle,
            ffi::kAXErrorInvalidUIElement => WindowError::InvalidHandle,
            ffi::kAXErrorNotImplemented => WindowError::Unsupported,
            // attempt to retrieve unsupported attribute
            ffi::kAXErrorNoValue => WindowError::Unsupported,
            ffi::kAXErrorAttributeUnsupported => WindowError::Unsupported,
            ffi::kAXErrorParameterizedAttributeUnsupported => WindowError::Unsupported,
            ffi::kAXErrorActionUnsupported => WindowError::Unsupported,
            ffi::kAXErrorNotificationUnsupported => WindowError::Unsupported,
            // because this event shouldn't be possible (it's handled manually) and there is no enum variant for it, we label it as an arbitrary error
            ffi::kAXErrorNotificationAlreadyRegistered => WindowError::ArbitraryFailure,
            // same here
            ffi::kAXErrorNotificationNotRegistered => WindowError::ArbitraryFailure,
            // no idea when this could occur, it's not documented
            ffi::kAXErrorNotEnoughPrecision => WindowError::ArbitraryFailure,
            // called when the accessibility API timeout is reached
            ffi::kAXErrorCannotComplete => WindowError::ArbitraryFailure,
            ffi::kAXErrorFailure => WindowError::ArbitraryFailure,
            _ => WindowError::ArbitraryFailure,
        }
    }
}
