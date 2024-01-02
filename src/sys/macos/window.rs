use std::mem::MaybeUninit;

use icrate::Foundation::{CGPoint, CGSize};

use crate::protocol::{Position, Size, WindowError, WindowId};

use super::ffi::{
    self, cfstring_from_str, cfstring_to_string, kAXErrorIllegalArgument, kAXErrorNoValue,
    kAXErrorSuccess, kAXFullScreenAttribute, kAXMinimizedAttribute, kAXPositionAttribute,
    kAXRaiseAction, kAXSizeAttribute, kAXTitleAttribute, kAXValueTypeCGSize, kCFBooleanFalse,
    kCFBooleanTrue, AXUIElementCopyAttributeValue, AXUIElementPerformAction, AXUIElementRef,
    AXUIElementSetAttributeValue, AXValueGetValue, AXValueRef, CFBooleanGetValue, CFBooleanRef,
    CFRelease, CFRetain, CFStringRef, CFTypeRef, _AXUIElementGetWindow,
};

// NOTE: this is safe to pass between threads (although perhaps not safe to query between threads?)
//       TLDR; it's a pointer for another process (or an ID depending on the backend framework)
//       https://lists.apple.com/archives/accessibility-dev/2013/Jun/msg00042.html
// NOTE: according to the URL below, it may be safe to use on different threads as long as it's only
//       being used by one thread at a time
//       https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/ThreadSafetySummary/ThreadSafetySummary.html
#[derive(Debug, Clone, Copy)]
pub struct Window {
    inner: AXUIElementRef,
}

// TODO: create a trait for this
// TODO: reduce boilerplate between some of these methods
impl Window {
    pub fn new(inner: AXUIElementRef) -> Window {
        Window { inner }
    }

    pub fn id(&self) -> Result<WindowId, WindowError> {
        let mut id = MaybeUninit::zeroed();
        let result = unsafe { _AXUIElementGetWindow(self.inner, id.as_mut_ptr()) };
        if result == kAXErrorSuccess {
            Ok(unsafe { id.assume_init() })
        } else {
            // as this is a private API, there is no formal specification for what errors may be returned
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn title(&self) -> Result<String, WindowError> {
        let mut title: MaybeUninit<CFStringRef> = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.inner,
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
                self.inner,
                cfstring_from_str(kAXSizeAttribute),
                size.as_mut_ptr() as *mut _,
            )
        };
        if result == kAXErrorSuccess {
            let mut frame: MaybeUninit<CGSize> = MaybeUninit::zeroed();
            unsafe {
                let value = size.assume_init();
                let result = AXValueGetValue(
                    value as AXValueRef, // TODO: sure this works?
                    kAXValueTypeCGSize,
                    frame.as_mut_ptr() as *mut _,
                );
                CFRelease(value);
                if result == 0 {
                    return Err(WindowError::InvalidInternalArgument);
                }
            }

            let frame = unsafe { frame.assume_init() };
            Ok(Size {
                width: frame.width,
                height: frame.height,
            })
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn position(&self) -> Result<Position, WindowError> {
        let mut position: MaybeUninit<CFTypeRef> = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.inner,
                cfstring_from_str(kAXPositionAttribute),
                position.as_mut_ptr() as *mut _,
            )
        };
        if result == kAXErrorSuccess {
            let mut frame: MaybeUninit<CGPoint> = MaybeUninit::zeroed();
            unsafe {
                let value = position.assume_init();
                AXValueGetValue(
                    value as AXValueRef, // TODO: sure this works?
                    kAXValueTypeCGSize,
                    frame.as_mut_ptr() as *mut _,
                );
                CFRelease(value);
                if result == 0 {
                    return Err(WindowError::InvalidInternalArgument);
                }
            }

            let frame = unsafe { frame.assume_init() };
            Ok(Position {
                x: frame.x,
                y: frame.y,
            })
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }

    pub fn fullscreened(&self) -> Result<bool, WindowError> {
        let mut fullscreened: MaybeUninit<CFTypeRef> = MaybeUninit::zeroed();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.inner,
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
                self.inner,
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

    pub fn exists(&self) -> Result<bool, WindowError> {
        // TODO: returns if this window still exists, usually this is done by seeing if one of the attribute setting functions fail
        todo!()
    }

    pub fn resize(&self, size: Size) -> Result<(), WindowError> {
        let result = unsafe {
            AXUIElementSetAttributeValue(
                self.inner,
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

    pub fn translate(&mut self, position: Position) -> Result<(), WindowError> {
        let result = unsafe {
            AXUIElementSetAttributeValue(
                self.inner,
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

    pub fn fullscreen(&mut self) -> Result<(), WindowError> {
        let result = unsafe {
            AXUIElementSetAttributeValue(
                self.inner,
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

    pub fn unfullscreen(&mut self) -> Result<(), WindowError> {
        let result = unsafe {
            AXUIElementSetAttributeValue(
                self.inner,
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
                self.inner,
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
                self.inner,
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
            unsafe { AXUIElementPerformAction(self.inner, cfstring_from_str(kAXRaiseAction)) };
        if result == kAXErrorSuccess {
            Ok(())
        } else {
            Err(WindowError::from_ax_error(result))
        }
    }
}
