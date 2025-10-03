use std::{
    mem::MaybeUninit,
    ptr::NonNull,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use libc::pid_t;
use objc2::{
    msg_send, msg_send_id,
    rc::{Id, Retained},
    ClassType,
};
use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
use objc2_application_services::{AXError, AXUIElement, AXValue, AXValueType};
use objc2_core_foundation::{
    kCFBooleanFalse, kCFBooleanTrue, CFBoolean, CFRetained, CFString, CFType, CGPoint, CGSize, Type,
};

use crate::{
    protocol::{Position, Size, WindowError},
    sys::platform::{application, ffi::CFArrayGetCount},
};

use super::{
    ffi::{
        cfstring_from_str, cfstring_to_string, kAXErrorSuccess, kAXFullScreenAttribute,
        kAXMinimizedAttribute, kAXPositionAttribute, kAXRaiseAction, kAXSizeAttribute,
        kAXTitleAttribute, AXUIElementCopyAttributeValue, AXUIElementPerformAction, AXUIElementRef,
        AXUIElementSetAttributeValue, AXValueGetValue, AXValueRef, CFBooleanGetValue, CFBooleanRef,
        CFRelease, CFStringRef, CFTypeRef, _AXUIElementGetWindow, __AXUIElement,
        kAXFocusedWindowAttribute, kAXFrontmostAttribute, kAXValueTypeCGPoint, AXUIElementGetPid,
        CFArrayGetValueAtIndex, CGWindowID,
    },
    WindowHandle,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    // NOTE:
    // * The ref is a pointer in another process (Carbon) or identifiers (Cocoa): https://lists.apple.com/archives/accessibility-dev/2013/Jun/msg00042.html
    // * Safe to use between threads, but will block anyways: https://lists.apple.com/archives/accessibility-dev/2012/Dec/msg00025.html
    inner: CFRetained<AXUIElement>,
    // TODO: now begs the question, can an AXUIElementRef for an application spontaneously change? Do we
    // need to validate this as well? If that's the case, then only store the application PID and we can
    // recreate the AXUIElementRef.
    app_handle: CFRetained<AXUIElement>,
}

// TODO: create a trait for this
// TODO: reduce boilerplate between some of these methods
impl Window {
    // TODO: add timeouts like for applications
    pub(super) fn new(
        inner: CFRetained<AXUIElement>,
        app_handle: CFRetained<AXUIElement>,
    ) -> Result<Window, WindowError> {
        Ok(Window { inner, app_handle })
    }

    pub fn handle(&self) -> WindowHandle {
        self.inner.clone()
    }

    pub fn title(&self) -> Result<String, WindowError> {
        let title = Self::value_for_attribute::<CFString>(
            &self.inner,
            &CFString::from_static_str(kAXTitleAttribute),
        )?;
        Ok(title.to_string())
    }

    // TODO: this returns the window size + title bar size.
    //       should we return this? return only the content size?
    //       make resize() include the title bar size? or just
    //       document it?
    pub fn size(&self) -> Result<Size, WindowError> {
        let frame: CGSize = Self::value_for_ax_value(
            &self.inner,
            &CFString::from_static_str(kAXSizeAttribute),
            AXValueType::CGSize,
        )?;
        Ok(Size {
            width: frame.width,
            height: frame.height,
        })
    }

    // TODO: for some reason this keeps returning Err(InvalidInternalArgument)...
    pub fn position(&self) -> Result<Position, WindowError> {
        let frame: CGPoint = Self::value_for_ax_value(
            &self.inner,
            &CFString::from_static_str(kAXPositionAttribute),
            AXValueType::CGPoint,
        )?;
        Ok(Position {
            x: frame.x,
            y: frame.y,
        })
    }

    pub fn is_focused(&self) -> Result<bool, WindowError> {
        // First check if the application is frontmost (AKA activated AKA application is focused).
        let frontmost = Self::bool_for_attribute(
            &self.app_handle,
            &CFString::from_static_str(kAXFrontmostAttribute),
        )?;
        if frontmost {
            let window = Self::value_for_attribute::<AXUIElement>(
                &self.app_handle,
                &CFString::from_static_str(kAXFocusedWindowAttribute),
            )?;

            // This tells us that this window was the last focused window within the application's windows.
            Ok(*window == *self.inner)
        } else {
            Ok(false)
        }
    }

    // TODO: when fullscreening then unfullscreening a window it still returns true
    pub fn is_fullscreen(&self) -> Result<bool, WindowError> {
        Self::bool_for_attribute(
            &self.inner,
            &CFString::from_static_str(kAXFullScreenAttribute),
        )
    }

    pub fn is_minimized(&self) -> Result<bool, WindowError> {
        Self::bool_for_attribute(
            &self.inner,
            &CFString::from_static_str(kAXMinimizedAttribute),
        )
    }

    #[inline]
    pub fn is_hidden(&self) -> Result<bool, WindowError> {
        // Default behavior of Window::hide is to minimize, so check if minimized.
        self.is_minimized()
    }

    // TODO: this sets the inner window wsize (excluding title bar)
    pub fn resize(&self, size: Size) -> Result<(), WindowError> {
        Self::set_value_for_attribute(
            &self.inner,
            &CFString::from_static_str(kAXSizeAttribute),
            // TODO: fixed in next release?
            // https://github.com/madsmtm/objc2/issues/777#issuecomment-3193276002
            unsafe {
                &*((&CGSize::new(size.width, size.height) as *const CGSize).cast::<CFType>())
            },
        )
    }

    pub fn reposition(&self, position: Position) -> Result<(), WindowError> {
        Self::set_value_for_attribute(
            &self.inner,
            &CFString::from_static_str(kAXPositionAttribute),
            // TODO: fixed in next release?
            // https://github.com/madsmtm/objc2/issues/777#issuecomment-3193276002
            unsafe {
                &*((&CGPoint::new(position.x, position.y) as *const CGPoint).cast::<CFType>())
            },
        )
    }

    pub fn focus(&self) -> Result<(), WindowError> {
        self.bring_to_front()?;

        // TODO: what about setting kAXFrontmostAttribute?
        unsafe {
            let app: Retained<NSRunningApplication> = msg_send![
                NSRunningApplication::class(),
                runningApplicationWithProcessIdentifier: self.pid()?
            ];
            // TODO: supposedly this option is deprecated, but it does provide the behavior we want, TEST IT
            //       this method also returns a bool signifying if the app has quit or if it can be activated
            app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
        }
        todo!()
    }

    pub fn fullscreen(&self) -> Result<(), WindowError> {
        Self::set_value_for_attribute(
            &self.inner,
            &CFString::from_static_str(kAXFullScreenAttribute),
            unsafe { &kCFBooleanTrue.unwrap() },
        )
    }

    pub fn unfullscreen(&self) -> Result<(), WindowError> {
        Self::set_value_for_attribute(
            &self.inner,
            &CFString::from_static_str(kAXFullScreenAttribute),
            unsafe { &kCFBooleanFalse.unwrap() },
        )
    }

    // bordered fullscreen AKA make window size of screen
    pub fn maximize(&self) -> Result<(), WindowError> {
        // TODO: calls move and resize, but how should we decide which display to do it for? add param?
        todo!()
    }

    pub fn minimize(&self) -> Result<(), WindowError> {
        Self::set_value_for_attribute(
            &self.inner,
            &CFString::from_static_str(kAXMinimizedAttribute),
            unsafe { &kCFBooleanTrue.unwrap() },
        )
    }

    pub fn unminimize(&self) -> Result<(), WindowError> {
        Self::set_value_for_attribute(
            &self.inner,
            &CFString::from_static_str(kAXMinimizedAttribute),
            unsafe { &kCFBooleanFalse.unwrap() },
        )
    }

    // TODO: if the application is hidden, then show the application and hide other windows besides this one
    pub fn show(&self) -> Result<(), WindowError> {
        self.unminimize()
    }

    // TODO: hide this window, minimizing is the best bet, can I set hidden attribute?
    pub fn hide(&self) -> Result<(), WindowError> {
        self.minimize()
    }

    pub fn bring_to_front(&self) -> Result<(), WindowError> {
        let result = unsafe {
            self.inner
                .perform_action(&CFString::from_static_str(kAXRaiseAction))
        };
        if result == AXError::Success {
            Ok(())
        } else {
            Err(result.into())
        }
    }

    fn pid(&self) -> Result<pid_t, WindowError> {
        let mut pid: MaybeUninit<pid_t> = MaybeUninit::uninit();
        let result = unsafe {
            self.app_handle
                .pid(NonNull::new_unchecked(pid.as_mut_ptr()))
        };
        if result == AXError::Success {
            Ok(unsafe { pid.assume_init() })
        } else {
            Err(result.into())
        }
    }

    fn set_value_for_attribute(
        handle: &AXUIElement,
        attribute: &CFString,
        value: &CFType,
    ) -> Result<(), WindowError> {
        let result = unsafe { handle.set_attribute_value(attribute, value) };
        if result == AXError::Success {
            Ok(())
        } else {
            Err(result.into())
        }
    }

    fn value_for_attribute<T: Type>(
        handle: &AXUIElement,
        attribute: &CFString,
    ) -> Result<CFRetained<T>, WindowError> {
        let mut value = MaybeUninit::uninit();
        let result = unsafe {
            handle.copy_attribute_value(attribute, NonNull::new_unchecked(value.as_mut_ptr()))
        };
        if result == AXError::Success {
            let value = unsafe {
                CFRetained::from_raw(NonNull::new_unchecked(&mut value.assume_init()).cast::<T>())
            };
            Ok(value)
        } else {
            Err(result.into())
        }
    }

    fn value_for_ax_value<T>(
        handle: &AXUIElement,
        attribute: &CFString,
        value_type: AXValueType,
    ) -> Result<T, WindowError> {
        let ax_value = Self::value_for_attribute::<AXValue>(handle, attribute)?;

        let mut frame: MaybeUninit<T> = MaybeUninit::zeroed();
        let result = unsafe {
            ax_value.value(
                value_type,
                NonNull::new_unchecked(frame.as_mut_ptr() as *mut _),
            )
        };

        if result {
            Ok(unsafe { frame.assume_init() })
        } else {
            Err(WindowError::InvalidInternalArgument)
        }
    }

    fn bool_for_attribute(handle: &AXUIElement, attribute: &CFString) -> Result<bool, WindowError> {
        let cf_boolean: MaybeUninit<CFType> = MaybeUninit::uninit();
        let result = unsafe {
            handle
                .copy_attribute_value(&attribute, NonNull::new_unchecked(&mut cf_boolean.as_ptr()))
        };
        if result == AXError::Success {
            let value = unsafe {
                let cf_boolean = CFRetained::from_raw(
                    NonNull::new_unchecked(&mut cf_boolean.assume_init()).cast::<CFBoolean>(),
                );
                cf_boolean.value()
            };
            Ok(value)
        } else {
            Err(result.into())
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
