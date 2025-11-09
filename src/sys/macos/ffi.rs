#![allow(non_upper_case_globals)]

use std::ops::Deref;

use objc2_application_services::AXUIElement;
use objc2_core_foundation::{CFRetained, Type};

pub const kAXFrontmostAttribute: &str = "AXFrontmost";
pub const kAXWindowsAttribute: &str = "AXWindows";
pub const kAXMinimizedAttribute: &str = "AXMinimized";
pub const kAXSizeAttribute: &str = "AXSize";
pub const kAXPositionAttribute: &str = "AXPosition";
pub const kAXTitleAttribute: &str = "AXTitle";
pub const kAXFocusedWindowAttribute: &str = "AXFocusedWindow";
pub const kAXFullScreenAttribute: &str = "AXFullScreen";
pub const kAXRaiseAction: &str = "AXRaise";

pub const kAXApplicationActivatedNotification: &str = "AXApplicationActivated";
pub const kAXResizedNotification: &str = "AXResized";
pub const kAXApplicationHiddenNotification: &str = "AXApplicationHidden";
pub const kAXApplicationShownNotification: &str = "AXApplicationShown";
pub const kAXWindowCreatedNotification: &str = "AXWindowCreated";
pub const kAXUIElementDestroyedNotification: &str = "AXUIElementDestroyed";
pub const kAXWindowMiniaturizedNotification: &str = "AXWindowMiniaturized";
pub const kAXWindowDeminiaturizedNotification: &str = "AXWindowDeminiaturized";
pub const kAXFocusedWindowChangedNotification: &str = "AXFocusedWindowChanged";
pub const kAXMovedNotification: &str = "AXMoved";
pub const kAXTitleChangedNotification: &str = "AXTitleChanged";

pub type CGWindowID = u32;

// TODO: AXUIElementRefs can be compared for equality using CFEqual, impl Eq for Window as well
//       https://lists.apple.com/archives/accessibility-dev/2006/Jun/msg00010.html
//       https://github.com/appium/appium-for-mac/blob/9e154e7de378374760344abd8572338535d6b7d8/Frameworks/PFAssistive.framework/Versions/J/Headers/PFUIElement.h#L305

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct CFRetainedSafe<T: Type>(pub CFRetained<T>);

impl<T: Type> Clone for CFRetainedSafe<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

unsafe impl<T: Type> Send for CFRetainedSafe<T> {}

impl<T: Type> Deref for CFRetainedSafe<T> {
    type Target = CFRetained<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    // PRIVATE API
    pub fn _AXUIElementGetWindow(element: &AXUIElement, identifier: *mut CGWindowID) -> i32;

}
