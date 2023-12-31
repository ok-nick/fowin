use std::{collections::HashMap, iter, ptr::NonNull};

use crossbeam::channel::Sender;
use icrate::{
    block2::ConcreteBlock,
    AppKit::{
        NSWorkspace, NSWorkspaceActiveSpaceDidChangeNotification,
        NSWorkspaceDidActivateApplicationNotification, NSWorkspaceDidHideApplicationNotification,
        NSWorkspaceDidLaunchApplicationNotification,
        NSWorkspaceDidTerminateApplicationNotification,
        NSWorkspaceDidUnhideApplicationNotification,
    },
    Foundation::{NSNotification, NSNotificationCenter},
};

use crate::protocol::{DisplayId, Position, Size, WindowEventInfo, WindowId, WindowManagerBackend};

use self::ffi::NSRunningApplication_processIdentifier;
pub use self::{application::Application, window::Window};

mod application;
mod ffi;
mod window;

// TODO: various properties of windows
// https://github.com/nikitabobko/AeroSpace/blob/0569bb0d663ebf732c2ea12cc168d4ff60378394/src/util/accessibility.swift#L24
// interesting: https://github.com/nikitabobko/AeroSpace/blob/0569bb0d663ebf732c2ea12cc168d4ff60378394/src/util/accessibility.swift#L296

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
pub struct WindowManager {
    // users only know about windows, apps aren't exposed to them
    apps: Vec<Application>,
    windows: HashMap<WindowId, Window>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            apps: Vec::new(),
            windows: HashMap::new(),
        }
    }

    // TODO: dedup here?
    // aka add all currently running apps
    pub fn init(&mut self) -> Result<(), ()> {
        let workspace = unsafe { NSWorkspace::sharedWorkspace().runningApplications() };
        // TODO: filter by activation policy regular?
        for app in workspace {
            let pid = unsafe { NSRunningApplication_processIdentifier(&app) };
            match pid {
                -1 => {
                    // doesn't exist
                }
                _ => self.apps.push(Application::new(pid)?),
            }
        }

        Ok(())
    }

    // since apps.windows() takes a ref to the app, need to extend the lifetime to self
    pub fn iter_windows(&mut self) -> impl Iterator<Item = Result<Window, ()>> + '_ {
        // an absolute monster of an iterator
        self.apps.iter().flat_map(|app| {
            app.windows()
                .into_iter()
                .flat_map(|windows| windows.into_iter().map(Ok::<Window, ()>))
        })
    }

    pub fn watch_windows(&self, sender: Sender<WindowEventInfo>) {
        let block = ConcreteBlock::new(move |notification: NonNull<NSNotification>| {
            let notification = unsafe { notification.as_ref() };
            let name = unsafe { notification.name() };

            if unsafe { &*name == NSWorkspaceDidLaunchApplicationNotification } {
                // TODO: construct an Application and add it to self.apps, then watch the application with the sender in param
                // TODO: I think you can only construct an application on the main thread and I don't think this block is guaranteed to be called on the main thread?

                todo!()
            } else if unsafe { &*name == NSWorkspaceDidActivateApplicationNotification } {
                // TODO: I think this one has to do with focus
                todo!()
            } else if unsafe { &*name == NSWorkspaceDidHideApplicationNotification } {
                // TODO: find windows corresponding to app and send hidden event
                todo!()
            } else if unsafe { &*name == NSWorkspaceDidUnhideApplicationNotification } {
                // TODO: ^
                todo!()
            } else if unsafe { &*name == NSWorkspaceActiveSpaceDidChangeNotification } {
                // TODO: this can cause many windows to go "hidden," so send the hidden events to the user
                todo!()
            } else if unsafe { &*name == NSWorkspaceDidTerminateApplicationNotification } {
                // TODO: can we easily gather all windows for the app and remove them from the cache? (self.windows)
                todo!()
            };
        })
        .copy();

        // TODO: initialize as static
        let GLOBAL_NOTIFICATIONS = [
            unsafe { NSWorkspaceDidLaunchApplicationNotification },
            unsafe { NSWorkspaceDidActivateApplicationNotification },
            unsafe { NSWorkspaceDidHideApplicationNotification },
            unsafe { NSWorkspaceDidUnhideApplicationNotification },
            unsafe { NSWorkspaceActiveSpaceDidChangeNotification },
            unsafe { NSWorkspaceDidTerminateApplicationNotification },
            // NSWorkspaceActiveDisplayDidChangeNotification
            // AppleInterfaceMenuBarHidingChangedNotification
        ];

        let center = unsafe { NSNotificationCenter::defaultCenter() };
        for notification in GLOBAL_NOTIFICATIONS {
            unsafe {
                center.addObserverForName_object_queue_usingBlock(
                    Some(notification),
                    None,
                    None,
                    &block,
                );
            }
        }
    }
}

impl WindowManagerBackend for WindowManager {
    fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }
}
