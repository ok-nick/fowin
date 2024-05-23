use std::{mem::MaybeUninit, os::raw, sync::mpsc::Sender, time::Duration};

use crate::{
    protocol::{self, WindowError, WindowEvent},
    sys::platform::ffi::{
        cfstring_to_string, kAXFocusedWindowAttribute, AXUIElementGetPid, CFArrayGetCount,
        CFArrayGetValueAtIndex, CFHash, CFRetain,
    },
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
        CFRelease, CFRunLoopAddSource, CFStringRef, __AXObserver, __AXUIElement,
        kAXApplicationActivatedNotification, kAXErrorNotificationUnsupported,
        AXUIElementSetMessagingTimeout, _AXUIElementGetWindow, kAXErrorCannotComplete, CGWindowID,
        __CFRunLoopSource,
    },
    window::Window,
};

const DEFAULT_AX_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub struct Application {
    inner: AXUIElementRef,
    pid: pid_t,
    timeout: Duration,
}

impl Application {
    pub fn new(pid: pid_t) -> Application {
        Application::with_timeout(pid, DEFAULT_AX_TIMEOUT)
    }

    // TODO: timeouts should be exposed to the user
    pub fn with_timeout(pid: pid_t, timeout: Duration) -> Application {
        let inner = AXUIElementRef(unsafe { AXUIElementCreateApplication(pid) });
        unsafe {
            AXUIElementSetMessagingTimeout(inner.0, timeout.as_secs_f32());
        }

        Application {
            inner,
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
        let len = unsafe { CFArrayGetCount(raw_windows) };
        Ok(WindowIterator {
            inner: raw_windows,
            app_handle: self.inner.clone(),
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
            _AXUIElementGetWindow(self.inner.0, _id.as_mut_ptr() as *mut _)
                == kAXErrorCannotComplete
        }
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
    // Resources are implicitly dropped after observer, so it's safe.
    observer: AXObserverRef,
    resources: Vec<Box<CallbackInfo>>,
}

impl Watcher {
    // Some additional information about these events:
    // https://github.com/appium/appium-for-mac/blob/9e154e7de378374760344abd8572338535d6b7d8/Frameworks/PFAssistive.framework/Versions/J/Headers/PFUIElement.h#L961-L994
    const NOTIFICATIONS: [&str; 10] = [
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
        let result = unsafe { AXObserverCreate(app.pid, app_notification, observer.as_mut_ptr()) };
        match result {
            kAXErrorSuccess => {
                let observer = AXObserverRef(unsafe { observer.assume_init() });

                let mut resources = Vec::new();

                // Since the destroyed notification doesn't include any information on the window, we must register
                // for each window with opaque data specifying the window being destroyed.
                let raw_windows = raw_windows(&app.inner)?;
                let len = unsafe { CFArrayGetCount(raw_windows) };
                for i in 0..len {
                    let window_handle = AXUIElementRef(unsafe {
                        CFArrayGetValueAtIndex(raw_windows, i) as *const _
                    });
                    let raw_handle = window_handle.0;
                    unsafe {
                        AXUIElementSetMessagingTimeout(raw_handle, app.timeout.as_secs_f32());
                    }
                    let info = Box::into_raw(Box::new(CallbackInfo {
                        sender: sender.clone(),
                        notification: Notification::Destroyed(window_handle),
                    }));

                    Watcher::add_notification(
                        kAXUIElementDestroyedNotification,
                        observer.0,
                        raw_handle,
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

                    Watcher::add_notification(notification, observer.0, app.inner.0, info)?;

                    resources.push(unsafe { Box::from_raw(info) });
                }

                Ok(Watcher {
                    observer,
                    resources,
                })
            }
            _ => Err(WindowError::from_ax_error(result)),
        }
    }

    pub(crate) fn run_on_thread(&self, thread_loop: *mut __CFRunLoopSource) {
        unsafe {
            CFRunLoopAddSource(
                thread_loop,
                AXObserverGetRunLoopSource(self.observer.0),
                kCFRunLoopDefaultMode,
            );
        }
    }

    fn add_notification(
        notification: &'static str,
        observer_handle: *mut __AXObserver,
        window_handle: *const __AXUIElement,
        info: *mut CallbackInfo,
    ) -> Result<(), WindowError> {
        let result = unsafe {
            AXObserverAddNotification(
                observer_handle,
                window_handle,
                cfstring_from_str(notification),
                info as *mut _,
            )
        };
        match result {
            kAXErrorSuccess => {}
            // If the notification is unsupported, there's nothing we can do.
            kAXErrorNotificationUnsupported => {}
            _ => {
                return Err(WindowError::from_ax_error(result));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Notification {
    Created(AXUIElementRef),
    Destroyed(AXUIElementRef),
    Focused(AXUIElementRef),
    Activated(AXUIElementRef),
    Moved(AXUIElementRef),
    Resized(AXUIElementRef),
    Renamed(AXUIElementRef),
    Shown(AXUIElementRef),
    Hidden(AXUIElementRef),
    Miniaturized(AXUIElementRef),
    Deminiaturized(AXUIElementRef),
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
            Notification::Miniaturized(_) => WindowEvent::Shown(window.unwrap()),
            Notification::Deminiaturized(_) => WindowEvent::Hidden(window.unwrap()),
        }
    }
}

#[derive(Debug)]
pub struct CallbackInfo {
    sender: Sender<Result<WindowEvent, WindowError>>,
    notification: Notification,
}

unsafe extern "C" fn app_notification(
    _observer: *mut __AXObserver,
    element: *const __AXUIElement,
    notification: CFStringRef,
    refcon: *mut raw::c_void,
) {
    unsafe {
        CFRetain(notification as *const _);
    }
    println!("{:?}", cfstring_to_string(notification));

    let callback_info = refcon as *mut CallbackInfo;
    let event = match &(*callback_info).notification {
        Notification::Created(app_handle)
        | Notification::Focused(app_handle)
        | Notification::Moved(app_handle)
        | Notification::Resized(app_handle)
        | Notification::Renamed(app_handle)
        | Notification::Miniaturized(app_handle)
        | Notification::Deminiaturized(app_handle) => {
            let window_handle = AXUIElementRef(element);
            window_handle.increment_ref_count();

            let window = match Window::new(window_handle, app_handle.clone()) {
                Ok(window) => window,
                // TODO: error?
                // If we can't make a window, then we can't get its id, which means there's some fishy business going on...
                Err(_) => return,
            };

            (*callback_info).notification.info(Some(window))
        }
        Notification::Activated(app_handle) => {
            // TODO: lots of code dupe between above and from window module
            let mut window: MaybeUninit<*const __AXUIElement> = MaybeUninit::uninit();
            let result = unsafe {
                AXUIElementCopyAttributeValue(
                    app_handle.0,
                    cfstring_from_str(kAXFocusedWindowAttribute),
                    window.as_mut_ptr() as *mut _,
                )
            };
            if result == kAXErrorSuccess {
                let window_handle = AXUIElementRef(unsafe { window.assume_init() });
                let window = match Window::new(window_handle, app_handle.clone()) {
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
            let mut pid: MaybeUninit<pid_t> = MaybeUninit::uninit();
            let result = unsafe { AXUIElementGetPid(app_handle.0, pid.as_mut_ptr()) };
            if result == kAXErrorSuccess {
                let pid = unsafe { pid.assume_init() };
                if let Ok(window_iter) = Application::new(pid).iter_windows() {
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
