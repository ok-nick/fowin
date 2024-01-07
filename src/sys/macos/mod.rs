use std::ptr::{self};

use crossbeam_channel::{Receiver, Sender};
use icrate::AppKit::{NSApplicationActivationPolicyProhibited, NSWorkspace};

use crate::protocol::{WindowError, WindowEvent};

use self::ffi::{
    kAXTrustedCheckOptionPrompt, kCFAllocatorDefault, kCFBooleanTrue, AXIsProcessTrusted,
    AXIsProcessTrustedWithOptions, CFDictionaryCreate, CFRelease,
    NSRunningApplication_processIdentifier,
};
pub use self::{application::Application, window::Window};

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
    sender: Sender<WindowEvent>,
    receiver: Receiver<WindowEvent>,
    // users only know about windows, apps aren't exposed to them
    apps: Vec<Application>,
}

impl Watcher {
    pub(self) fn new(
        sender: Sender<WindowEvent>,
        receiver: Receiver<WindowEvent>,
        apps: Vec<Application>,
    ) -> Watcher {
        Watcher {
            sender,
            receiver,
            apps,
        }
    }

    // since app.windows() takes a ref to the app, need to extend the lifetime to self
    pub fn iter_windows(&self) -> impl Iterator<Item = Result<Window, WindowError>> + '_ {
        // an absolute monster of an iterator
        self.apps.iter().flat_map(|app| {
            app.windows()
                .into_iter()
                .flat_map(|windows| windows.into_iter())
        })
    }

    pub fn next_request(&self) -> WindowEvent {
        // It can only error if the sender is disconnected, but that's not possible because we
        // always have a reference to the sender within this struct.
        self.receiver.recv().unwrap()
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
    iter_apps().flat_map(|app| app.windows()).flatten()
}

pub fn watch() -> Result<Watcher, WindowError> {
    let (sender, receiver) = crossbeam_channel::unbounded();
    let apps = iter_apps()
        // TODO: if it fails should we stop the whole thing or somehow notify upstream that this app failed to be watched?
        .map_while(|app| app.watch(sender.clone()).ok().map(|_| app))
        .collect();

    // TODO: add KV observing on NSWorkspace.runningApplications to find when an app is opened/closed and add/remove it from self.apps

    Ok(Watcher::new(sender, receiver, apps))
}

fn iter_apps() -> impl Iterator<Item = Application> {
    unsafe { NSWorkspace::sharedWorkspace().runningApplications() }
        .into_iter()
        .filter(|app| unsafe { app.activationPolicy() } != NSApplicationActivationPolicyProhibited)
        .map_while(|app| {
            let pid = unsafe { NSRunningApplication_processIdentifier(&app) };
            // if it's -1 then it isn't associated with a process
            if pid != -1 {
                Some(Application::new(pid))
            } else {
                None
            }
        })
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
