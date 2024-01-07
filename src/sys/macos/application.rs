use std::{mem::MaybeUninit, os::raw, time::Instant};

use crossbeam_channel::Sender;

use crate::{
    protocol::{self, WindowError, WindowEvent, WindowEventKind},
    sys::platform::ffi::{CFArrayGetCount, CFArrayGetValueAtIndex},
};

use super::{
    ffi::{
        cfstring_from_str, kAXApplicationHiddenNotification, kAXApplicationShownNotification,
        kAXErrorSuccess, kAXFocusedWindowChangedNotification, kAXMovedNotification,
        kAXResizedNotification, kAXTitleChangedNotification, kAXUIElementDestroyedNotification,
        kAXWindowCreatedNotification, kAXWindowDeminiaturizedNotification,
        kAXWindowMiniaturizedNotification, kAXWindowsAttribute, kCFRunLoopDefaultMode, pid_t,
        AXObserverAddNotification, AXObserverCreate, AXObserverGetRunLoopSource, AXObserverRef,
        AXUIElementCopyAttributeValue, AXUIElementCreateApplication, AXUIElementRef, CFArrayRef,
        CFEqual, CFRelease, CFRunLoopAddSource, CFRunLoopGetCurrent, CFStringRef, __AXObserver,
        __AXUIElement, kAXErrorNotificationUnsupported,
    },
    window::Window,
};

// https://github.com/appium/appium-for-mac/blob/9e154e7de378374760344abd8572338535d6b7d8/Frameworks/PFAssistive.framework/Versions/J/Headers/PFUIElement.h#L961-L994
const APP_NOTIFICATIONS: [&str; 10] = [
    kAXWindowCreatedNotification,
    kAXUIElementDestroyedNotification,
    kAXWindowMiniaturizedNotification,
    kAXWindowDeminiaturizedNotification,
    kAXFocusedWindowChangedNotification,
    // kAXFocusedUIElementChangedNotification - states when the app was focused?
    // kAXApplicationActivatedNotification - also states when app was focused?
    kAXMovedNotification,
    kAXResizedNotification,
    kAXTitleChangedNotification,
    // TODO: the issue states these events happen too soon, to the point where the window is visible but not movable (yet)
    //       I'd like to do a little more experimentation on these events before moving to NSWorkspace notifications
    // https://github.com/ianyh/Amethyst/issues/662
    kAXApplicationShownNotification,
    // TODO: https://github.com/appium/appium-for-mac/blob/9e154e7de378374760344abd8572338535d6b7d8/Frameworks/PFAssistive.framework/Versions/J/Headers/PFUIElement.h#L412
    // AXUIElementRef for windows are destroyed when app is hidden?
    kAXApplicationHiddenNotification,
];

#[derive(Debug, Clone)]
pub struct Application {
    inner: AXUIElementRef,
    pid: pid_t,
}

impl Application {
    pub fn new(pid: pid_t) -> Application {
        Application {
            inner: AXUIElementRef(unsafe { AXUIElementCreateApplication(pid) }),
            pid,
        }
    }

    pub fn windows(&self) -> Result<WindowIterator, WindowError> {
        let raw_windows = raw_windows(&self.inner)?;
        let len = unsafe { CFArrayGetCount(raw_windows) };
        Ok(WindowIterator {
            inner: raw_windows,
            app_handle: self.inner.clone(),
            len,
            index: 0,
        })
    }

    pub fn watch(&self, sender: Sender<WindowEvent>) -> Result<Watcher, WindowError> {
        Watcher::new(self.pid, self.inner.clone(), sender)
    }
}

#[derive(Debug)]
pub struct WindowIterator {
    inner: CFArrayRef,
    app_handle: AXUIElementRef,
    len: i64,
    index: i64,
}

impl Iterator for WindowIterator {
    type Item = Result<Window, WindowError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let element = AXUIElementRef(unsafe {
                CFArrayGetValueAtIndex(self.inner, self.index) as *const _
            });
            element.increment_ref_count();

            self.index += 1;

            Some(Window::new(element, self.app_handle.clone()))
        } else {
            None
        }
    }
}

impl Drop for WindowIterator {
    fn drop(&mut self) {
        // TODO: handle this from within CFArrayRef itself
        unsafe {
            CFRelease(self.inner as *const _);
        }
    }
}

#[derive(Debug)]
pub struct Watcher {
    // sender is implicitly dropped after observer, so it's safe
    observer: AXObserverRef,
    callback_info: Box<CallbackInfo>,
}

impl Watcher {
    pub fn new(
        pid: pid_t,
        app_handle: AXUIElementRef,
        sender: Sender<WindowEvent>,
    ) -> Result<Watcher, WindowError> {
        let mut observer = MaybeUninit::uninit();
        let result =
            unsafe { AXObserverCreate(pid, Some(app_notification), observer.as_mut_ptr()) };
        match result {
            kAXErrorSuccess => {
                let observer = AXObserverRef(unsafe { observer.assume_init() });
                let raw_app_handle = app_handle.0;
                let callback_info = Box::into_raw(Box::new(CallbackInfo { app_handle, sender }));

                for notification in APP_NOTIFICATIONS {
                    let result = unsafe {
                        AXObserverAddNotification(
                            observer.0,
                            raw_app_handle,
                            cfstring_from_str(notification),
                            callback_info as *mut _,
                        )
                    };
                    match result {
                        // if the notification is unsupported, there's nothing we can do
                        kAXErrorNotificationUnsupported => {}
                        _ => {
                            return Err(WindowError::from_ax_error(result));
                        }
                    }
                }

                unsafe {
                    CFRunLoopAddSource(
                        // TODO: read above window.rs struct, not sure if it must run on main thread?
                        CFRunLoopGetCurrent(),
                        AXObserverGetRunLoopSource(observer.0),
                        kCFRunLoopDefaultMode,
                    );
                }

                Ok(Watcher {
                    observer,
                    callback_info: unsafe { Box::from_raw(callback_info) },
                })
            }
            _ => Err(WindowError::from_ax_error(result)),
        }
    }

    // NOTE: I can't see this being useful. To unwatch, drop the Watcher. To watch again, create a new Watcher.
    //       To support this method, we'd need to keep track of registered notifications via a HashSet and perform
    //       additional logic.
    // pub fn unwatch(&self) -> Result<(), WindowError> {
    //     for notification in APP_NOTIFICATIONS {
    //         let result = unsafe {
    //             AXObserverRemoveNotification(
    //                 self.observer.0,
    //                 self.inner.0,
    //                 // TODO: cache this
    //                 cfstring_from_str(notification),
    //             )
    //         };
    //         if result != kAXErrorSuccess {
    //             return Err(WindowError::from_ax_error(result));
    //         }
    //     }

    //     unsafe {
    //         CFRunLoopSourceInvalidate(AXObserverGetRunLoopSource(self.observer.0));
    //     }

    //     Ok(())
    // }
}

#[derive(Debug)]
pub struct CallbackInfo {
    app_handle: AXUIElementRef,
    sender: Sender<WindowEvent>,
}

unsafe extern "C" fn app_notification(
    _observer: *mut __AXObserver,
    element: *const __AXUIElement,
    notification: CFStringRef,
    refcon: *mut raw::c_void,
) {
    let timestamp = Instant::now();

    // TODO: I believe when, for example, app hidden event is sent, the element parameter here is a reference to the applications element
    // more here: https://github.com/appium/appium-for-mac/blob/9e154e7de378374760344abd8572338535d6b7d8/Frameworks/PFAssistive.framework/Versions/J/Headers/PFUIElement.h#L961-L994
    // if an app is focused, for example, the window foucsed can be the app if no winodw currently exists

    let info = &*(refcon as *mut CallbackInfo);
    let window = match Window::new(AXUIElementRef(element).clone(), info.app_handle.clone()) {
        Ok(window) => window,
        // if we can't make a window, then we can't get its id, which means there's some fishy business going on...
        Err(_) => return,
    };

    let kind = if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXWindowCreatedNotification) as *const _,
    ) != 0
    {
        WindowEventKind::Opened
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXUIElementDestroyedNotification) as *const _,
    ) != 0
    {
        WindowEventKind::Closed
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXWindowMiniaturizedNotification) as *const _,
    ) != 0
    {
        WindowEventKind::Hidden
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXWindowDeminiaturizedNotification) as *const _,
    ) != 0
    {
        WindowEventKind::Shown
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXFocusedWindowChangedNotification) as *const _,
    ) != 0
    {
        WindowEventKind::Focused
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXMovedNotification) as *const _,
    ) != 0
    {
        WindowEventKind::Moved
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXResizedNotification) as *const _,
    ) != 0
    {
        WindowEventKind::Resized
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXTitleChangedNotification) as *const _,
    ) != 0
    {
        WindowEventKind::Renamed
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXApplicationShownNotification) as *const _,
    ) != 0
    {
        WindowEventKind::Shown
    } else if CFEqual(
        notification as *const _,
        cfstring_from_str(kAXApplicationHiddenNotification) as *const _,
    ) != 0
    {
        WindowEventKind::Hidden
    } else {
        // technically unreachable, but who knows
        return;
    };

    // crossbeam::channel::Sender is both Send + Sync, so we don't need to take care of synchronization
    let sender = &mut *(refcon as *mut Sender<WindowEvent>);
    // if error, then it was disconnected, thus, do nothing
    let _ = sender.send(WindowEvent::with_timestamp(
        kind,
        protocol::Window(window),
        timestamp,
    ));
}

pub(super) fn raw_windows(inner: &AXUIElementRef) -> Result<CFArrayRef, WindowError> {
    let mut windows: MaybeUninit<CFArrayRef> = MaybeUninit::uninit();
    let result = unsafe {
        AXUIElementCopyAttributeValue(
            inner.0,
            cfstring_from_str(kAXWindowsAttribute),
            windows.as_mut_ptr() as *mut _,
        )
    };
    if result == kAXErrorSuccess {
        Ok(unsafe { windows.assume_init() })
    } else {
        Err(WindowError::from_ax_error(result))
    }
}
