#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unused)]

// TODO: eventual goal is to replace everything here with icrate once they get everything sorted out
// inspiration: https://github.com/dimusic/active-win-pos-rs/tree/main
// and: https://github.com/eiz/accessibility/tree/master/accessibility-sys
// https://github.com/wusyong/carbon-bindgen/blob/467fca5d71047050b632fbdfb41b1f14575a8499/bindings.rs

use std::{
    ffi::{c_char, CStr, CString},
    mem,
};

use icrate::objc2::msg_send;

pub const kAXErrorSuccess: _bindgen_ty_1571 = 0;

pub const kAXFocusedUIElementAttribute: &str = "AXFocusedUIElement";
pub const kAXWindowAttribute: &str = "AXWindow";
pub const kAXWindowsAttribute: &str = "AXWindows";
pub const kAXMinimizedAttribute: &str = "AXMinimized";
pub const kAXSizeAttribute: &str = "AXSize";
pub const kAXPositionAttribute: &str = "AXPosition";
pub const kAXTitleAttribute: &str = "AXTitle";
pub const kAXFocusedWindowAttribute: &str = "AXFocusedWindow";
pub const kAXCloseButtonAttribute: &str = "AXCloseButton";
pub const kAXFullScreenButtonAttribute: &str = "AXFullScreenButton";
pub const kAXFullScreenAttribute: &str = "AXFullScreen";
pub const kAXHiddenAttribute: &str = "AXHidden";
pub const kAXRaiseAction: &str = "AXRaise";

pub const kAXWindowCreatedNotification: &str = "AXWindowCreated";
pub const kAXUIElementDestroyedNotification: &str = "AXUIElementDestroyed";
pub const kAXWindowMiniaturizedNotification: &str = "AXWindowMiniaturized";
pub const kAXWindowDeminiaturizedNotification: &str = "AXWindowDeminiaturized";
pub const kAXFocusedWindowChangedNotification: &str = "AXFocusedWindowChanged";
pub const kAXMovedNotification: &str = "AXMoved";
pub const kAXTitleChangedNotification: &str = "AXTitleChanged";

pub type _bindgen_ty_1571 = ::std::os::raw::c_int;
pub type UInt8 = ::std::os::raw::c_uchar;
pub type __int32_t = ::std::os::raw::c_int;
pub type SInt32 = ::std::os::raw::c_int;
pub type UInt32 = ::std::os::raw::c_uint;
pub type Boolean = ::std::os::raw::c_uchar;
pub type _bindgen_ty_15 = ::std::os::raw::c_uint;
pub type _bindgen_ty_1575 = ::std::os::raw::c_uint;
pub type CGFloat = f64;
pub type CFBooleanRef = *const __CFBoolean;

pub type AXValueRef = *const __AXValue;
pub type AXValueType = UInt32;
pub type __darwin_pid_t = __int32_t;
pub type pid_t = __darwin_pid_t;
pub type CFStringRef = *const __CFString;
pub type CFTypeRef = *const ::std::os::raw::c_void;
pub type CFAllocatorRef = *const __CFAllocator;
pub type CFStringEncoding = UInt32;
pub type CFIndex = ::std::os::raw::c_long;
pub const kCFStringEncodingUTF8: _bindgen_ty_15 = 134217984;
pub type CFDictionaryRef = *const __CFDictionary;
pub const kAXValueTypeCGSize: _bindgen_ty_1575 = 2;
pub const kAXValueTypeCGPoint: _bindgen_ty_1575 = 1;
pub type CFArrayRef = *const __CFArray;

pub type AXUIElementRef = *const __AXUIElement;
pub type AXObserverRef = *mut __AXObserver;
pub type AXObserverCallbackWithInfo = ::std::option::Option<
    unsafe extern "C" fn(
        observer: AXObserverRef,
        element: AXUIElementRef,
        notification: CFStringRef,
        info: CFDictionaryRef,
        refcon: *mut ::std::os::raw::c_void,
    ),
>;
pub type AXObserverCallback = ::std::option::Option<
    unsafe extern "C" fn(
        observer: AXObserverRef,
        element: AXUIElementRef,
        notification: CFStringRef,
        refcon: *mut ::std::os::raw::c_void,
    ),
>;
pub type AXError = SInt32;
pub type CFRunLoopSourceRef = *mut __CFRunLoopSource;
pub type CFRunLoopRef = *mut __CFRunLoop;
pub type CFRunLoopMode = CFStringRef;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __CFArray {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __CFBoolean {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __AXValue {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __CFDictionary {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __CFRunLoop {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __CFAllocator {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __CFString {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __AXUIElement {
    _unused: [u8; 0],
}

// TODO: accessibilty objects cannot be shared or sent across threads (enable negative impl feature])
// impl !Send for __AXUIElement {}
// impl !Syncfor __AXUIElement {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __AXObserver {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __CFRunLoopSource {
    _unused: [u8; 0],
}

// #[link(name = "Carbon", kind = "framework")]
extern "C" {
    pub static kCFAllocatorDefault: CFAllocatorRef;
    pub static kCFRunLoopDefaultMode: CFRunLoopMode;
    pub static kCFBooleanTrue: CFBooleanRef;
    pub static kCFBooleanFalse: CFBooleanRef;

    pub fn AXUIElementCreateApplication(pid: pid_t) -> AXUIElementRef;

    pub fn AXObserverCreate(
        application: pid_t,
        callback: AXObserverCallback,
        outObserver: *mut AXObserverRef,
    ) -> AXError;

    pub fn AXObserverAddNotification(
        observer: AXObserverRef,
        element: AXUIElementRef,
        notification: CFStringRef,
        refcon: *mut ::std::os::raw::c_void,
    ) -> AXError;

    pub fn AXObserverGetRunLoopSource(observer: AXObserverRef) -> CFRunLoopSourceRef;

    pub fn CFStringCreateWithBytes(
        alloc: CFAllocatorRef,
        bytes: *const UInt8,
        numBytes: CFIndex,
        encoding: CFStringEncoding,
        isExternalRepresentation: Boolean,
    ) -> CFStringRef;

    pub fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);

    pub fn CFRunLoopGetMain() -> CFRunLoopRef;

    pub fn AXObserverRemoveNotification(
        observer: AXObserverRef,
        element: AXUIElementRef,
        notification: CFStringRef,
    ) -> AXError;

    pub fn CFRunLoopSourceInvalidate(source: CFRunLoopSourceRef);

    pub fn CFRelease(cf: CFTypeRef);

    pub fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> AXError;

    pub fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;

    pub fn CFEqual(cf1: CFTypeRef, cf2: CFTypeRef) -> Boolean;

    pub fn AXObserverCreateWithInfoCallback(
        application: pid_t,
        callback: AXObserverCallbackWithInfo,
        outObserver: *mut AXObserverRef,
    ) -> AXError;

    // PRIVATE API
    pub fn AXUIElementGetWindow(element: AXUIElementRef, identifier: *mut u32) -> i32;

    pub fn CFStringGetLength(theString: CFStringRef) -> CFIndex;

    pub fn CFStringGetCStringPtr(
        theString: CFStringRef,
        encoding: CFStringEncoding,
    ) -> *const ::std::os::raw::c_char;

    pub fn CFStringGetCString(
        theString: CFStringRef,
        buffer: *mut ::std::os::raw::c_char,
        bufferSize: CFIndex,
        encoding: CFStringEncoding,
    ) -> Boolean;

    pub fn AXValueGetValue(
        value: AXValueRef,
        theType: AXValueType,
        valuePtr: *mut ::std::os::raw::c_void,
    ) -> Boolean;

    pub fn CFBooleanGetValue(boolean: CFBooleanRef) -> Boolean;

    pub fn AXUIElementPerformAction(element: AXUIElementRef, action: CFStringRef) -> AXError;

    pub fn CFArrayGetCount(theArray: CFArrayRef) -> CFIndex;

    pub fn CFArrayGetValueAtIndex(
        theArray: CFArrayRef,
        idx: CFIndex,
    ) -> *const ::std::os::raw::c_void;

    pub fn CFRetain(cf: CFTypeRef) -> CFTypeRef;
}

// TODO: verify correctness
pub unsafe fn NSRunningApplication_processIdentifier(
    app: &icrate::AppKit::NSRunningApplication,
) -> pid_t {
    msg_send![app, processIdentifier]
}

// TODO: need to CFRelease
// should cache all the cfstring constants
pub unsafe fn cfstring_from_str(str: &str) -> CFStringRef {
    // TODO: CFStringCreateWithBytesNoCopy
    CFStringCreateWithBytes(
        kCFAllocatorDefault,
        str.as_ptr(),
        str.len() as CFIndex, // TODO: constrain
        kCFStringEncodingUTF8,
        false as Boolean,
    )
}

// TODO: can use this but it's not guaranteed to work
// https://developer.apple.com/documentation/corefoundation/1542133-cfstringgetcstringptr
// fn cfstring_to_str<'a>(cfstring: CFStringRef) -> Option<&'a str> {
//     let cstr_ptr = unsafe { CFStringGetCStringPtr(cfstring, 0) };

//     if !cstr_ptr.is_null() {
//         let length = unsafe { CFStringGetLength(cfstring) };
//         // should be valid UTF-8
//         Some(unsafe { CStr::from_ptr(cstr_ptr).to_str().unwrap() })
//     } else {
//         None
//     }
// }

// NOTE: this will release the string for you
pub fn cfstring_to_string(cfstring: CFStringRef) -> Option<String> {
    unsafe {
        let length = CFStringGetLength(cfstring) + 1;
        // TODO: error if length > usize
        let mut buffer: Vec<c_char> = Vec::with_capacity(length as usize);

        if CFStringGetCString(cfstring, buffer.as_mut_ptr(), length, kCFStringEncodingUTF8) != 0 {
            CFRelease(cfstring as *const _);

            Some(
                CString::from_raw(buffer.as_mut_ptr())
                    .to_string_lossy()
                    .into_owned(),
            )
        } else {
            None
        }
    }
}

pub fn cfarray_to_vec<T>(cfarray: CFArrayRef) -> Vec<T> {
    let len = unsafe { CFArrayGetCount(cfarray) };
    let mut vec = Vec::with_capacity(len as usize);
    for i in 0..len {
        let element = unsafe { CFArrayGetValueAtIndex(cfarray, i) };
        vec.push(element);
    }

    unsafe {
        CFRelease(cfarray as *const _);

        // the most diabolical unsafeness ever
        mem::transmute(vec)
    }
}
