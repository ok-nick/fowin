use std::{error::Error, fmt, time::Instant};

pub use self::window::Window;

mod window;

/// A unique identifier representing a window.
pub type WindowId = u32;

/// A posiiton with an x and y axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// The x position.
    pub x: f64,
    /// The y position.
    pub y: f64,
}

/// A size with width and height.
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug)]
pub enum WindowEventInfo {
    /// The window was first opened.
    Opened(Window),
    /// The window was closed.
    Closed(WindowId),
    /// The window was hidden.
    Hidden(Window),
    /// The window was shown.
    Shown(Window),
    /// The window was focused.
    Focused(Window),
    /// The window was moved.
    Moved(Window),
    /// The window was resized.
    Resized(Window),
    /// The window title was renamed.
    Renamed(Window),
}

/// An event signifying a change in window properties.
#[derive(Debug)]
pub struct WindowEvent {
    info: WindowEventInfo,
    // window: Window,
    timestamp: Instant,
}

impl WindowEvent {
    /// Create a new [`WindowEvent`](WindowEvent) with the specified event info.
    pub fn new(info: WindowEventInfo) -> WindowEvent {
        WindowEvent {
            info,
            timestamp: Instant::now(),
        }
    }

    /// Create a new [`WindowEvent`](WindowEvent) with the specified event info and timestamp.
    pub fn with_timestamp(info: WindowEventInfo, timestamp: Instant) -> WindowEvent {
        WindowEvent { info, timestamp }
    }

    /// Returns the info of the window event.
    pub fn info(&self) -> &WindowEventInfo {
        &self.info
    }

    /// Returns whethere this window event happened before the specified window event.
    pub fn before(&self, other: WindowEvent) -> bool {
        self.timestamp < other.timestamp
    }
}

// TODO: add context to errors
/// An error caused by the operating system.
#[derive(Debug)]
pub enum WindowError {
    /// The program does not have sufficient permissions to access the underlying API. Call [request_trust](crate::request_trust) to request the necessary permission.
    NotTrusted,
    /// An invalid argument was passed internally.
    ///
    /// This type of error means there is a bug in this library!
    InvalidInternalArgument,

    /// The window is already being watched for this event.
    ///
    /// This type of error should never be possible.
    // AlreadyWatching,

    /// Cannot unwatch if it was never watched in the first place.
    // WasNeverWatching,

    /// The handle to the window is invalid. This could mean it no longer exists.
    InvalidHandle,
    /// The specified window does not support this type of operation.
    Unsupported,
    /// There was a random internal failure in the operating system.
    ArbitraryFailure,
}
impl Error for WindowError {}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowError::NotTrusted => {
                write!(
                    f,
                    "the program needs to request permission to the underlying API"
                )
            }
            WindowError::InvalidInternalArgument => {
                write!(
                    f,
                    "internal bug, input incorrect parameter, it's not you it's me!"
                )
            }
            // WindowError::AlreadyWatching => {
            //     write!(f, "already watching this window")
            // }
            // WindowError::WasNeverWatching => {
            //     write!(f, "cannot unwatch a window that was never watched")
            // }
            WindowError::InvalidHandle => {
                write!(f, "cannot perform operation on invalid handle")
            }
            WindowError::Unsupported => {
                write!(f, "the window does not support the windowing API")
            }
            WindowError::ArbitraryFailure => {
                write!(f, "arbitrary failure returned by the operating system")
            }
        }
    }
}
