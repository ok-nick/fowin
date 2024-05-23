use ffi::{CGDisplayPixelsHigh, CGDisplayPixelsWide};

use crate::{LogicalSize, PhysicalPosition, PhysicalSize};

use self::ffi::{
    CGDirectDisplayID, CGDisplayCopyDisplayMode, CGDisplayModeGetPixelHeight,
    CGDisplayModeGetPixelWidth, CGDisplayModeGetRefreshRate, CGDisplayModeRef, CGDisplayRotation,
    CGMainDisplayID,
};

type ScreenHandle = CGDirectDisplayID;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screen {
    inner: CGDirectDisplayID,
}

impl Screen {
    pub fn handle(&self) -> ScreenHandle {
        self.inner
    }

    pub fn name(&self) -> String {
        // TODO: CGDisplayModelNumber + CGDisplaySerialNumber or alternatives (maybe NSScreen?)
        todo!()
    }

    // Return value is in degrees.
    pub fn rotation(&self) -> f64 {
        // TODO: returns 0 if display is invalid
        unsafe { CGDisplayRotation(self.inner) }
    }

    // TODO: is this reliably calculable?
    pub fn scale_factor(&self) -> f32 {
        todo!()
    }

    // TODO: https://github.com/rust-windowing/winit/blob/3e8fa410735cfb1486b6f7fd636c810fffd6c268/src/platform_impl/macos/monitor.rs#L214
    // Return value is in hertz.
    pub fn refresh_rate(&self) -> f64 {
        // TODO: can return 0 for "unconventional" displays, how should I handle
        unsafe { CGDisplayModeGetRefreshRate(self.mode()) }
    }

    pub fn is_primary(&self) -> bool {
        self.inner == unsafe { CGMainDisplayID() }
    }

    pub fn modes(&self) {
        todo!()
    }

    // NOTE: interesting info about position/sizes
    // source: https://github.com/rust-windowing/winit/issues/2645
    // source: https://github.com/tauri-apps/tao/issues/816

    // NOTE: CGDisplayPixelsHigh and CGDisplayPixelsWide are old and use points
    // source: https://github.com/lionheart/openradar-mirror/issues/18671
    // TODO: does CGDisplayBounds return pixels or points?
    pub fn physical_size(&self) -> PhysicalSize {
        unsafe {
            let mode = self.mode();
            PhysicalSize {
                width: CGDisplayModeGetPixelWidth(mode) as u64,
                height: CGDisplayModeGetPixelHeight(mode) as u64,
            }
        }
    }

    fn mode(&self) -> CGDisplayModeRef {
        // TODO: returns null if display is invalid, also must release via CGDisplayModeRelease
        unsafe { CGDisplayCopyDisplayMode(self.inner) }
    }

    pub fn logical_size(&self) -> LogicalSize {
        unsafe {
            LogicalSize {
                // TODO: is there a more accurate method of obtaining logical size?
                width: CGDisplayPixelsWide(self.inner) as f32,
                height: CGDisplayPixelsHigh(self.inner) as f32,
            }
        }
    }

    // TODO: test NSScreen.frame and CGDisplayBounds for physical or logical
    // it's noted that CGDisplayBounds has (0, 0) for top-left corner of main display
    // there exists also NSScreen.convertRectToBacking, I wonder if it works for screen coordinates (rather than window coords)?
    pub fn physical_position(&self) -> PhysicalPosition {
        todo!()
    }

    // TODO: physical position of a monitor is more useful than logical position
    // and if physical position exists, there is no need for logical position
    // this is because a logical position doesn't tell us much, it can be anything depending
    // on the scale factor and resolution of the other monitors
    // pub fn logical_position(&self) -> LogicalPosition {
    //     todo!()
    // }
}

// TODO: also provide events for displays, such as power on/off, etc.

// https://raw.githubusercontent.com/wusyong/carbon-bindgen/467fca5d71047050b632fbdfb41b1f14575a8499/bindings.rs
mod ffi {
    pub type CGDirectDisplayID = u32;
    pub type CGDisplayModeRef = CGDisplayMode;
    pub type CFDictionaryRef = *const __CFDictionary;
    pub type CFArrayRef = *const __CFArray;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct CGDisplayMode {
        _unused: [u8; 0],
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct __CFDictionary {
        _unused: [u8; 0],
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct __CFArray {
        _unused: [u8; 0],
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        pub fn CGDisplayPixelsWide(display: CGDirectDisplayID) -> usize;
        pub fn CGDisplayPixelsHigh(display: CGDirectDisplayID) -> usize;
        pub fn CGDisplayCopyDisplayMode(display: CGDirectDisplayID) -> CGDisplayModeRef;
        pub fn CGDisplayModeGetPixelWidth(mode: CGDisplayModeRef) -> usize;
        pub fn CGDisplayModeGetPixelHeight(mode: CGDisplayModeRef) -> usize;
        pub fn CGDisplayModeGetRefreshRate(mode: CGDisplayModeRef) -> f64;
        pub fn CGMainDisplayID() -> CGDirectDisplayID;
        pub fn CGDisplayRotation(display: CGDirectDisplayID) -> f64;
        pub fn CGDisplayModeRelease(mode: CGDisplayModeRef);
        pub fn CGDisplayCopyAllDisplayModes(
            display: CGDirectDisplayID,
            options: CFDictionaryRef,
        ) -> CFArrayRef;
    }
}
