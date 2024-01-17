use std::{
    borrow::Borrow,
    iter::{self, Once},
    ptr::{self},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, ThreadId},
    time::Duration,
};

use icrate::AppKit::{NSApplicationActivationPolicyRegular, NSWorkspace};

use crate::protocol::{WindowError, WindowEvent};

pub use self::{application::Application, window::Window};
use self::{
    application::WindowIterator,
    ffi::{
        kAXTrustedCheckOptionPrompt, kCFAllocatorDefault, kCFBooleanTrue, kCFRunLoopDefaultMode,
        kCFRunLoopRunFinished, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
        CFDictionaryCreate, CFRelease, CFRunLoopRunInMode, NSRunningApplication_processIdentifier,
    },
};

mod application;
mod ffi;
mod window;

// TODO: various properties of windows
// https://github.com/nikitabobko/AeroSpace/blob/0569bb0d663ebf732c2ea12cc168d4ff60378394/src/util/accessibility.swift#L24
// interesting: https://github.com/nikitabobko/AeroSpace/blob/0569bb0d663ebf732c2ea12cc168d4ff60378394/src/util/accessibility.swift#L296

// TODO: worth looking into, AXUIElementSetMessagingTimeout

// TODO: also worth noting that these accessibility functions can take a decent amount of time to return
//       it's said they must also run on the main thread, so it may be worthwhile to run program code on background threads
// NOTE: read window.rs above struct, I believe it's able to be called from any thread as long as it's one thread at a time?

// TODO: use AXUIElementCopyAttributeNames to get a list of supported attributes for the window
// and can use AXUIElementCopyAttributeValues to get multiple values at once

// NOTE: info about api
// https://github.com/gshen7/macOSNotes

// NOTE: useful info about window ids:
// https://stackoverflow.com/questions/7422666/uniquely-identify-active-window-on-os-x
// https://stackoverflow.com/questions/311956/getting-a-unique-id-for-a-window-of-another-application/312099#312099

#[derive(Debug)]
pub struct Watcher {
    // TODO: the sender will be passed to newly launched apps
    sender: Sender<Result<WindowEvent, WindowError>>,
    receiver: Receiver<Result<WindowEvent, WindowError>>,
    // users only know about windows, apps aren't exposed to them
    // apps: Vec<Application>,
    watchers: Vec<application::Watcher>,
    // TODO: keep track of disconnected applications (aka apps that failed to be watched)
    //       and somehow interface it to the user so they can reconnect
    thread_id: ThreadId,
}

impl Watcher {
    // All watchers MUST be created on the same thread that this struct is created.
    pub(self) fn new(
        sender: Sender<Result<WindowEvent, WindowError>>,
        receiver: Receiver<Result<WindowEvent, WindowError>>,
        // apps: Vec<Application>,
        watchers: Vec<application::Watcher>,
    ) -> Watcher {
        Watcher {
            sender,
            receiver,
            // apps,
            watchers,
            thread_id: thread::current().id(),
        }
    }

    // Since app.windows() takes a ref to the app, we must extend the lifetime to self.
    pub fn iter_windows(&self) -> impl Iterator<Item = Result<Window, WindowError>> + '_ {
        //TODO: iter_windows_with_app_iter(self.apps.iter())
        iter_windows_with_app_iter(iter_apps())
    }

    pub fn reconnect(&self) {
        // TODO: this function will attempt to reconnect failed-to-connect watchers
    }

    // TODO: This function MUST be called on the thread its watchers were created (preferably main thread for perf/responsiveness?)
    //       it would be better if I can separate the sender logic from the event loop logic so that they can be used separately (and
    //       so that checking the current thread + other things isn't required every iteration)
    pub fn next_request(&self) -> Result<WindowEvent, WindowError> {
        assert!(
            thread::current().id() == self.thread_id,
            "can only get next request on the same thread the `Watcher` was created"
        );

        loop {
            let result = unsafe { CFRunLoopRunInMode(kCFRunLoopDefaultMode, f64::MAX, true as u8) };
            if result == kCFRunLoopRunFinished {
                // TODO: in this case there are no watchers watching, either because they all failed to connect (introduce reconnecting)
                //       or there are literally no windows to watch (doubt will ever be the case). Nevertheless, if this error occurs every
                //       time this function is called, CPU usage will skyrocket, so perhaps add some sort of delay?
                //
                //       essentially, we are waiting for a new application to be launched and a new internal watcher to be created. maybe
                //       we can signify with an event
            }

            // It can only error w/ disconnected if the sender is disconnected, but that's not possible because we
            // always have a reference to the sender within this struct. If it errors with empty then we skip to the
            // next iteration.
            if let Ok(event) = self.receiver.try_recv() {
                return event;
            }
        }
    }

    // TODO: this function will call CFRunLoopInMode w/ interval seconds
    pub fn next_request_buffered(&self, interval: Duration) -> Result<WindowEvent, WindowError> {
        todo!()
    }

    // TODO: same as above, but orders the output
    pub fn next_request_buffered_ordered(
        &self,
        interval: Duration,
    ) -> Result<WindowEvent, WindowError> {
        todo!()
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

pub fn iter_windows() -> impl Iterator<Item = Result<Window, WindowError>> {
    iter_windows_with_app_iter(iter_apps())
}

pub fn watch() -> Result<Watcher, WindowError> {
    let (sender, receiver) = mpsc::channel();
    let watchers = iter_apps()
        .filter_map(|app| match app.watch(sender.clone()) {
            Ok(watcher) => Some(watcher),
            Err(err) => {
                // TODO: in this case, we need to somehow give the users the option to attempt to reconnect to the app
                // Safe to unwrap, we created the sender right above, so we know it's connected and empty.
                sender.send(Err(err)).unwrap();
                None
            }
        })
        .collect();

    // TODO: add KV observing on NSWorkspace.runningApplications to find when an app is opened/closed and add/remove it from self.apps

    Ok(Watcher::new(sender, receiver, watchers))
}

#[inline]
fn iter_windows_with_app_iter(
    app_iter: impl Iterator<Item = impl Borrow<Application>>,
) -> impl Iterator<Item = Result<Window, WindowError>> {
    app_iter.flat_map(|app| {
        app.borrow()
            .windows()
            .map(WindowIteratorOrErr::WindowIterator)
            .unwrap_or_else(|err| WindowIteratorOrErr::Err(iter::once(Err(err))))
    })
}

fn iter_apps() -> impl Iterator<Item = Application> {
    unsafe { NSWorkspace::sharedWorkspace().runningApplications() }
        .into_iter()
        // TODO: I believe an NSApplicationActivationPolicyAccessory type of app can also spawn their own windows,
        //       however, many times they fail to be watched and can take a few seconds to respond. TLDR; do some research
        // .filter(|app| unsafe { app.activationPolicy() } != NSApplicationActivationPolicyProhibited)
        .filter(|app| unsafe { app.activationPolicy() } == NSApplicationActivationPolicyRegular)
        .map_while(|app| {
            let pid = unsafe { NSRunningApplication_processIdentifier(&app) };
            // if it's -1 then the app isn't associated with a process
            if pid != -1 {
                Some(Application::new(pid))
            } else {
                None
            }
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
