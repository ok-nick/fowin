#![allow(non_camel_case_types)]
// TODO: thread info:
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

pub const kAXErrorSuccess: i32 = 0;
pub const kAXErrorFailure: i32 = -25200;
pub const kAXErrorIllegalArgument: i32 = -25201;
pub const kAXErrorInvalidUIElement: i32 = -25202;
pub const kAXErrorInvalidUIElementObserver: i32 = -25203;
pub const kAXErrorCannotComplete: i32 = -25204;
pub const kAXErrorAttributeUnsupported: i32 = -25205;
pub const kAXErrorActionUnsupported: i32 = -25206;
pub const kAXErrorNotificationUnsupported: i32 = -25207;
pub const kAXErrorNotImplemented: i32 = -25208;
pub const kAXErrorNotificationAlreadyRegistered: i32 = -25209;
pub const kAXErrorNotificationNotRegistered: i32 = -25210;
pub const kAXErrorAPIDisabled: i32 = -25211;
pub const kAXErrorNoValue: i32 = -25212;
pub const kAXErrorParameterizedAttributeUnsupported: i32 = -25213;
pub const kAXErrorNotEnoughPrecision: i32 = -25214;

pub const kAXDescriptionAttribute: &str = "AXDescription";
pub const kAXEnabledAttribute: &str = "AXEnabled";
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

pub const kAXFocusedUIElementChangedNotification: &str = "AXFocusedUIElementChanged";
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
pub type CFHashCode = ::std::os::raw::c_ulong;
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

pub type CFMutableDictionaryRef = *mut __CFDictionary;
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
pub type CFTimeInterval = f64;
pub type CFRunLoopRunResult = SInt32;
pub const kCFRunLoopRunFinished: SInt32 = 1;

// TODO: AXUIElementRefs can be compared for equality using CFEqual, impl Eq for Window as well
//       https://lists.apple.com/archives/accessibility-dev/2006/Jun/msg00010.html
//       https://github.com/appium/appium-for-mac/blob/9e154e7de378374760344abd8572338535d6b7d8/Frameworks/PFAssistive.framework/Versions/J/Headers/PFUIElement.h#L305
#[repr(transparent)]
#[derive(Debug)]
pub struct AXUIElementRef(pub *const __AXUIElement);
impl AXUIElementRef {
    pub fn increment_ref_count(&self) {
        unsafe {
            CFRetain(self.0 as *const _);
        }
    }
}
unsafe impl Sync for AXUIElementRef {}
unsafe impl Send for AXUIElementRef {}
impl Clone for AXUIElementRef {
    fn clone(&self) -> Self {
        self.increment_ref_count();
        AXUIElementRef(self.0)
    }
}
impl Drop for AXUIElementRef {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0 as *const _);
        }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct AXObserverRef(pub *mut __AXObserver);
unsafe impl Send for AXObserverRef {}
impl AXObserverRef {
    pub fn increment_ref_count(&self) {
        unsafe {
            CFRetain(self.0 as *const _);
        }
    }
}
impl Clone for AXObserverRef {
    fn clone(&self) -> Self {
        self.increment_ref_count();
        AXObserverRef(self.0)
    }
}
impl Drop for AXObserverRef {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0 as *const _ as *const _);
        }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct CFRunLoopSourceRef(pub *mut __CFRunLoopSource);
unsafe impl Send for CFRunLoopSourceRef {}
impl CFRunLoopSourceRef {
    pub fn increment_ref_count(&self) {
        unsafe {
            CFRetain(self.0 as *const _);
        }
    }
}
impl Clone for CFRunLoopSourceRef {
    fn clone(&self) -> Self {
        self.increment_ref_count();
        CFRunLoopSourceRef(self.0)
    }
}
impl Drop for CFRunLoopSourceRef {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0 as *const _ as *const _);
        }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct CFRunLoopRef(pub *mut __CFRunLoopSource);
unsafe impl Send for CFRunLoopRef {}
impl CFRunLoopRef {
    pub fn increment_ref_count(&self) {
        unsafe {
            CFRetain(self.0 as *const _);
        }
    }
}
impl Clone for CFRunLoopRef {
    fn clone(&self) -> Self {
        self.increment_ref_count();
        CFRunLoopRef(self.0)
    }
}
impl Drop for CFRunLoopRef {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0 as *const _ as *const _);
        }
    }
}

pub type AXObserverCallbackWithInfo = unsafe extern "C" fn(
    observer: *mut __AXObserver,
    element: *const __AXUIElement,
    notification: CFStringRef,
    info: CFDictionaryRef,
    refcon: *mut ::std::os::raw::c_void,
);

pub type AXObserverCallback = unsafe extern "C" fn(
    observer: *mut __AXObserver,
    element: *const __AXUIElement,
    notification: CFStringRef,
    refcon: *mut ::std::os::raw::c_void,
);
pub type AXError = SInt32;
pub type CFRunLoopMode = CFStringRef;

pub type CFDictionaryRetainCallBack = ::std::option::Option<
    unsafe extern "C" fn(
        allocator: CFAllocatorRef,
        value: *const ::std::os::raw::c_void,
    ) -> *const ::std::os::raw::c_void,
>;
pub type CFDictionaryReleaseCallBack = ::std::option::Option<
    unsafe extern "C" fn(allocator: CFAllocatorRef, value: *const ::std::os::raw::c_void),
>;
pub type CFDictionaryCopyDescriptionCallBack = ::std::option::Option<
    unsafe extern "C" fn(value: *const ::std::os::raw::c_void) -> CFStringRef,
>;
pub type CFDictionaryEqualCallBack = ::std::option::Option<
    unsafe extern "C" fn(
        value1: *const ::std::os::raw::c_void,
        value2: *const ::std::os::raw::c_void,
    ) -> Boolean,
>;
pub type CFDictionaryHashCallBack =
    ::std::option::Option<unsafe extern "C" fn(value: *const ::std::os::raw::c_void) -> CFHashCode>;

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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CFDictionaryKeyCallBacks {
    pub version: CFIndex,
    pub retain: CFDictionaryRetainCallBack,
    pub release: CFDictionaryReleaseCallBack,
    pub copyDescription: CFDictionaryCopyDescriptionCallBack,
    pub equal: CFDictionaryEqualCallBack,
    pub hash: CFDictionaryHashCallBack,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CFDictionaryValueCallBacks {
    pub version: CFIndex,
    pub retain: CFDictionaryRetainCallBack,
    pub release: CFDictionaryReleaseCallBack,
    pub copyDescription: CFDictionaryCopyDescriptionCallBack,
    pub equal: CFDictionaryEqualCallBack,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CFRunLoopSourceContext {
    pub version: CFIndex,
    pub info: *mut ::std::os::raw::c_void,
    pub retain: ::std::option::Option<
        unsafe extern "C" fn(info: *const ::std::os::raw::c_void) -> *const ::std::os::raw::c_void,
    >,
    pub release: ::std::option::Option<unsafe extern "C" fn(info: *const ::std::os::raw::c_void)>,
    pub copyDescription: ::std::option::Option<
        unsafe extern "C" fn(info: *const ::std::os::raw::c_void) -> CFStringRef,
    >,
    pub equal: ::std::option::Option<
        unsafe extern "C" fn(
            info1: *const ::std::os::raw::c_void,
            info2: *const ::std::os::raw::c_void,
        ) -> Boolean,
    >,
    pub hash: ::std::option::Option<
        unsafe extern "C" fn(info: *const ::std::os::raw::c_void) -> CFHashCode,
    >,
    pub schedule: ::std::option::Option<
        unsafe extern "C" fn(
            info: *mut ::std::os::raw::c_void,
            rl: *mut __CFRunLoopSource,
            mode: CFRunLoopMode,
        ),
    >,
    pub cancel: ::std::option::Option<
        unsafe extern "C" fn(
            info: *mut ::std::os::raw::c_void,
            rl: *mut __CFRunLoopSource,
            mode: CFRunLoopMode,
        ),
    >,
    pub perform: ::std::option::Option<unsafe extern "C" fn(info: *mut ::std::os::raw::c_void)>,
}

pub type CFArrayRetainCallBack = ::std::option::Option<
    unsafe extern "C" fn(
        allocator: CFAllocatorRef,
        value: *const ::std::os::raw::c_void,
    ) -> *const ::std::os::raw::c_void,
>;
pub type CFArrayReleaseCallBack = ::std::option::Option<
    unsafe extern "C" fn(allocator: CFAllocatorRef, value: *const ::std::os::raw::c_void),
>;
pub type CFArrayCopyDescriptionCallBack = ::std::option::Option<
    unsafe extern "C" fn(value: *const ::std::os::raw::c_void) -> CFStringRef,
>;
pub type CFArrayEqualCallBack = ::std::option::Option<
    unsafe extern "C" fn(
        value1: *const ::std::os::raw::c_void,
        value2: *const ::std::os::raw::c_void,
    ) -> Boolean,
>;

pub type CGWindowListOption = u32;

pub struct CFArrayCallBacks {
    pub version: CFIndex,
    pub retain: CFArrayRetainCallBack,
    pub release: CFArrayReleaseCallBack,
    pub copyDescription: CFArrayCopyDescriptionCallBack,
    pub equal: CFArrayEqualCallBack,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __CFNumber {
    _unused: [u8; 0],
}
pub type CFNumberRef = *const __CFNumber;
pub type CFNumberType = CFIndex;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub static kCGWindowNumber: CFStringRef;
    pub static kCGWindowStoreType: CFStringRef;
    pub static kCGWindowLayer: CFStringRef;
    pub static kCGWindowBounds: CFStringRef;
    pub static kCGWindowSharingState: CFStringRef;
    pub static kCGWindowAlpha: CFStringRef;
    pub static kCGWindowOwnerPID: CFStringRef;
    pub static kCGWindowMemoryUsage: CFStringRef;
    pub static kCGWindowWorkspace: CFStringRef;
    pub static kCGWindowOwnerName: CFStringRef;
    pub static kCGWindowName: CFStringRef;
    pub static kCGWindowIsOnscreen: CFStringRef;
    pub static kCGWindowBackingLocationVideoMemory: CFStringRef;

    pub fn CGWindowListCreateDescriptionFromArray(windowArray: CFArrayRef) -> CFArrayRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    pub static kCFAllocatorDefault: CFAllocatorRef;
    pub static kCFRunLoopDefaultMode: CFRunLoopMode;
    pub static kCFRunLoopCommonModes: CFRunLoopMode;
    pub static kCFBooleanTrue: CFBooleanRef;
    pub static kCFBooleanFalse: CFBooleanRef;

    pub fn CFGetRetainCount(cf: CFTypeRef) -> CFIndex;

    pub fn CFNumberGetValue(
        number: CFNumberRef,
        theType: CFNumberType,
        valuePtr: *mut ::std::os::raw::c_void,
    ) -> Boolean;

    pub fn CFArrayCreate(
        allocator: CFAllocatorRef,
        values: *mut *const ::std::os::raw::c_void,
        numValues: CFIndex,
        callBacks: *const CFArrayCallBacks,
    ) -> CFArrayRef;

    pub fn CFDictionaryGetValue(
        theDict: CFDictionaryRef,
        key: *const ::std::os::raw::c_void,
    ) -> *const ::std::os::raw::c_void;

    pub fn CGWindowListCopyWindowInfo(
        option: CGWindowListOption,
        relativeToWindow: CGWindowID,
    ) -> CFArrayRef;

    pub fn CFStringCreateWithBytes(
        alloc: CFAllocatorRef,
        bytes: *const UInt8,
        numBytes: CFIndex,
        encoding: CFStringEncoding,
        isExternalRepresentation: Boolean,
    ) -> CFStringRef;

    pub fn CFRunLoopAddSource(
        rl: *mut __CFRunLoopSource,
        source: *mut __CFRunLoopSource,
        mode: CFRunLoopMode,
    );

    pub fn CFRunLoopGetMain() -> *mut __CFRunLoopSource;

    pub fn CFRunLoopSourceInvalidate(source: *mut __CFRunLoopSource);

    pub fn CFRelease(cf: CFTypeRef);

    pub fn CFBooleanGetValue(boolean: CFBooleanRef) -> Boolean;

    pub fn CFEqual(cf1: CFTypeRef, cf2: CFTypeRef) -> Boolean;

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

    pub fn CFArrayGetCount(theArray: CFArrayRef) -> CFIndex;

    pub fn CFArrayGetValueAtIndex(
        theArray: CFArrayRef,
        idx: CFIndex,
    ) -> *const ::std::os::raw::c_void;

    pub fn CFRetain(cf: CFTypeRef) -> CFTypeRef;

    pub fn CFDictionarySetValue(
        theDict: CFMutableDictionaryRef,
        key: *const ::std::os::raw::c_void,
        value: *const ::std::os::raw::c_void,
    );

    pub fn CFDictionaryCreate(
        allocator: CFAllocatorRef,
        keys: *mut *const ::std::os::raw::c_void,
        values: *mut *const ::std::os::raw::c_void,
        numValues: CFIndex,
        keyCallBacks: *const CFDictionaryKeyCallBacks,
        valueCallBacks: *const CFDictionaryValueCallBacks,
    ) -> CFDictionaryRef;

    pub fn CFRunLoopGetCurrent() -> *mut __CFRunLoopSource;

    pub fn CFRunLoopRun();

    pub fn CFRunLoopRunInMode(
        mode: CFRunLoopMode,
        seconds: CFTimeInterval,
        returnAfterSourceHandled: Boolean,
    ) -> CFRunLoopRunResult;

    pub fn CFStringGetMaximumSizeForEncoding(
        length: CFIndex,
        encoding: CFStringEncoding,
    ) -> CFIndex;

    pub fn CFRunLoopSourceSignal(source: *mut __CFRunLoopSource);

    pub fn CFRunLoopSourceCreate(
        allocator: CFAllocatorRef,
        order: CFIndex,
        context: *mut CFRunLoopSourceContext,
    ) -> *mut __CFRunLoopSource;

    pub fn CFRunLoopWakeUp(rl: *mut __CFRunLoopSource);

    pub fn CFDictionaryGetKeysAndValues(
        theDict: CFDictionaryRef,
        keys: *mut *const ::std::os::raw::c_void,
        values: *mut *const ::std::os::raw::c_void,
    );

    pub fn CFDictionaryGetCount(theDict: CFDictionaryRef) -> CFIndex;
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    pub static mut kAXTrustedCheckOptionPrompt: CFStringRef;

    pub fn AXUIElementCreateApplication(pid: pid_t) -> *const __AXUIElement;

    pub fn AXObserverCreate(
        application: pid_t,
        callback: AXObserverCallback,
        outObserver: *mut *mut __AXObserver,
    ) -> AXError;

    pub fn AXObserverAddNotification(
        observer: *mut __AXObserver,
        element: *const __AXUIElement,
        notification: CFStringRef,
        refcon: *mut ::std::os::raw::c_void,
    ) -> AXError;

    pub fn AXObserverGetRunLoopSource(observer: *mut __AXObserver) -> *mut __CFRunLoopSource;

    pub fn AXObserverRemoveNotification(
        observer: *mut __AXObserver,
        element: *const __AXUIElement,
        notification: CFStringRef,
    ) -> AXError;

    pub fn AXUIElementSetAttributeValue(
        element: *const __AXUIElement,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> AXError;

    pub fn AXUIElementCopyAttributeValue(
        element: *const __AXUIElement,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;

    pub fn AXObserverCreateWithInfoCallback(
        application: pid_t,
        callback: AXObserverCallbackWithInfo,
        outObserver: *mut *mut __AXObserver,
    ) -> AXError;

    // PRIVATE API
    pub fn _AXUIElementGetWindow(element: *const __AXUIElement, identifier: *mut CGWindowID)
        -> i32;

    pub fn AXValueGetValue(
        value: AXValueRef,
        theType: AXValueType,
        valuePtr: *mut ::std::os::raw::c_void,
    ) -> Boolean;

    pub fn AXUIElementPerformAction(element: *const __AXUIElement, action: CFStringRef) -> AXError;

    pub fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> Boolean;

    pub fn AXIsProcessTrusted() -> Boolean;

    pub fn AXUIElementGetPid(element: *const __AXUIElement, pid: *mut pid_t) -> AXError;

    pub fn AXUIElementCreateSystemWide() -> AXUIElementRef;

    pub fn AXUIElementSetMessagingTimeout(
        element: *const __AXUIElement,
        timeoutInSeconds: f32,
    ) -> AXError;

    pub fn AXUIElementIsAttributeSettable(
        element: *const __AXUIElement,
        attribute: CFStringRef,
        settable: *mut Boolean,
    ) -> AXError;
}

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
        let length = CFStringGetLength(cfstring);
        let max_length = CFStringGetMaximumSizeForEncoding(length, kCFStringEncodingUTF8) + 1;
        // TODO: error if length > usize
        let mut buffer: Vec<c_char> = Vec::with_capacity(max_length as usize);

        if CFStringGetCString(
            cfstring,
            buffer.as_mut_ptr(),
            max_length,
            kCFStringEncodingUTF8,
        ) != 0
        {
            CFRelease(cfstring as *const _);

            Some(
                CString::from_raw(buffer.leak().as_mut_ptr())
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
