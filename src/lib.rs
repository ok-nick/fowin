#![feature(mutex_unpoison)]

use protocol::WindowEvent;
pub use protocol::{Position, Size, Window, WindowError, WindowEventInfo, WindowId};

mod protocol;
mod sys;

/// A handle that provides various methods for interacting with windows and window events.
#[derive(Debug)]
pub struct Watcher(sys::Watcher);

impl Watcher {
    /// Returns an iterator over all existing windows.
    ///
    /// This function differs from [`iter_windows`](iter_windows) in that it iterates a
    /// cached set of updated windows, offering better efficiency.
    #[inline]
    pub fn iter_windows(&self) -> impl Iterator<Item = Result<Window, WindowError>> + '_ {
        self.0.iter_windows().map(|result| result.map(Window))
    }

    /// Returns the next window event.
    ///
    /// Note, these events are not guaranteed to be precisely ordered. However, they do provide
    /// a timestamp that can be used for ordering. Consider buffering events if order is important.
    #[inline]
    pub fn next_request(&self) -> Result<WindowEvent, WindowError> {
        self.0.next_request()
    }
}

/// Returns whether or not permission is granted to access the necessary APIs.
#[inline]
pub fn trusted() -> bool {
    sys::trusted()
}

/// Requests permission from the operating systems to access the necessary APIs.
///
/// On macOS, this function will open a prompt for the user to accept.
#[inline]
pub fn request_trust() -> Result<bool, WindowError> {
    sys::request_trust()
}

/// Returns an iterator over all existing windows.
///
/// This function differs from [`Watcher::iter_windows`](Watcher::iter_windows)
/// in that it uses an ad-hoc approach to ask the operating system for a list of
/// existing windows. Use [`Watcher::iter_windows`](Watcher::iter_windows)
/// if you already have a handle to take advantage of caching.
#[inline]
pub fn iter_windows() -> impl Iterator<Item = Result<Window, WindowError>> {
    sys::iter_windows().map(|result| result.map(Window))
}

/// Watches for all window events and passes them to the specified sender.
///
/// To stop watching events, drop the returned [Watcher](Watcher).
///
/// Note, this function will begin listening to new events. To access a list of
/// existing windows, call [`Watcher::iter_windows`](Watcher::iter_windows).
#[inline]
pub fn watch() -> Result<Watcher, WindowError> {
    Ok(Watcher(sys::watch()?))
}
