use std::{
    collections::HashMap,
    iter,
    ptr::{self, NonNull},
};

use crossbeam_channel::{Receiver, Sender};
use icrate::{
    block2::ConcreteBlock,
    objc2::runtime::Bool,
    AppKit::{
        NSApplicationActivationPolicyProhibited, NSRunningApplication, NSWorkspace,
        NSWorkspaceActiveSpaceDidChangeNotification, NSWorkspaceDidActivateApplicationNotification,
        NSWorkspaceDidHideApplicationNotification, NSWorkspaceDidLaunchApplicationNotification,
        NSWorkspaceDidTerminateApplicationNotification,
        NSWorkspaceDidUnhideApplicationNotification,
    },
    Foundation::{NSEnumerationReverse, NSNotification, NSNotificationCenter},
};

use crate::protocol::{Position, Size, WindowError, WindowEvent, WindowEventKind, WindowId};

use self::ffi::{
    kAXTrustedCheckOptionPrompt, kCFAllocatorDefault, kCFBooleanTrue, AXIsProcessTrusted,
    AXIsProcessTrustedWithOptions, CFDictionaryCreate, CFDictionarySetValue, CFRelease,
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

// TODO: https://github.com/koekeishiya/yabai/blob/7c02059084cbd4874ef25ce25a80b89101eb3536/src/workspace.m#L130
// https://github.com/nikitabobko/AeroSpace/blob/0569bb0d663ebf732c2ea12cc168d4ff60378394/src/GlobalObserver.swift#L6
// static GLOBAL_NOTIFICATIONS: [&NSAccessibilityNotificationName; 6] = [
//     unsafe { NSWorkspaceDidLaunchApplicationNotification },
//     unsafe { NSWorkspaceDidActivateApplicationNotification },
//     unsafe { NSWorkspaceDidHideApplicationNotification },
//     unsafe { NSWorkspaceDidUnhideApplicationNotification },
//     unsafe { NSWorkspaceActiveSpaceDidChangeNotification },
//     unsafe { NSWorkspaceDidTerminateApplicationNotification },
//     // NSWorkspaceActiveDisplayDidChangeNotification
//     // AppleInterfaceMenuBarHidingChangedNotification
// ];

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

    // TODO: wrap the error so we aren't exposing crossbeam types
    pub fn next_request(&self) -> Result<WindowEvent, crossbeam_channel::RecvError> {
        self.receiver.recv()
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
    iter_apps()
        .flat_map(|result| result.into_iter().flat_map(|app| app.windows()))
        .flatten()
}

// pub fn iter_windows() -> impl Iterator<Item = Result<Window, WindowError>> {
//     // holy flat_maps
//     iter_apps().flat_map(|result| {
//         result.into_iter().flat_map(|app| {
//             app.windows()
//                 .into_iter()
//                 .flat_map(|windows| windows.into_iter())
//         })
//     })
// }

// TODO: this isn't necessary since KVO will tell us exactly which indices change
//  fn check_new_apps(&mut self) -> Result<(), ()> {
//     let workspace = unsafe { NSWorkspace::sharedWorkspace().runningApplications() };

//     let common = self.apps.last();
//     // app, index, stop
//     let block = ConcreteBlock::new(move |app, _, mut stop: NonNull<Bool>| {
//         // TODO: if app == common {
//         // we found all the new apps, stop iterating
//         if true {
//             unsafe { *stop.as_ptr() = Bool::NO };
//         } else {

//         }
//     });
//     unsafe {
//         workspace.enumerateObjectsWithOptions_usingBlock(NSEnumerationReverse, &block);
//     }

//     Ok(())
// }

pub fn watch() -> Result<Watcher, WindowError> {
    let (sender, receiver) = crossbeam_channel::unbounded();
    let apps = iter_apps()
        .flatten()
        // TODO: if it fails should we stop the whole thing or skip?
        .map_while(|app| app.watch(sender.clone()).ok().map(|_| app))
        .collect();

    // TODO: add KV observing on NSWorkspace.runningApplications to find when an app is opened/closed and add/remove it from self.apps

    Ok(Watcher::new(sender, receiver, apps))
}

//  fn watch(&mut self, sender: Sender<WindowEventInfo>) {
//     self.add_running_apps().unwrap();

//     // TODO: also consider implementing the older Carbon API for events like kEventAppLaunched, necessary?
//     let block = ConcreteBlock::new(move |notification: NonNull<NSNotification>| {
//         // TODO: confirm safety of notification in NSNotification docs
//         let notification = unsafe { notification.as_ref() };
//         let name = unsafe { notification.name() };

//         if unsafe { &*name == NSWorkspaceDidActivateApplicationNotification } {
//             // TODO: I believe this has to do with changing focus, need to find which window is being focused
//             todo!()
//         } else if unsafe { &*name == NSWorkspaceDidHideApplicationNotification } {
//             // TODO: find windows corresponding to app and send hidden event
//             todo!()
//         } else if unsafe { &*name == NSWorkspaceDidUnhideApplicationNotification } {
//             // TODO: ^
//             todo!()
//         } else if unsafe { &*name == NSWorkspaceActiveSpaceDidChangeNotification } {
//             // TODO: this can cause windows to be open and non-hidden but also invisible since it's on a different space...
//             todo!()
//         };
//     })
//     .copy();

//     // TODO: initialize as static
//     let GLOBAL_NOTIFICATIONS = [
//         // TODO: it says to use key-value observing for NSWorkspace.runningApplications to detect both foreground and background applications instead of launch/terminating events
//         // https://github.com/koekeishiya/yabai/blob/656f8c868e5247246593950e0af2815e89313cce/src/workspace.m#L182
//         // unsafe { NSWorkspaceDidLaunchApplicationNotification },
//         unsafe { NSWorkspaceDidActivateApplicationNotification },
//         unsafe { NSWorkspaceDidHideApplicationNotification },
//         unsafe { NSWorkspaceDidUnhideApplicationNotification },
//         unsafe { NSWorkspaceActiveSpaceDidChangeNotification },
//         // unsafe { NSWorkspaceDidTerminateApplicationNotification },
//         // NSWorkspaceActiveDisplayDidChangeNotification
//         // AppleInterfaceMenuBarHidingChangedNotification
//     ];

//     let center = unsafe { NSNotificationCenter::defaultCenter() };
//     for notification in GLOBAL_NOTIFICATIONS {
//         let something = unsafe {
//             center.addObserverForName_object_queue_usingBlock(
//                 Some(notification),
//                 None,
//                 None,
//                 &block,
//             )
//         };
//     }
// }

fn iter_apps() -> impl Iterator<Item = Result<Application, WindowError>> {
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

// fn running_apps() -> Result<Vec<Application>, WindowError> {
//     let mut apps = Vec::new();

//     let workspace = unsafe { NSWorkspace::sharedWorkspace().runningApplications() };
//     for app in workspace {
//         // a prohibited application can never have a window
//         if unsafe { app.activationPolicy() } != NSApplicationActivationPolicyProhibited {
//             let pid = unsafe { NSRunningApplication_processIdentifier(&app) };
//             // if it's -1 then it isn't associated with a process
//             if pid != -1 {
//                 apps.push(Application::new(pid)?)
//             }
//         }
//     }

//     Ok(apps)
// }

impl WindowError {
    // TODO:
    // https://developer.apple.com/documentation/applicationservices/axerror?language=objc
    pub(self) fn from_ax_error(code: i32) -> WindowError {
        match code {
            ffi::kAXErrorAPIDisabled => WindowError::NotTrusted,
            ffi::kAXErrorIllegalArgument => WindowError::InvalidInternalArgument,
            ffi::kAXErrorNoValue => WindowError::InvalidInternalArgument,
            ffi::kAXErrorInvalidUIElementObserver => WindowError::InvalidInternalArgument,
            ffi::kAXErrorNotificationUnsupported => WindowError::InvalidInternalArgument,
            ffi::kAXErrorNotificationAlreadyRegistered => WindowError::AlreadyWatching,
            ffi::kAXErrorNotificationNotRegistered => WindowError::WasNeverWatching,
            ffi::kAXErrorInvalidUIElement => WindowError::InvalidHandle,
            ffi::kAXErrorNotImplemented => WindowError::AlienUnsupported,
            ffi::kAXErrorAttributeUnsupported => WindowError::AlienUnsupported,
            ffi::kAXErrorActionUnsupported => WindowError::AlienUnsupported,
            ffi::kAXErrorParameterizedAttributeUnsupported => WindowError::AlienUnsupported,
            ffi::kAXErrorCannotComplete => WindowError::ArbitraryFailure,
            ffi::kAXErrorFailure => WindowError::ArbitraryFailure,
            _ => WindowError::ArbitraryFailure,
        }
    }
}
