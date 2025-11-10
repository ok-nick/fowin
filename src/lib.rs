pub use protocol::{Position, Size, Window, WindowError, WindowEvent, WindowHandle};

mod protocol;
mod sys;

/// A handle that provides various methods for interacting with windows and window events.
#[derive(Debug)]
pub struct Watcher {
    inner: sys::Watcher,
}

impl Watcher {
    /// Watches for all window events.
    ///
    /// To stop watching events, drop the returned [Watcher](Watcher).
    ///
    /// Note, this function will begin listening to new events. To access a list of
    /// existing windows, call [`Watcher::iter_windows`](Watcher::iter_windows).
    #[inline]
    pub fn new() -> Result<Watcher, WindowError> {
        Ok(Watcher {
            inner: sys::Watcher::new()?,
        })
    }

    /// Returns the next window event.
    ///
    /// Note, these events are not guaranteed to be precisely ordered. However, they do provide
    /// a timestamp that can be used for ordering. Consider buffering events if order is important.
    #[inline]
    pub fn next_request(&mut self) -> Result<WindowEvent, WindowError> {
        self.inner.next_request()
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
#[inline]
pub fn iter_windows() -> impl Iterator<Item = Result<Window, WindowError>> {
    sys::iter_windows().map(|result| result.map(Window))
}

/// Returns the globally focused window if one exists.
#[inline]
pub fn focused_window() -> Result<Option<Window>, WindowError> {
    sys::focused_window().map(|option| option.map(Window))
}
