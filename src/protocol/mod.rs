use std::time::Instant;

use crate::sys;

pub use self::window::Window;

mod window;

/// A unique identifier representing a window.
pub type WindowId = u32;

/// A posiiton with an x and y axis.
#[derive(Debug)]
pub struct Position {
    /// The x position.
    pub x: f64,
    /// The y position.
    pub y: f64,
}

/// A size with width and height.
#[derive(Debug)]
pub struct Size {
    /// The width of the size.
    pub width: f64,
    /// The height of the size.
    pub height: f64,
}

// TODO: consider writing the protocol as traits so that they can be used w/ third party crates

// generic interface over backend, provides functions like:
// window resized, window moved, etc. is it possible to be generic over this for all platforms?

// Keybinds:
// https://github.com/Narsil/rdev
// https://github.com/obv-mikhail/inputbot
// https://github.com/tauri-apps/global-hotkey

// Windowing/keybinds:
// https://github.com/RustAudio/baseview/tree/master

// Windowing:
// https://github.com/LGUG2Z/komorebi/blob/master/komorebi/src/window.rs
//    https://github.com/LGUG2Z/komorebi/blob/master/komorebi/src/windows_api.rs#L361
//    https://github.com/ianyh/Silica (used by Amethyst)

// komorebi:
// EVENT_OBJECT_DESTROY
// EVENT_OBJECT_HIDE
// EVENT_OBJECT_CLOAKED
// EVENT_SYSTEM_MINIMIZESTART
// EVENT_OBJECT_SHOW | EVENT_SYSTEM_MINIMIZEEND
// EVENT_OBJECT_UNCLOAKED
// EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND
// EVENT_SYSTEM_MOVESIZESTART
// EVENT_SYSTEM_MOVESIZEEND
// EVENT_SYSTEM_CAPTURESTART | EVENT_SYSTEM_CAPTUREEND
// EVENT_OBJECT_NAMECHANGE

// same:
// kAXWindowCreatedNotification | EVENT_OBJECT_CREATE
// kAXUIElementDestroyedNotification | EVENT_OBJECT_DESTROY
// kAXWindowMiniaturizedNotification | EVENT_OBJECT_HIDE | EVENT_OBJECT_CLOAKED | EVENT_SYSTEM_MINIMIZESTART
// kAXWindowDeminiaturizedNotification | EVENT_OBJECT_SHOW | EVENT_SYSTEM_MINIMIZEEND | EVENT_OBJECT_UNCLOAKED
// kAXFocusedWindowChangedNotification | EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND
// kAXMovedNotification | EVENT_SYSTEM_MOVESIZESTART | EVENT_SYSTEM_MOVESIZEEND
// (can be handled with diff macos api) | EVENT_SYSTEM_CAPTURESTART | EVENT_SYSTEM_CAPTUREEND
// kAXTitleChangedNotification | EVENT_OBJECT_NAMECHANGE

// NOTE: komorebi doesn't listen to EVENT_OBJECT_CREATE because "some apps like firefox" don't send them
// https://github.com/LGUG2Z/komorebi/blob/42ac13e0bd24c2775874cac891826024054e4e3c/komorebi/src/window_manager_event.rs#L127

/// The kind of window event that was sent.
#[derive(Debug, PartialEq, Eq)]
pub enum WindowEventKind {
    /// The window was first opened.
    Opened,
    /// The window was closed.
    Closed,
    /// The window was hidden.
    Hidden,
    /// The window was shown.
    Shown,
    /// The window was focused.
    Focused,
    /// The window was moved.
    Moved,
    /// The window was resized.
    Resized,
    /// The window title was renamed.
    Renamed,
}

/// An event signifying a change in window properties.
#[derive(Debug)]
pub struct WindowEvent {
    kind: WindowEventKind,
    window: sys::Window,
    timestamp: Instant,
}

impl WindowEvent {
    /// Create a new [`WindowEvent`](WindowEvent) with the specified kind and window.
    pub fn new(kind: WindowEventKind, window: Window) -> WindowEvent {
        WindowEvent {
            kind,
            window: window.0,
            timestamp: Instant::now(),
        }
    }

    /// Create a new [`WindowEvent`](WindowEvent) with the specified kind, window, and timestamp.
    pub fn with_timestamp(
        kind: WindowEventKind,
        window: Window,
        timestamp: Instant,
    ) -> WindowEvent {
        WindowEvent {
            kind,
            window: window.0,
            timestamp,
        }
    }
}

// TODO: add context
/// An error caused by the underlying operating system.
#[derive(Debug)]
pub enum WindowError {
    /// The API used to operate on windows is disabled.
    ApiDisabled,
    /// An invalid argument was passed internally.
    ///
    /// This type of error means there is a bug in this library!
    InvalidInternalArgument,
    // TODO: change to InvalidInternalArgument instead?
    /// The window is already being watched for this event.
    ///
    /// This type of error should never be possible.
    AlreadyWatching,
    /// Cannot unwatch if it was never watched in the first place.
    WasNeverWatching,
    /// The handle to the window is invalid. This could mean it no longer exists.
    InvalidHandle,
    /// The specified window does not support this type of operation.
    AlienUnsupported,
    /// There was a random internal failure in the operating system.
    ArbitraryFailure,
}
