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
    protocol::{Position, Size, WindowError},
    sys::platform::{application, ffi::CFArrayGetCount},
};

use super::{
    ffi::{
        cfstring_from_str, cfstring_to_string, kAXErrorSuccess, kAXFullScreenAttribute,
        kAXMinimizedAttribute, kAXPositionAttribute, kAXRaiseAction, kAXSizeAttribute,
        kAXTitleAttribute, kAXValueTypeCGSize, kCFBooleanFalse, kCFBooleanTrue,
        AXUIElementCopyAttributeValue, AXUIElementPerformAction, AXUIElementRef,
        AXUIElementSetAttributeValue, AXValueGetValue, AXValueRef, CFBooleanGetValue, CFBooleanRef,
        CFRelease, CFStringRef, CFTypeRef, _AXUIElementGetWindow, __AXUIElement,
        kAXFocusedWindowAttribute, kAXFrontmostAttribute, kAXValueTypeCGPoint, pid_t,
        AXUIElementGetPid, CFArrayGetValueAtIndex, CGWindowID,
        NSRunningApplication_processIdentifier,
    },
    WindowHandle,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    // NOTE:
    // * The ref is a pointer in another process (Carbon) or identifiers (Cocoa): https://lists.apple.com/archives/accessibility-dev/2013/Jun/msg00042.html
    // * Safe to use between threads, but will block anyways: https://lists.apple.com/archives/accessibility-dev/2012/Dec/msg00025.html
    inner: AXUIElementRef,
    // TODO: now begs the question, can an AXUIElementRef for an application spontaneously change? Do we
    // need to validate this as well? If that's the case, then only store the application PID and we can
    // recreate the AXUIElementRef.
    app_handle: AXUIElementRef,
}

// TODO: create a trait for this
// TODO: reduce boilerplate between some of these methods
impl Window {
    // TODO: add timeouts like for applications
    pub(super) fn new(
        inner: AXUIElementRef,
        app_handle: AXUIElementRef,
    ) -> Result<Window, WindowError> {
        Ok(Window { inner, app_handle })
    }

    pub fn handle(&self) -> WindowHandle {
        self.inner.clone()
    }

    pub fn title(&self) -> Result<String, WindowError> {
        let mut title: MaybeUninit<CFStringRef> = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.inner.0,
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
                self.inner.0,
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
                self.inner.0,
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
                    Ok(window == self.inner)
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
                self.inner.0,
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
                self.inner.0,
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
                self.inner.0,
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
                self.inner.0,
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
                self.inner.0,
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
                self.inner.0,
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
                self.inner.0,
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
                self.inner.0,
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
        let result =
            unsafe { AXUIElementPerformAction(self.inner.0, cfstring_from_str(kAXRaiseAction)) };
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
}

// TODO: there is no reason to use this private API. An AXUIElementRef is a unique handle.
//       Although, it's important to note that ids on Carbon may not be unique, in contrast to Cocoa. Carbon AXUIElementRef handles
//       may change at any time (unpredictably), and I previously used its id to confirm that per operation. There is still no
//       guarantee we are able to identify if the window has changed as we do not know the guarantees of this private API and it is highly likely
//       that ids are reused. Anyways, carbon has been deprecated for a long time and only supports 32 bit apps, so not a big market
// NOTE: this operation is pretty quick ~60 microseconds
//       interesting notes from yabai about ids: https://github.com/koekeishiya/yabai/blob/edb34504d1caa7bfa33a97ff46f3570b9f2f7e3d/src/window_manager.c#L1438
// pub(super) fn _id(inner: &AXUIElementRef) -> Result<WindowId, WindowError> {
//     let mut id: MaybeUninit<CGWindowID> = MaybeUninit::zeroed();
//     let result = unsafe { _AXUIElementGetWindow(inner.0, id.as_mut_ptr()) };
//     if result == kAXErrorSuccess {
//         Ok(unsafe { id.assume_init() })
//     } else {
//         // As this is a private API, there is no formal specification for which errors may be returned,
//         // but we can take a good guess.
//         Err(WindowError::from_ax_error(result))
//     }
// }
