use std::{mem::MaybeUninit, time::Instant};

use crossbeam_channel::Sender;

use crate::{
    protocol::{self, WindowError, WindowEvent, WindowEventKind},
    sys::platform::ffi::{CFArrayGetCount, CFArrayGetValueAtIndex},
};

use super::{
    ffi::{
        cfarray_to_vec, cfstring_from_str, kAXApplicationHiddenNotification,
        kAXApplicationShownNotification, kAXErrorSuccess, kAXFocusedWindowChangedNotification,
        kAXMovedNotification, kAXResizedNotification, kAXTitleChangedNotification,
        kAXUIElementDestroyedNotification, kAXWindowCreatedNotification,
        kAXWindowDeminiaturizedNotification, kAXWindowMiniaturizedNotification,
        kAXWindowsAttribute, kCFRunLoopDefaultMode, pid_t, AXObserverAddNotification,
        AXObserverCreate, AXObserverGetRunLoopSource, AXObserverRef, AXObserverRemoveNotification,
        AXUIElementCopyAttributeValue, AXUIElementCreateApplication, AXUIElementRef, CFArrayRef,
        CFEqual, CFRelease, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopGetMain,
        CFRunLoopSourceInvalidate, CFStringRef, __AXUIElement,
    },
    window::Window,
};

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
    kAXApplicationHiddenNotification,
];

// TODO: UIElementRefs can be compared for equality using CFEqual, impl Eq for Window as well
//       https://lists.apple.com/archives/accessibility-dev/2006/Jun/msg00010.html
#[derive(Debug)]
pub struct Application {
    inner: AXUIElementRef,
    // TODO: is this thread-safe?? It's a CFType
    observer: AXObserverRef,
    pid: pid_t,
}

impl Application {
    pub fn new(pid: pid_t) -> Result<Self, WindowError> {
        let element = unsafe { AXUIElementCreateApplication(pid) };
        let mut observer = MaybeUninit::uninit();
        let result =
            unsafe { AXObserverCreate(pid, Some(app_notification), observer.as_mut_ptr()) };
        if result == kAXErrorSuccess {
            Ok(Application {
                inner: element,
                observer: unsafe { observer.assume_init() },
                pid,
            })
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    // TODO: I can return a custom struct that wraps the CFArrayRef to avoid copying, or even better, return an iterator
    // TODO: how does CGWindowListCopyWindowInfo compare?
    pub fn windows(&self) -> Result<Vec<Window>, WindowError> {
        let mut windows: MaybeUninit<CFArrayRef> = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.inner,
                cfstring_from_str(kAXWindowsAttribute),
                windows.as_mut_ptr() as *mut _,
            )
        };
        if result == kAXErrorSuccess {
            let cfarray = unsafe { windows.assume_init() };

            let len = unsafe { CFArrayGetCount(cfarray) };
            let mut windows = Vec::with_capacity(len as usize);
            for i in 0..len {
                let element = AXUIElementRef(unsafe {
                    CFArrayGetValueAtIndex(cfarray, i) as *const __AXUIElement
                });
                windows.push(Window::new(element));
            }

            unsafe {
                CFRelease(cfarray as *const _);
            }

            Ok(windows)
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn watch(&self, sender: Sender<WindowEvent>) -> Result<(), WindowError> {
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
                return Err(WindowError::from_ax_error(result));
            }
        }

        unsafe {
            CFRunLoopAddSource(
                // TODO: read above window.rs struct, not sure if it must run on main thread?
                CFRunLoopGetCurrent(),
                AXObserverGetRunLoopSource(self.observer),
                kCFRunLoopDefaultMode,
            );
        }

        Ok(())
    }

    // TODO: call on app terminated?
    pub fn unwatch(&self) -> Result<(), WindowError> {
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
                return Err(WindowError::from_ax_error(result));
            }
        }

        unsafe {
            CFRunLoopSourceInvalidate(AXObserverGetRunLoopSource(self.observer));
        }

        Ok(())
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.inner.0 as *const _);
            CFRelease(self.observer as *const _);
        }
    }
}

unsafe extern "C" fn app_notification(
    _observer: AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    refcon: *mut ::std::os::raw::c_void,
) {
    let timestamp = Instant::now();
    let window = Window::new(element);

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
        // TODO: technically not reachable, but who knows
        unreachable!()
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
