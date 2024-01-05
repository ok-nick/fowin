use std::{
    mem::MaybeUninit,
    ptr,
    sync::{Arc, RwLock, RwLockReadGuard},
    time::Instant,
};

use icrate::Foundation::{CGPoint, CGSize};

use crate::{
    protocol::{Position, Size, WindowError, WindowId},
    sys::platform::{
        application,
        ffi::{
            kCGWindowName, kCGWindowNumber, kCGWindowOwnerName, CFArrayGetCount,
            CFDictionaryGetValue, CFNumberGetValue, CFNumberRef, CFStringGetLength, CGWindowID,
            __AXUIElement,
        },
    },
};

use super::ffi::{
    self, cfstring_from_str, cfstring_to_string, kAXErrorIllegalArgument, kAXErrorNoValue,
    kAXErrorSuccess, kAXFullScreenAttribute, kAXMinimizedAttribute, kAXPositionAttribute,
    kAXRaiseAction, kAXSizeAttribute, kAXTitleAttribute, kAXValueTypeCGSize, kCFBooleanFalse,
    kCFBooleanTrue, AXUIElementCopyAttributeValue, AXUIElementPerformAction, AXUIElementRef,
    AXUIElementSetAttributeValue, AXValueGetValue, AXValueRef, CFBooleanGetValue, CFBooleanRef,
    CFRelease, CFRetain, CFStringRef, CFTypeRef, _AXUIElementGetWindow, kAXValueTypeCGPoint,
    kCFAllocatorDefault, CFArrayCreate, CFArrayGetValueAtIndex, CFArrayRef, CFDictionaryRef,
    CGWindowListCopyWindowInfo, CGWindowListCreateDescriptionFromArray,
};

// NOTE: this is safe to pass between threads (although perhaps not safe to query between threads?)
//       TLDR; it's a pointer for another process (or an ID depending on the backend framework)
//       https://lists.apple.com/archives/accessibility-dev/2013/Jun/msg00042.html
// NOTE: according to the URL below, it may be safe to use on different threads as long as it's only
//       being used by one thread at a time
//       https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/ThreadSafetySummary/ThreadSafetySummary.html
#[derive(Debug)]
pub struct Window {
    // Wrapped in an RwLock so that we can easily revalidate the inner ref, if need be.
    inner: RwLock<AXUIElementRef>,
    // TODO: now begs the question, can an AXUIElementRef for an application spontaneously change? Do we
    // need to validate this as well? If that's the case, then only store the application PID and we can
    // recreate the AXUIElementRef.
    app_inner: AXUIElementRef,
    id: WindowId,
}

// TODO: create a trait for this
// TODO: reduce boilerplate between some of these methods
impl Window {
    pub(super) fn new(
        inner: AXUIElementRef,
        app_inner: AXUIElementRef,
    ) -> Result<Window, WindowError> {
        Ok(Window {
            id: _id(&inner)?,
            inner: RwLock::new(inner),
            app_inner,
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
        match self.inner.read() {
            // if the handle is outdated, try revalidating, otherwise it's good
            Ok(inner) => match self.id == _id(&inner)? {
                true => Ok(true),
                false => self.try_revalidate(),
            },
            // the lock is poisoned, fix it
            Err(_) => self.try_revalidate(),
        }
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

    // TODO: can I do this here? it would be most logical..
    // pub fn focused(&self) -> u32 {
    //     todo!()
    // }

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
                    value as AXValueRef, // TODO: sure this works?
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
                    value as AXValueRef, // TODO: sure this works?
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

    pub fn fullscreened(&self) -> Result<bool, WindowError> {
        let mut fullscreened: MaybeUninit<CFTypeRef> = MaybeUninit::zeroed();
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
        let mut hidden: MaybeUninit<CFTypeRef> = MaybeUninit::zeroed();
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

    fn read_inner(&self) -> Result<RwLockReadGuard<AXUIElementRef>, WindowError> {
        if self.exists()? {
            Ok(self.inner.read().unwrap())
        } else {
            Err(WindowError::InvalidHandle)
        }
    }

    // Attempt to revalidate the underlying window handle, if it still exists. If a handle was found,
    // return true, otherwise false.
    fn try_revalidate(&self) -> Result<bool, WindowError> {
        let raw_windows = application::raw_windows(&self.app_inner)?;
        let len = unsafe { CFArrayGetCount(raw_windows) };
        for i in 0..len {
            let window =
                AXUIElementRef(unsafe { CFArrayGetValueAtIndex(raw_windows, i) as *const _ });

            if _id(&window)? == self.id {
                let poisioned = self.inner.is_poisoned();

                let mut inner = self.inner.write().unwrap_or_else(|mut inner| {
                    **inner.get_mut() = window.clone();
                    self.inner.clear_poison();
                    inner.into_inner()
                });

                if !poisioned {
                    *inner = window.clone();
                }
                return Ok(true);
            }
        }

        Ok(false)
    }
}

// NOTE: this operation is pretty quick ~60 microseconds
// TODO: interesting notes from yabai about ids: https://github.com/koekeishiya/yabai/blob/edb34504d1caa7bfa33a97ff46f3570b9f2f7e3d/src/window_manager.c#L1438
fn _id(inner: &AXUIElementRef) -> Result<WindowId, WindowError> {
    let mut id = MaybeUninit::zeroed();
    let result = unsafe { _AXUIElementGetWindow(inner.0, id.as_mut_ptr()) };
    if result == kAXErrorSuccess {
        Ok(unsafe { id.assume_init() })
    } else {
        // As this is a private API, there is no formal specification for what errors may be returned,
        // but we can take a good guess.
        Err(WindowError::from_ax_error(result))
    }
}
