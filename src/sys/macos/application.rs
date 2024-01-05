use std::{marker::PhantomData, mem::MaybeUninit, time::Instant};

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
        CFEqual, CFGetRetainCount, CFRelease, CFRetain, CFRunLoopAddSource, CFRunLoopGetCurrent,
        CFRunLoopGetMain, CFRunLoopSourceInvalidate, CFStringRef, __AXObserver, __AXUIElement,
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

// TODO: how does it behave when Window drops the inner AXUIElementRef?
#[derive(Debug)]
pub struct WindowIterator<'a> {
    inner: CFArrayRef,
    len: i64,
    index: i64,
    phantom: PhantomData<&'a ()>,
}

// impl<'a> Iterator for WindowIterator<'a> {
//     type Item = Window;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.index < self.len {
//             let window = Window::new(AXUIElementRef(unsafe {
//                 CFArrayGetValueAtIndex(self.inner, self.index) as *const __AXUIElement
//             }));

//             self.index += 1;

//             // TODO: temp unwrap
//             Some(window.unwrap())
//         } else {
//             None
//         }
//     }
// }

// TODO: not sure if the AXUIElementRef is always valid, this thread is for windows:
//     https://lists.apple.com/archives/accessibility-dev/2013/Jun/msg00045.html
// TODO: UIElementRefs can be compared for equality using CFEqual, impl Eq for Window as well
//       https://lists.apple.com/archives/accessibility-dev/2006/Jun/msg00010.html
#[derive(Debug, Clone)]
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
                inner: AXUIElementRef(element),
                observer: AXObserverRef(unsafe { observer.assume_init() }),
                pid,
            })
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    // TODO: I can return a custom struct that wraps the CFArrayRef to avoid copying, or even better, return an iterator
    pub fn windows(&self) -> Result<Vec<Result<Window, WindowError>>, WindowError> {
        let raw_windows = raw_windows(&self.inner)?;
        unsafe {
            println!(
                "CFARRAY_START: {}",
                CFGetRetainCount(raw_windows as *const _)
            );
        }

        let len = unsafe { CFArrayGetCount(raw_windows) };
        let mut windows = Vec::with_capacity(len as usize);
        for i in 0..len {
            let element =
                AXUIElementRef(unsafe { CFArrayGetValueAtIndex(raw_windows, i) as *const _ })
                    .clone();
            // unsafe {
            //     println!(
            //         "AXUIElementRef_START: {}",
            //         CFGetRetainCount(element as *const _)
            //     );
            // }
            windows.push(Window::new(element, self.inner.clone()));
        }

        unsafe {
            CFRelease(raw_windows as *const _);
        }

        // unsafe {
        //     println!("CFARRAY_END: {}", CFGetRetainCount(raw_windows as *const _));
        // }
        // if !windows.is_empty() {
        //     unsafe {
        //         println!(
        //             "AXUIElementRef_END: {}",
        //             CFGetRetainCount(*windows.first().unwrap() as *const _)
        //         );
        //     }
        // }

        Ok(windows)
        // Ok(WindowIterator {
        //     inner: cfarray,
        //     len: unsafe { CFArrayGetCount(cfarray) },
        //     index: 0,
        //     phantom: PhantomData,
        // })
    }

    pub fn watch(&self, sender: Sender<WindowEvent>) -> Result<(), WindowError> {
        for notification in APP_NOTIFICATIONS {
            let result = unsafe {
                AXObserverAddNotification(
                    self.observer.0,
                    self.inner.0,
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
                AXObserverGetRunLoopSource(self.observer.0),
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
                    self.observer.0,
                    self.inner.0,
                    // TODO: cache this
                    cfstring_from_str(notification),
                )
            };
            if result != kAXErrorSuccess {
                return Err(WindowError::from_ax_error(result));
            }
        }

        unsafe {
            CFRunLoopSourceInvalidate(AXObserverGetRunLoopSource(self.observer.0));
        }

        Ok(())
    }
}

unsafe extern "C" fn app_notification(
    _observer: *mut __AXObserver,
    element: *const __AXUIElement,
    notification: CFStringRef,
    refcon: *mut ::std::os::raw::c_void,
) {
    let timestamp = Instant::now();
    // the clone is a cheeky way of calling CFRetain
    // TODO: pass in the self.inner of app
    let window = Window::new(AXUIElementRef(element.clone()), todo!());

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
        // TODO: temp unwrap
        protocol::Window(window.unwrap()),
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
