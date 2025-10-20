use std::{mem::MaybeUninit, os::raw, ptr::NonNull, sync::mpsc::Sender, time::Duration};

use libc::pid_t;
use objc2_application_services::{
    AXError, AXObserver, AXUIElement, AXUIElementCopyAttributeValue, AXUIElementCreateApplication,
    AXUIElementSetMessagingTimeout,
};
use objc2_core_foundation::{
    kCFRunLoopDefaultMode, CFArray, CFRetained, CFRunLoop, CFString, CFType,
};

use crate::{
    protocol::{self, WindowError, WindowEvent},
    sys::platform::{
        ffi::{
            cfstring_to_string, kAXFocusedWindowAttribute, AXUIElementGetPid, CFArrayGetCount,
            CFArrayGetValueAtIndex, CFHash, CFRetain,
        },
        ffi2::CFRetainedSafe,
    },
};

use super::{
    ffi::{
        cfstring_from_str, kAXApplicationHiddenNotification, kAXApplicationShownNotification,
        kAXErrorSuccess, kAXFocusedWindowChangedNotification, kAXMovedNotification,
        kAXResizedNotification, kAXTitleChangedNotification, kAXUIElementDestroyedNotification,
        kAXWindowCreatedNotification, kAXWindowDeminiaturizedNotification,
        kAXWindowMiniaturizedNotification, kAXWindowsAttribute, AXObserverAddNotification,
        AXObserverCreate, AXObserverGetRunLoopSource, AXObserverRef, AXUIElementRef, CFArrayRef,
        CFRelease, CFRunLoopAddSource, CFStringRef, CGWindowID, _AXUIElementGetWindow,
        __AXObserver, __AXUIElement, __CFRunLoopSource, kAXApplicationActivatedNotification,
        kAXErrorCannotComplete, kAXErrorNotificationUnsupported,
    },
    window::Window,
};

const DEFAULT_AX_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub struct Application {
    inner: CFRetainedSafe<AXUIElement>,
    pid: pid_t,
    timeout: Duration,
}

impl Application {
    pub fn new(pid: pid_t) -> Application {
        Application::with_timeout(pid, DEFAULT_AX_TIMEOUT)
    }

    // TODO: timeouts should be exposed to the user
    pub fn with_timeout(pid: pid_t, timeout: Duration) -> Application {
        let inner = unsafe { AXUIElement::new_application(pid) };
        unsafe {
            // TODO: handle err
            inner.set_messaging_timeout(timeout.as_secs_f32());
        }

        Application {
            inner: CFRetainedSafe(inner),
            pid,
            timeout,
        }
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub fn pid(&self) -> pid_t {
        self.pid
    }

    // TODO: return iterator not struct?
    pub fn iter_windows(&self) -> Result<WindowIterator, WindowError> {
        let raw_windows = raw_windows(&self.inner)?;
        let len = raw_windows.as_opaque().count();
        Ok(WindowIterator {
            inner: raw_windows,
            app_handle: self.inner.0.clone(),
            len,
            index: 0,
        })
    }

    pub fn supported(&self) {
        // TODO: return a list of notifications that are able to be registered by this app
        //       probably should do it on a window level, since we register windows now
    }

    pub fn watch(
        &self,
        sender: Sender<Result<WindowEvent, WindowError>>,
    ) -> Result<Watcher, WindowError> {
        Watcher::new(self, sender)
    }

    // Some processes aren't immediately accessible by the AX API and default to erroring with kAXErrorCannotComplete.
    // In that case, we should retry (ideally on a separate thread) until timeout is reached. Some applications already
    // take more than 1 second to respond, although, those applications typically don't allow access to their AX APIs
    // anyways. If there is a valid application that takes over 1 second to respond (extremely unlikely), then oops.
    pub(crate) fn should_wait(&self) -> bool {
        let mut _id: MaybeUninit<CGWindowID> = MaybeUninit::zeroed();
        unsafe {
            // TODO: avoid using private API?
            _AXUIElementGetWindow(&self.inner, _id.as_mut_ptr() as *mut _)
                == AXError::CannotComplete.0
        }
    }
}

#[derive(Debug)]
pub struct WindowIterator {
    inner: CFRetained<CFArray<AXUIElement>>,
    app_handle: CFRetained<AXUIElement>,
    len: isize,
    index: isize,
}

impl Iterator for WindowIterator {
    type Item = Result<Window, WindowError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let element = unsafe {
                CFRetained::retain(NonNull::new(
                    self.inner.as_opaque().value_at_index(self.index) as *mut AXUIElement,
                )?)
            };

            self.index += 1;

            Some(Window::new(element, self.app_handle.clone()))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Watcher {
    // Resources are implicitly dropped after observer, so it's safe.
    observer: CFRetainedSafe<AXObserver>,
    resources: Vec<Box<CallbackInfo>>,
}

impl Watcher {
    // Some additional information about these events:
    // https://github.com/appium/appium-for-mac/blob/9e154e7de378374760344abd8572338535d6b7d8/Frameworks/PFAssistive.framework/Versions/J/Headers/PFUIElement.h#L961-L994
    const NOTIFICATIONS: [&'static str; 10] = [
        // This event only triggers with an app handle and always passes window handle to the callback.
        kAXWindowCreatedNotification,
        // TODO: does it pass a window or app handle?
        // This event only triggers with an app handle. It's triggered when window focus changes within
        // an app if the previously focused window was in the same app.
        kAXFocusedWindowChangedNotification,
        // This event only triggers with an app handle and always passes an app handle to the callback.
        // It's triggered when window focus changes to a new app.
        kAXApplicationActivatedNotification,
        // This event triggers with both app + window handles and always passes a window handle to the callback.
        kAXMovedNotification,
        // This event triggers with both app + window handles and always passes a window handle to the callback.
        kAXResizedNotification,
        // This event triggers with both app + window handles and always passes a window handle to the callback.
        kAXTitleChangedNotification,
        // TODO: the issue states these events happen too soon, to the point where the window is visible but not movable (yet)
        //       I'd like to do a little more experimentation on these events before moving to NSWorkspace notifications
        //       https://github.com/ianyh/Amethyst/issues/662
        // This event only triggers with an app handle and always passes an app handle to the callback.
        kAXApplicationShownNotification,
        // TODO: https://github.com/appium/appium-for-mac/blob/9e154e7de378374760344abd8572338535d6b7d8/Frameworks/PFAssistive.framework/Versions/J/Headers/PFUIElement.h#L412
        // This event only triggers with an app handle and always passes an app handle to the callback.
        kAXApplicationHiddenNotification,
        // This event triggers with both app + window handles and always passes a window handle to the callback.
        kAXWindowMiniaturizedNotification,
        // This event triggers with both app + window handles and always passes a window handle to the callback.
        kAXWindowDeminiaturizedNotification,
        // If registered using an app handle, the event will additionally trigger during circumstances like
        // changing tabs in Safari. It passes an invalid handle (neither app nor window handle) to the callback.
        // kAXUIElementDestroyedNotification, // NOTE: commented because it's manually handled
    ];

    pub fn new(
        app: &Application,
        sender: Sender<Result<WindowEvent, WindowError>>,
    ) -> Result<Watcher, WindowError> {
        let mut observer = MaybeUninit::uninit();
        let result = unsafe {
            AXObserver::create(
                app.pid,
                Some(app_notification),
                NonNull::new_unchecked(observer.as_mut_ptr()),
            )
        };
        match result {
            AXError::Success => {
                let observer =
                    unsafe { CFRetained::from_raw(NonNull::new_unchecked(observer.assume_init())) };
                let mut resources = Vec::new();

                // Since the destroyed notification doesn't include any information on the window, we must register
                // for each window with opaque data specifying the window being destroyed.
                let raw_windows = raw_windows(&app.inner)?;
                for window_handle in raw_windows.iter() {
                    unsafe {
                        // TODO: handle error?
                        window_handle.set_messaging_timeout(app.timeout.as_secs_f32());
                    }
                    let info = Box::into_raw(Box::new(CallbackInfo {
                        sender: sender.clone(),
                        notification: Notification::Destroyed(CFRetainedSafe(
                            window_handle.clone(),
                        )),
                    }));

                    Watcher::add_notification(
                        &CFString::from_static_str(kAXUIElementDestroyedNotification),
                        &observer,
                        &window_handle,
                        info,
                    )?;

                    resources.push(unsafe { Box::from_raw(info) })
                }

                for notification in Watcher::NOTIFICATIONS {
                    let info = Box::into_raw(Box::new(CallbackInfo {
                        sender: sender.clone(),
                        notification: match notification {
                            kAXWindowCreatedNotification => {
                                Notification::Created(app.inner.clone())
                            }
                            kAXFocusedWindowChangedNotification => {
                                Notification::Focused(app.inner.clone())
                            }
                            kAXApplicationActivatedNotification => {
                                Notification::Activated(app.inner.clone())
                            }
                            kAXMovedNotification => Notification::Moved(app.inner.clone()),
                            kAXResizedNotification => Notification::Resized(app.inner.clone()),
                            kAXTitleChangedNotification => Notification::Renamed(app.inner.clone()),
                            kAXApplicationShownNotification => {
                                Notification::Shown(app.inner.clone())
                            }
                            kAXApplicationHiddenNotification => {
                                Notification::Hidden(app.inner.clone())
                            }
                            kAXWindowMiniaturizedNotification => {
                                Notification::Miniaturized(app.inner.clone())
                            }
                            kAXWindowDeminiaturizedNotification => {
                                Notification::Deminiaturized(app.inner.clone())
                            }
                            _ => unreachable!(),
                        },
                    }));

                    Watcher::add_notification(
                        &CFString::from_static_str(notification),
                        &observer,
                        &app.inner,
                        info,
                    )?;

                    resources.push(unsafe { Box::from_raw(info) });
                }

                Ok(Watcher {
                    observer: CFRetainedSafe(observer),
                    resources,
                })
            }
            _ => Err(result.into()),
        }
    }

    pub(crate) fn run_on_thread(&self, thread_loop: &CFRunLoop) {
        unsafe {
            thread_loop.add_source(
                Some(&self.observer.run_loop_source()),
                kCFRunLoopDefaultMode,
            );
        }
    }

    fn add_notification(
        notification: &CFString,
        observer_handle: &AXObserver,
        window_handle: &AXUIElement,
        info: *mut CallbackInfo,
    ) -> Result<(), WindowError> {
        let result = unsafe {
            observer_handle.add_notification(window_handle, notification, info as *mut _)
        };
        match result {
            AXError::Success => {}
            // If the notification is unsupported, there's nothing we can do.
            AXError::NotificationUnsupported => {}
            _ => {
                return Err(result.into());
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Notification {
    Created(CFRetainedSafe<AXUIElement>),
    Destroyed(CFRetainedSafe<AXUIElement>),
    Focused(CFRetainedSafe<AXUIElement>),
    Activated(CFRetainedSafe<AXUIElement>),
    Moved(CFRetainedSafe<AXUIElement>),
    Resized(CFRetainedSafe<AXUIElement>),
    Renamed(CFRetainedSafe<AXUIElement>),
    Shown(CFRetainedSafe<AXUIElement>),
    Hidden(CFRetainedSafe<AXUIElement>),
    Miniaturized(CFRetainedSafe<AXUIElement>),
    Deminiaturized(CFRetainedSafe<AXUIElement>),
}

impl Notification {
    // TODO: kind of meh function that relies on the caller guaranteeing some constraints
    pub fn info(&self, window: Option<Window>) -> WindowEvent {
        let window = window.map(protocol::Window);
        match self {
            Notification::Created(_) => WindowEvent::Opened(window.unwrap()),
            Notification::Destroyed(window_handle) => {
                WindowEvent::Closed(protocol::WindowHandle(window_handle.clone()))
            }
            Notification::Focused(_) => WindowEvent::Focused(window.unwrap()),
            Notification::Activated(_) => WindowEvent::Focused(window.unwrap()),
            Notification::Moved(_) => WindowEvent::Moved(window.unwrap()),
            Notification::Resized(_) => WindowEvent::Resized(window.unwrap()),
            Notification::Renamed(_) => WindowEvent::Renamed(window.unwrap()),
            Notification::Shown(_) => WindowEvent::Shown(window.unwrap()),
            Notification::Hidden(_) => WindowEvent::Hidden(window.unwrap()),
            Notification::Miniaturized(_) => WindowEvent::Minimized(window.unwrap()),
            Notification::Deminiaturized(_) => WindowEvent::Unminimized(window.unwrap()),
        }
    }
}

#[derive(Debug)]
pub struct CallbackInfo {
    sender: Sender<Result<WindowEvent, WindowError>>,
    notification: Notification,
}

unsafe extern "C-unwind" fn app_notification(
    _observer: NonNull<AXObserver>,
    element: NonNull<AXUIElement>,
    notification: NonNull<CFString>,
    refcon: *mut raw::c_void,
) {
    // TODO: is this temporary for testing?
    let notification = unsafe { CFRetained::retain(notification) };
    println!("{:?}", notification.to_string());

    let callback_info = refcon as *mut CallbackInfo;
    let event = match &(*callback_info).notification {
        Notification::Created(app_handle)
        | Notification::Focused(app_handle)
        | Notification::Moved(app_handle)
        | Notification::Resized(app_handle)
        | Notification::Renamed(app_handle)
        | Notification::Miniaturized(app_handle)
        | Notification::Deminiaturized(app_handle) => {
            let window_handle = CFRetained::retain(element);

            let window = match Window::new(window_handle, app_handle.0.clone()) {
                Ok(window) => window,
                // TODO: error?
                // If we can't make a window, then we can't get its id, which means there's some fishy business going on...
                Err(_) => return,
            };

            (*callback_info).notification.info(Some(window))
        }
        Notification::Activated(app_handle) => {
            // TODO: lots of code dupe between above and from window module
            let mut window_handle = MaybeUninit::uninit();
            let result = unsafe {
                app_handle.copy_attribute_value(
                    &CFString::from_static_str(kAXFocusedWindowAttribute),
                    NonNull::new_unchecked(window_handle.as_mut_ptr()),
                )
            };
            if result == AXError::Success {
                let window_handle = CFRetained::from_raw(
                    NonNull::new_unchecked(&mut window_handle.assume_init()).cast::<AXUIElement>(),
                );
                let window = match Window::new(window_handle, app_handle.0.clone()) {
                    Ok(window) => window,
                    Err(_) => return,
                };
                (*callback_info).notification.info(Some(window))
            } else {
                // This could occur when an application has no windows but is focused (using cmd+tab).
                return;
            }
        }
        Notification::Shown(app_handle) | Notification::Hidden(app_handle) => {
            // TODO: we do a lot of error skipping here, reevaluate
            let mut pid = MaybeUninit::uninit();
            let result = unsafe { app_handle.pid(NonNull::new_unchecked(pid.as_mut_ptr())) };
            if result == AXError::Success {
                if let Ok(window_iter) = Application::new(pid.assume_init()).iter_windows() {
                    for window in window_iter.into_iter().flatten() {
                        let _ = (*callback_info)
                            .sender
                            .send(Ok((*callback_info).notification.info(Some(window))));
                    }
                }
            }
            return;
        }
        Notification::Destroyed(_) => (*callback_info).notification.info(None),
    };

    // It can only error if the sender is disconnected, and in that case, who cares.
    let _ = (*callback_info).sender.send(Ok(event));
}

pub(super) fn raw_windows(
    inner: &AXUIElement,
) -> Result<CFRetained<CFArray<AXUIElement>>, WindowError> {
    let mut windows = MaybeUninit::uninit();
    let result = unsafe {
        inner.copy_attribute_value(
            &CFString::from_str(kAXWindowsAttribute),
            NonNull::new_unchecked(windows.as_mut_ptr()),
        )
    };
    if result == AXError::Success {
        Ok(unsafe {
            CFRetained::cast_unchecked(CFRetained::from_raw(
                NonNull::new_unchecked(&mut windows.assume_init()).cast::<CFArray>(),
            ))
        })
    } else {
        Err(result.into())
    }
}
