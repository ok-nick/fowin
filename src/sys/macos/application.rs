use std::{mem::MaybeUninit, os::raw, sync::mpsc::Sender, time::Instant};

use crate::{
    protocol::{self, WindowError, WindowEvent, WindowEventInfo},
    sys::platform::ffi::{cfstring_to_string, CFArrayGetCount, CFArrayGetValueAtIndex, CFRetain},
    WindowId,
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
        CFRelease, CFRunLoopAddSource, CFRunLoopGetCurrent, CFStringRef, __AXObserver,
        __AXUIElement, kAXApplicationActivatedNotification, kAXErrorNotificationUnsupported,
    },
    window::{Window, _id},
};

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

    pub fn supported(&self) {
        // TODO: return a list of notifications that are able to be registered by this app
    }

    pub fn watch(
        self,
        sender: Sender<Result<WindowEvent, WindowError>>,
    ) -> Result<Watcher, WindowError> {
        Watcher::new(self, sender)
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
    // Resourecs are implicitly dropped after observer, so it's safe.
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
        app: Application,
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
                    let info = Box::into_raw(Box::new(CallbackInfo {
                        sender: sender.clone(),
                        notification: Notification::Destroyed(_id(&window_handle)?),
                    }));

                    Watcher::add_notification(
                        kAXUIElementDestroyedNotification,
                        observer.0,
                        window_handle.0,
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

                unsafe {
                    CFRunLoopAddSource(
                        CFRunLoopGetCurrent(),
                        AXObserverGetRunLoopSource(observer.0),
                        kCFRunLoopDefaultMode, // TODO: test using common modes, see if it's more responsive
                    );
                }

                Ok(Watcher {
                    observer,
                    resources,
                })
            }
            _ => Err(WindowError::from_ax_error(result)),
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
            // TODO: this occurs when trying to subcribe to an unsubscriptable process (common case)
            // as well as other obscure issues. yabai solves this by having a blacklist of known unsubscriptable
            // processes.. doesn't seem too reliable. I wonder how often this error occurs?
            // https://github.com/koekeishiya/yabai/issues/439
            // https://github.com/koekeishiya/yabai/blob/60380a1f18ebaa503fda29a72647fd8f5f5ce43b/src/process_manager.c#L14-L61
            // kAXErrorCannotComplete => {}
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
    Destroyed(WindowId),
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
    pub fn info(&self, window: Option<Window>) -> WindowEventInfo {
        let window = window.map(protocol::Window);
        match self {
            Notification::Created(_) => WindowEventInfo::Opened(window.unwrap()),
            Notification::Destroyed(id) => WindowEventInfo::Closed(*id),
            Notification::Focused(_) => WindowEventInfo::Focused(window.unwrap()),
            Notification::Activated(_) => WindowEventInfo::Focused(window.unwrap()),
            Notification::Moved(_) => WindowEventInfo::Moved(window.unwrap()),
            Notification::Resized(_) => WindowEventInfo::Resized(window.unwrap()),
            Notification::Renamed(_) => WindowEventInfo::Renamed(window.unwrap()),
            Notification::Shown(_) => WindowEventInfo::Shown(window.unwrap()),
            Notification::Hidden(_) => WindowEventInfo::Hidden(window.unwrap()),
            Notification::Miniaturized(_) => WindowEventInfo::Shown(window.unwrap()),
            Notification::Deminiaturized(_) => WindowEventInfo::Hidden(window.unwrap()),
        }
    }
}

#[derive(Debug)]
pub struct CallbackInfo {
    sender: Sender<Result<WindowEvent, WindowError>>,
    notification: Notification,
}

// TODO: try AXObserverCreateWithInfoCallback and iterate the info param to see if there's any useful info
// TODO: also for some reaosn this isn't being called
unsafe extern "C" fn app_notification(
    _observer: *mut __AXObserver,
    element: *const __AXUIElement,
    notification: CFStringRef,
    refcon: *mut raw::c_void,
) {
    let timestamp = Instant::now();

    // TODO: temp
    unsafe {
        CFRetain(notification as *const _);
    }
    println!("{:?}", cfstring_to_string(notification));

    let callback_info = refcon as *mut CallbackInfo;
    let info = match &(*callback_info).notification {
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
        Notification::Shown(app_handle)
        | Notification::Hidden(app_handle)
        | Notification::Activated(app_handle) => {
            // TODO: find which window(s) to send event to
            // TODO: for activated, test if we focus the app (using cmd+tab) and the app has no available windows,
            //       I think element will == application, otherwise the focused window??? test it
            todo!()
        }
        Notification::Destroyed(_) => (*callback_info).notification.info(None),
    };

    // It can only error if the sender is disconnected, and in that case, who cares.
    let _ = (*callback_info)
        .sender
        .send(Ok(WindowEvent::with_timestamp(info, timestamp)));
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
