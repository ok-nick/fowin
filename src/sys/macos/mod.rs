use std::ptr::NonNull;

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

use crate::protocol::{Position, Size, WindowEventInfo, WindowId, WindowManagerBackend};

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

pub fn all_apps() -> Result<Vec<Application>, ()> {
    let mut apps = Vec::new();

    let workspace = unsafe { NSWorkspace::sharedWorkspace().runningApplications() };
    // TODO: filter by activation policy regular?
    for app in workspace {
        let pid = unsafe { NSRunningApplication_processIdentifier(&app) };
        match pid {
            -1 => {
                // doesn't exist
            }
            _ => apps.push(Application::new(pid)?),
        }
    }

    Ok(apps)
}
// TODO: reorganize this, I should probably store apps within the window manager?
#[derive(Debug)]
pub struct WindowManager {
    apps: Vec<Application>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self { apps: Vec::new() }
    }

    pub fn listen_global_events(&self, sender: Sender<WindowEventInfo>) {
        // let selector = sel!(global_notification);

        // let mut builder = ClassBuilder::new("TODO", NSObject::class()).unwrap();
        // unsafe { builder.add_method(selector, global_notification) }
        // let class = builder.register();

        // let center = unsafe { NSWorkspace::sharedWorkspace().notificationCenter() };
        // for notification in GLOBAL_NOTIFICATIONS {
        //     unsafe {
        //         center.addObserver_selector_name_object(class, selector, Some(notification), None);
        //     }
        // }
        let center = unsafe { NSNotificationCenter::defaultCenter() };
        // let block = ConcreteBlock::new(move |notification: NonNull<NSNotification>| {
        //     let notification = unsafe { notification.as_ref() };
        // })
        let block = ConcreteBlock::new(move |notification: NonNull<NSNotification>| {
            let notification = unsafe { notification.as_ref() };
            let name = unsafe { notification.name() };

            // TODO: standardize event names, construct windowevent, send to sender
            if unsafe { &*name == (NSWorkspaceDidLaunchApplicationNotification) } {
                // TODO: we need to add this app to the watched windows
                // should we do it from here or should we send an event for the user to add it? The latter may be more useful so they can idiomatically filter applications
                todo!()
            } else if unsafe { &*name == (NSWorkspaceDidActivateApplicationNotification) } {
                todo!()
            } else if unsafe { &*name == (NSWorkspaceDidHideApplicationNotification) } {
                todo!()
            } else if unsafe { &*name == (NSWorkspaceDidUnhideApplicationNotification) } {
                todo!()
            } else if unsafe { &*name == (NSWorkspaceActiveSpaceDidChangeNotification) } {
                todo!()
            } else if unsafe { &*name == (NSWorkspaceDidTerminateApplicationNotification) } {
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

// TODO: how will I keep track of windows, what data type are window ids?
impl WindowManagerBackend for WindowManager {
    fn show_window(&self, id: WindowId) {
        todo!()
    }

    fn hide_window(&self, id: WindowId) {
        todo!()
    }

    fn focus_window(&self, id: WindowId) {
        todo!()
    }

    fn move_window(&self, id: WindowId, position: Position) {
        // AXUIElementSetAttributeValue
        // kAXPositionAttribute
        todo!()
    }

    fn resize_window(&self, id: WindowId, size: Size) {
        todo!()
    }
}

// TODO: using selector-based API
// declare_class!(
//     struct TODO;

//     unsafe impl ClassType for TODO {
//         type Super = NSObject;
//         type Mutability = mutability::Immutable;
//         const NAME: &'static str = "TODO";
//     }

//     unsafe impl TODO {
//         #[method(global_notification:)]
//         fn global_notification(&self, notification: &NSNotification) {}
//     }
// );

// fn global_notification(notification: NonNull<NSNotification>) {
// TODO: the notification object can store data (such as the sender)
// }
