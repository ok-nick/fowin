use std::{
    mem::MaybeUninit,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use icrate::{
    objc2::{msg_send_id, rc::Id, ClassType},
    AppKit::{NSApplicationActivateIgnoringOtherApps, NSRunningApplication, NSWorkspace},
    Foundation::{CGPoint, CGSize},
};

use crate::{
    protocol::{Position, Size, WindowError, WindowId},
    sys::platform::{application, ffi::CFArrayGetCount},
};

use super::ffi::{
    cfstring_from_str, cfstring_to_string, kAXErrorSuccess, kAXFullScreenAttribute,
    kAXMinimizedAttribute, kAXPositionAttribute, kAXRaiseAction, kAXSizeAttribute,
    kAXTitleAttribute, kAXValueTypeCGSize, kCFBooleanFalse, kCFBooleanTrue,
    AXUIElementCopyAttributeValue, AXUIElementPerformAction, AXUIElementRef,
    AXUIElementSetAttributeValue, AXValueGetValue, AXValueRef, CFBooleanGetValue, CFBooleanRef,
    CFRelease, CFStringRef, CFTypeRef, _AXUIElementGetWindow, __AXUIElement,
    kAXFocusedWindowAttribute, kAXFrontmostAttribute, kAXValueTypeCGPoint, pid_t,
    AXUIElementGetPid, CFArrayGetValueAtIndex, CGWindowID, NSRunningApplication_processIdentifier,
};

// TODO: I believe we can do CFEqual on the AXUIElementRef rather than id comparisons
#[derive(Debug, Clone)]
pub struct Window {
    // NOTE:
    // * The ref is a pointer in another process (Carbon) or identifiers (Cocoa): https://lists.apple.com/archives/accessibility-dev/2013/Jun/msg00042.html
    // * Safe to use between threads, but will block anyways: https://lists.apple.com/archives/accessibility-dev/2012/Dec/msg00025.html
    // Wrapped in an RwLock so that we can easily revalidate the inner ref, if need be.
    inner: Arc<RwLock<AXUIElementRef>>,
    // TODO: now begs the question, can an AXUIElementRef for an application spontaneously change? Do we
    // need to validate this as well? If that's the case, then only store the application PID and we can
    // recreate the AXUIElementRef.
    app_handle: AXUIElementRef,
    id: WindowId,
}

// TODO: create a trait for this
// TODO: reduce boilerplate between some of these methods
impl Window {
    // TODO: add timeouts like for applications
    pub(super) fn new(
        inner: AXUIElementRef,
        app_handle: AXUIElementRef,
    ) -> Result<Window, WindowError> {
        Ok(Window {
            id: _id(&inner)?,
            inner: Arc::new(RwLock::new(inner)),
            app_handle,
        })
    }

    pub fn id(&self) -> Result<WindowId, WindowError> {
        _id(&*self.read_inner()?)
    }

    // An AXUIElementRef is a handle containing a (not-unique) identifier to an underlying window. Due to
    // unpredictable circumstances, the identifier may point to a different window. This function will
    // compare the cached window id to what the OS thinks it is, to verify whether the window changed. In
    // the event that it did change, this function will attempt to revalidate the handle by finding the new
    // handle corresponding to the cached id.
    // source: https://lists.apple.com/archives/accessibility-dev/2013/Jun/msg00045.html
    pub fn exists(&self) -> Result<bool, WindowError> {
        // Scope the guard so that the read lock drops before we attempt acquire a write lock in try_revalidate.
        {
            let guard = self.inner.read();
            if let Ok(inner) = guard {
                if self.id == _id(&inner)? {
                    return Ok(true);
                }
            }
        }

        // If the lock is poisoned or the handle has been recycled, fix it.
        self.try_revalidate()
    }

    pub fn title(&self) -> Result<String, WindowError> {
        let mut title: MaybeUninit<CFStringRef> = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.read_inner()?.0,
                cfstring_from_str(kAXTitleAttribute),
                title.as_mut_ptr() as *mut _,
            )
        };
        if result == kAXErrorSuccess {
            cfstring_to_string(unsafe { title.assume_init() })
                // TODO: different error type? it says it errors if the "conversion fails" or the buffer is too small. I believe with these paramters it should always succeed
                .ok_or(WindowError::InvalidInternalArgument)
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn size(&self) -> Result<Size, WindowError> {
        let mut size: MaybeUninit<CFTypeRef> = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.read_inner()?.0,
                cfstring_from_str(kAXSizeAttribute),
                size.as_mut_ptr() as *mut _,
            )
        };
        if result == kAXErrorSuccess {
            let mut frame: MaybeUninit<CGSize> = MaybeUninit::zeroed();
            let result = unsafe {
                let value = size.assume_init();
                let result = AXValueGetValue(
                    value as AXValueRef,
                    kAXValueTypeCGSize,
                    frame.as_mut_ptr() as *mut _,
                );
                CFRelease(value);
                result
            };

            if result != 0 {
                let frame = unsafe { frame.assume_init() };
                Ok(Size {
                    width: frame.width,
                    height: frame.height,
                })
            } else {
                Err(WindowError::InvalidInternalArgument)
            }
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    // TODO: for some reason this keeps returning Err(InvalidInternalArgument)...
    pub fn position(&self) -> Result<Position, WindowError> {
        let mut position: MaybeUninit<CFTypeRef> = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.read_inner()?.0,
                cfstring_from_str(kAXPositionAttribute),
                position.as_mut_ptr() as *mut _,
            )
        };
        if result == kAXErrorSuccess {
            let mut frame: MaybeUninit<CGPoint> = MaybeUninit::zeroed();
            let result = unsafe {
                let value = position.assume_init();
                AXValueGetValue(
                    value as AXValueRef,
                    kAXValueTypeCGPoint,
                    frame.as_mut_ptr() as *mut _,
                );
                CFRelease(value);
                result
            };

            if result != 0 {
                let frame = unsafe { frame.assume_init() };
                Ok(Position {
                    x: frame.x,
                    y: frame.y,
                })
            } else {
                Err(WindowError::InvalidInternalArgument)
            }
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn focused(&self) -> Result<bool, WindowError> {
        // First check if the application is frontmost (AKA activated AKA application is focused).
        let mut frontmost: MaybeUninit<CFTypeRef> = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.app_handle.0,
                cfstring_from_str(kAXFrontmostAttribute),
                frontmost.as_mut_ptr() as *mut _,
            )
        };
        if result == kAXErrorSuccess {
            let frontmost = unsafe {
                let value = frontmost.assume_init();
                let frontmost = CFBooleanGetValue(value as CFBooleanRef);
                CFRelease(value);
                frontmost
            };
            if frontmost != 0 {
                let mut window: MaybeUninit<*const __AXUIElement> = MaybeUninit::uninit();
                let result = unsafe {
                    AXUIElementCopyAttributeValue(
                        self.app_handle.0,
                        cfstring_from_str(kAXFocusedWindowAttribute),
                        window.as_mut_ptr() as *mut _,
                    )
                };
                if result == kAXErrorSuccess {
                    let window = AXUIElementRef(unsafe { window.assume_init() });
                    // This tells us that this window was the last focused window within the application's windows.
                    Ok(_id(&window)? == self.id()?)
                } else {
                    Err(WindowError::from_ax_error(result))
                }
            } else {
                Ok(false)
            }
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn fullscreened(&self) -> Result<bool, WindowError> {
        let mut fullscreened: MaybeUninit<CFTypeRef> = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.read_inner()?.0,
                cfstring_from_str(kAXFullScreenAttribute),
                fullscreened.as_mut_ptr() as *mut _,
            )
        };
        if result == kAXErrorSuccess {
            let fullscreened = unsafe {
                let value = fullscreened.assume_init();
                let fullscreened = CFBooleanGetValue(value as CFBooleanRef);
                CFRelease(value);
                fullscreened
            };
            Ok(fullscreened != 0)
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn minimized(&self) -> Result<bool, WindowError> {
        let mut hidden: MaybeUninit<CFTypeRef> = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.read_inner()?.0,
                cfstring_from_str(kAXMinimizedAttribute),
                hidden.as_mut_ptr() as *mut _,
            )
        };
        if result == kAXErrorSuccess {
            let hidden = unsafe {
                let value = hidden.assume_init();
                let hidden = CFBooleanGetValue(value as CFBooleanRef);
                CFRelease(value);
                hidden
            };
            Ok(hidden != 0)
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn visible(&self) -> Result<bool, WindowError> {
        // TODO: returns if this window is visible, by means of app->hidden? and window->minimized
        // also check if window size > 0?
        // another thing to take into consideration is if the display is off or if it's on a visible (macos) space
        todo!()
    }

    pub fn resize(&self, size: Size) -> Result<(), WindowError> {
        let result = unsafe {
            AXUIElementSetAttributeValue(
                self.read_inner()?.0,
                cfstring_from_str(kAXSizeAttribute),
                &CGSize::new(size.width, size.height) as *const _ as *const _,
            )
        };
        if result == kAXErrorSuccess {
            Ok(())
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn translate(&self, position: Position) -> Result<(), WindowError> {
        let result = unsafe {
            AXUIElementSetAttributeValue(
                self.read_inner()?.0,
                cfstring_from_str(kAXPositionAttribute),
                &CGPoint::new(position.x, position.y) as *const _ as *const _,
            )
        };
        if result == kAXErrorSuccess {
            Ok(())
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn focus(&self) -> Result<(), WindowError> {
        self.bring_to_front()?;

        // TODO: what about setting kAXFrontmostAttribute?
        unsafe {
            let app: Id<NSRunningApplication> = msg_send_id![
                NSRunningApplication::class(),
                runningApplicationWithProcessIdentifier: self.pid()?
            ];
            // TODO: supposedly this option is deprecated, but it does provide the behavior we want, TEST IT
            //       this method also returns a bool signifying if the app has quit or if it can be activated
            app.activateWithOptions(NSApplicationActivateIgnoringOtherApps);
        }
        todo!()
    }

    pub fn fullscreen(&self) -> Result<(), WindowError> {
        let result = unsafe {
            AXUIElementSetAttributeValue(
                self.read_inner()?.0,
                cfstring_from_str(kAXFullScreenAttribute),
                &kCFBooleanTrue as *const _ as *const _,
            )
        };
        if result == kAXErrorSuccess {
            Ok(())
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn unfullscreen(&self) -> Result<(), WindowError> {
        let result = unsafe {
            AXUIElementSetAttributeValue(
                self.read_inner()?.0,
                cfstring_from_str(kAXFullScreenAttribute),
                &kCFBooleanFalse as *const _ as *const _,
            )
        };
        if result == kAXErrorSuccess {
            Ok(())
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    // bordered fullscreen AKA make window size of screen
    pub fn maximize(&self) -> Result<(), WindowError> {
        // TODO: calls move and resize, but how should we decide which display to do it for? add param?
        todo!()
    }

    // TODO: this is a WINDOW handling library, not an application handling
    //       if the application is hidden, then show the application and hide other windows besides this one
    pub fn show(&self) -> Result<(), WindowError> {
        let result = unsafe {
            AXUIElementSetAttributeValue(
                self.read_inner()?.0,
                cfstring_from_str(kAXMinimizedAttribute),
                &kCFBooleanFalse as *const _ as *const _,
            )
        };
        if result == kAXErrorSuccess {
            Ok(())
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn hide(&self) -> Result<(), WindowError> {
        // TODO: hide this window, minimizing is the best bet
        let result = unsafe {
            AXUIElementSetAttributeValue(
                self.read_inner()?.0,
                cfstring_from_str(kAXMinimizedAttribute),
                &kCFBooleanTrue as *const _ as *const _,
            )
        };
        if result == kAXErrorSuccess {
            Ok(())
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn bring_to_front(&self) -> Result<(), WindowError> {
        let result = unsafe {
            AXUIElementPerformAction(self.read_inner()?.0, cfstring_from_str(kAXRaiseAction))
        };
        if result == kAXErrorSuccess {
            Ok(())
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    fn pid(&self) -> Result<pid_t, WindowError> {
        let mut pid: MaybeUninit<pid_t> = MaybeUninit::uninit();
        let result = unsafe { AXUIElementGetPid(self.app_handle.0, pid.as_mut_ptr()) };
        if result == kAXErrorSuccess {
            Ok(unsafe { pid.assume_init() })
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    fn read_inner(&self) -> Result<RwLockReadGuard<AXUIElementRef>, WindowError> {
        if self.exists()? {
            // exists() is called on the same thread, so if the thread was poisoned, it would never reach here anyways, therefore the unwrap is safe
            Ok(self.inner.read().unwrap())
        } else {
            Err(WindowError::InvalidHandle)
        }
    }

    // Attempt to revalidate the underlying window handle, if it still exists. If a handle was found,
    // return true, otherwise false.
    fn try_revalidate(&self) -> Result<bool, WindowError> {
        let inner = self.inner.write();
        // If the lock isn't poisoned, then check if it's valid.
        if let Ok(inner) = &inner {
            // If the handle was validated while we were waiting to obtain the lock, then no work needs to be done, it's valid.
            if self.id == _id(inner)? {
                return Ok(true);
            }
        }

        let raw_windows = application::raw_windows(&self.app_handle)?;
        let len = unsafe { CFArrayGetCount(raw_windows) };
        for i in 0..len {
            let window =
                AXUIElementRef(unsafe { CFArrayGetValueAtIndex(raw_windows, i) as *const _ });

            // TODO: I can also use CFEqual on the AXUIElementRef pointers iirc, read more @ ffi::AXUIElementRef
            if _id(&window)? == self.id {
                match inner {
                    Ok(mut inner) => {
                        *inner = window.clone();
                    }
                    Err(mut err) => {
                        **err.get_mut() = window.clone();
                        self.inner.clear_poison();
                    }
                }

                return Ok(true);
            }
        }

        Ok(false)
    }
}

// NOTE: this operation is pretty quick ~60 microseconds
// TODO: interesting notes from yabai about ids: https://github.com/koekeishiya/yabai/blob/edb34504d1caa7bfa33a97ff46f3570b9f2f7e3d/src/window_manager.c#L1438
pub(super) fn _id(inner: &AXUIElementRef) -> Result<WindowId, WindowError> {
    let mut id: MaybeUninit<CGWindowID> = MaybeUninit::zeroed();
    let result = unsafe { _AXUIElementGetWindow(inner.0, id.as_mut_ptr()) };
    if result == kAXErrorSuccess {
        Ok(unsafe { id.assume_init() })
    } else {
        // As this is a private API, there is no formal specification for which errors may be returned,
        // but we can take a good guess.
        Err(WindowError::from_ax_error(result))
    }
}
