use std::{error::Error, fmt, io, time::Instant};

pub use window::Window;

use crate::sys;

mod window;

// TODO: differentiate physical and logical pixels

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

/// A handle representing a window.
///
/// This handle is only guaranteed to be unique whilst the underlying window is alive, they may be recycled. If you are caching windows and using their
/// handle as an "identifier," then there are two things you should be sure to handle (no pun intended):
/// * If a window is destroyed, consider the handle disposed and remove it from the cache
/// * If a window is created, check equality on all recorded handles, if there is a match, then the handle was reused and the old handle should be disposed
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct WindowHandle(pub(crate) sys::WindowHandle);

/// An event signifying a change in window properties.
#[derive(Debug)]
pub enum WindowEvent {
    /// The window was first opened.
    Opened(Window),
    /// The window was closed.
    Closed(WindowHandle),
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
    // TODO: error type derived from windows where errors aren't predictable (maybe combine this w/ ArbitraryFailure)
    ///
    OsError(io::Error),
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
            WindowError::OsError(_) => {
                write!(f, "TODO")
            }
        }
    }
}
