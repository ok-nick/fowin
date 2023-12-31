use std::mem::MaybeUninit;

use icrate::Foundation::{CGPoint, CGSize};

use crate::protocol::{Position, Size, WindowId};

use super::ffi::{
    cfstring_from_str, cfstring_to_string, kAXErrorSuccess, kAXFullScreenAttribute,
    kAXMinimizedAttribute, kAXPositionAttribute, kAXRaiseAction, kAXSizeAttribute,
    kAXTitleAttribute, kAXValueTypeCGSize, kCFBooleanFalse, kCFBooleanTrue,
    AXUIElementCopyAttributeValue, AXUIElementGetWindow, AXUIElementPerformAction, AXUIElementRef,
    AXUIElementSetAttributeValue, AXValueGetValue, AXValueRef, CFBooleanGetValue, CFBooleanRef,
    CFRelease, CFStringRef, CFTypeRef,
};

#[derive(Debug)]
pub struct Window {
    // TODO: this is reference counted, impl clone for axuielementref that internally increases the ref count
    inner: AXUIElementRef,
}

// TODO: create a trait for this
// TODO: reduce boilerplate between some of these methods
impl Window {
    pub fn new(inner: AXUIElementRef) -> Window {
        Window { inner }
    }

    pub fn id(&self) -> Result<WindowId, ()> {
        let mut id = MaybeUninit::zeroed();
        let result = unsafe { AXUIElementGetWindow(self.inner, id.as_mut_ptr()) };
        if result == kAXErrorSuccess {
            Ok(unsafe { id.assume_init() })
        } else {
            Err(())
        }
    }

    pub fn title(&self) -> Result<String, ()> {
        let mut title: MaybeUninit<CFStringRef> = MaybeUninit::uninit();
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.inner,
                cfstring_from_str(kAXTitleAttribute),
                title.as_mut_ptr() as *mut _,
            )
        };
        if result == kAXErrorSuccess {
            cfstring_to_string(unsafe { title.assume_init() }).ok_or(())
        } else {
            Err(())
        }
    }

    // TODO: can't do this here, needs to be done globally
    // pub fn focused(&self) -> u32 {
    //     todo!()
    // }

    pub fn size(&self) -> Result<Size, ()> {
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
                AXValueGetValue(
                    value as AXValueRef, // TODO: sure this works?
                    kAXValueTypeCGSize,
                    frame.as_mut_ptr() as *mut _,
                );
                CFRelease(value);
            }

            let frame = unsafe { frame.assume_init() };
            Ok(Size {
                width: frame.width,
                height: frame.height,
            })
        } else {
            Err(())
        }
    }

    pub fn position(&self) -> Result<Position, ()> {
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
            }

            let frame = unsafe { frame.assume_init() };
            Ok(Position {
                x: frame.x,
                y: frame.y,
            })
        } else {
            Err(())
        }
    }

    pub fn fullscreened(&self) -> Result<bool, ()> {
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
            Err(())
        }
    }

    pub fn minimized(&self) -> Result<bool, ()> {
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
            Err(())
        }
    }

    pub fn visible(&self) -> Result<bool, ()> {
        // TODO: returns if this window is visible, by means of app->hidden? and window->minimized
        // also check if window size > 0?
        todo!()
    }

    pub fn exists(&self) -> Result<bool, ()> {
        // TODO: returns if this window still exists, usually this is done by seeing if one of the attribute setting functions fail
        todo!()
    }

    pub fn resize(&self, size: Size) -> Result<(), ()> {
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
            // TODO
            Err(())
        }
    }

    pub fn r#move(&mut self, position: Position) -> Result<(), ()> {
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
            // TODO
            Err(())
        }
    }

    pub fn fullscreen(&mut self) -> Result<(), ()> {
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
            // TODO
            Err(())
        }
    }

    pub fn unfullscreen(&mut self) -> Result<(), ()> {
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
            // TODO
            Err(())
        }
    }

    // aka fullscreen without hiding dock/menu bar
    pub fn fullscreen_windowed(&self) {
        // TODO: calls move and resize, it is the users choice to bring to front and focus
    }

    pub fn maximize(&self) -> Result<(), ()> {
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
            // TODO
            Err(())
        }
    }

    pub fn minimize(&self) -> Result<(), ()> {
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
            // TODO
            Err(())
        }
    }

    pub fn bring_to_front(&self) -> Result<(), ()> {
        let result =
            unsafe { AXUIElementPerformAction(self.inner, cfstring_from_str(kAXRaiseAction)) };
        if result == kAXErrorSuccess {
            Ok(())
        } else {
            // TODO
            Err(())
        }
    }
}
