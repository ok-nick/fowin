use std::{mem::MaybeUninit, time::Instant};

use crossbeam::channel::Sender;

use crate::protocol::{WindowEvent, WindowEventInfo};

use super::{
    ffi::{
        cfstring_from_str, kAXErrorSuccess, kAXFocusedWindowChangedNotification,
        kAXMovedNotification, kAXTitleChangedNotification, kAXUIElementDestroyedNotification,
        kAXWindowCreatedNotification, kAXWindowDeminiaturizedNotification,
        kAXWindowMiniaturizedNotification, kAXWindowsAttribute, kCFRunLoopDefaultMode, pid_t,
        AXObserverAddNotification, AXObserverCreate, AXObserverGetRunLoopSource, AXObserverRef,
        AXObserverRemoveNotification, AXUIElementCopyAttributeValue, AXUIElementCreateApplication,
        AXUIElementRef, CFEqual, CFRelease, CFRunLoopAddSource, CFRunLoopGetMain,
        CFRunLoopSourceInvalidate, CFStringRef,
    },
    window::Window,
};

const APP_NOTIFICATIONS: [&str; 7] = [
    kAXWindowCreatedNotification,
    kAXUIElementDestroyedNotification,
    kAXWindowMiniaturizedNotification,
    kAXWindowDeminiaturizedNotification,
    kAXFocusedWindowChangedNotification,
    kAXMovedNotification,
    kAXTitleChangedNotification,
];

#[derive(Debug)]
pub struct Application {
    inner: AXUIElementRef,
    observer: AXObserverRef,
    pid: pid_t,
}

impl Application {
    pub fn new(pid: pid_t) -> Result<Self, ()> {
        let element = unsafe { AXUIElementCreateApplication(pid) };
        let mut observer = MaybeUninit::uninit();
        let result =
            unsafe { AXObserverCreate(pid, Some(app_notification), observer.as_mut_ptr()) };
        // TODO: error codes: https://developer.apple.com/documentation/applicationservices/axerror
        // TODO: delegate errors to result
        if result == kAXErrorSuccess {
            Ok(Application {
                inner: element,
                observer: unsafe { observer.assume_init() },
                pid,
            })
        } else {
            // TODO
            Err(())
        }
    }

    pub fn windows(&self) {
        let mut windows = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.inner,
                cfstring_from_str(kAXWindowsAttribute),
                windows.as_mut_ptr(),
            )
        };
        // TODO: errors
        if result == kAXErrorSuccess {
            // should be a CFArray
            let windows = unsafe { windows.assume_init() };
            // TODO: window struct?
        }
    }

    pub fn observe(&self, sender: Sender<WindowEventInfo>) {
        for notification in APP_NOTIFICATIONS {
            let result = unsafe {
                AXObserverAddNotification(
                    self.observer,
                    self.inner,
                    cfstring_from_str(notification),
                    // TODO: CLEAN THIS UP ON DROP!!
                    Box::into_raw(Box::new(sender.clone())) as *mut _,
                )
            };
            if result != kAXErrorSuccess {
                // TODO: delegate errors to result
                return;
            }
        }

        unsafe {
            CFRunLoopAddSource(
                // CFRunLoopGetMain or CFRunLoopGetCurrent?
                CFRunLoopGetMain(),
                AXObserverGetRunLoopSource(self.observer),
                kCFRunLoopDefaultMode,
            );
        }
    }

    // TODO: call on app terminated?
    pub fn unobserve(&self) {
        for notification in APP_NOTIFICATIONS {
            let result = unsafe {
                AXObserverRemoveNotification(
                    self.observer,
                    self.inner,
                    // TODO: cache this
                    cfstring_from_str(notification),
                )
            };
            if result != kAXErrorSuccess {
                // TODO: delegate errors to result
                return;
            }
        }

        unsafe {
            CFRunLoopSourceInvalidate(AXObserverGetRunLoopSource(self.observer));
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.inner as *const _);
            CFRelease(self.observer as *const _);
        }
    }
}

unsafe extern "C" fn app_notification(
    observer: AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    refcon: *mut ::std::os::raw::c_void,
) {
    let timestamp = Instant::now();
    let window = Window::new(element);

    let event = if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXWindowCreatedNotification) as *const _,
    ) != 0
    {
        // TODO: use AXUIElementCopyAttributeValue

        WindowEvent::Opened
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXUIElementDestroyedNotification) as *const _,
    ) != 0
    {
        WindowEvent::Closed
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXWindowMiniaturizedNotification) as *const _,
    ) != 0
    {
        WindowEvent::Hidden
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXWindowDeminiaturizedNotification) as *const _,
    ) != 0
    {
        WindowEvent::Shown
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXFocusedWindowChangedNotification) as *const _,
    ) != 0
    {
        WindowEvent::Focused
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXMovedNotification) as *const _,
    ) != 0
    {
        WindowEvent::Moved
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXTitleChangedNotification) as *const _,
    ) != 0
    {
        WindowEvent::Renamed
    } else {
        // TODO: technically not reachable, but who knows
        unreachable!()
    };

    // crossbeam::channel::Sender is both Send + Sync, so we don't need to take care of synchronization
    let sender = &mut *(refcon as *mut Sender<WindowEventInfo>);
    // if error, then it was disconnected, thus, do nothing
    let _ = sender.send(WindowEventInfo {
        event,
        timestamp,
        window,
    });
}
