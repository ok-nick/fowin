use crate::{
    protocol::{Position, Size, WindowError, WindowHandle},
    sys,
};

/// Representation of a single window that can be queried and operated on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window(pub(crate) sys::Window);

impl Window {
    // TODO: Is there a point to returning this struct? maybe we should have a platform-specific way to return a raw handle instead. The handle could be useful for hashing (e.g. key in hash map), but maybe it'd be better to just hash the window struct itself (need to impl Hash)?
    /// A handle associated with the window.
    #[inline]
    pub fn handle(&self) -> WindowHandle {
        WindowHandle(self.0.handle())
    }

    /// The title of the window.
    #[inline]
    pub fn title(&self) -> Result<String, WindowError> {
        self.0.title()
    }

    /// The logical size of the window.
    #[inline]
    pub fn size(&self) -> Result<Size, WindowError> {
        self.0.size()
    }

    /// The logical position of the window relative to the current display.
    #[inline]
    pub fn position(&self) -> Result<Position, WindowError> {
        self.0.position()
    }

    /// Whether or not the window is focused.
    #[inline]
    pub fn is_focused(&self) -> Result<bool, WindowError> {
        self.0.is_focused()
    }

    /// Whether or not the window is fullscreened.
    #[inline]
    pub fn is_fullscreen(&self) -> Result<bool, WindowError> {
        self.0.is_fullscreen()
    }

    /// Whether or not the window is minimized.
    #[inline]
    pub fn is_minimized(&self) -> Result<bool, WindowError> {
        self.0.is_minimized()
    }

    /// Whether or not the window is hidden.
    ///
    /// On macOS, this function checks if the window is minimized.
    ///
    /// On Windows, this function checks if the window is hidden or cloaked.
    /// "Cloaking" a window is a fancy way of hiding it, [read more here](https://devblogs.microsoft.com/oldnewthing/20200302-00/?p=103507).
    #[inline]
    pub fn is_hidden(&self) -> Result<bool, WindowError> {
        self.0.is_hidden()
    }

    /// Change the size of the window.
    #[inline]
    pub fn resize(&self, size: Size) -> Result<(), WindowError> {
        self.0.resize(size)
    }

    /// Change the position of the window.
    #[inline]
    pub fn reposition(&self, position: Position) -> Result<(), WindowError> {
        self.0.reposition(position)
    }

    /// Focus the window.
    #[inline]
    pub fn focus(&self) -> Result<(), WindowError> {
        self.0.focus()
    }

    /// Fullscreen the window.
    #[inline]
    pub fn fullscreen(&self) -> Result<(), WindowError> {
        self.0.fullscreen()
    }

    /// Unfullscreen the window.
    #[inline]
    pub fn unfullscreen(&self) -> Result<(), WindowError> {
        self.0.unfullscreen()
    }

    /// Maximize the window.
    ///
    /// This means, make it the size of the current display and position it at the top-left.
    #[inline]
    pub fn maximize(&self) -> Result<(), WindowError> {
        self.0.maximize()
    }

    /// Minimizes the window.
    #[inline]
    pub fn minimize(&self) -> Result<(), WindowError> {
        self.0.minimize()
    }

    /// Unminimizes the window.
    #[inline]
    pub fn unminimize(&self) -> Result<(), WindowError> {
        self.0.unminimize()
    }

    /// Show the window.
    ///
    /// On macOS, this function will unminimize the window. However, if the application is hidden,
    /// this function will unhide the application, then hide (minimize) all of the other windows.
    /// Note that a hidden application means that all windows for that application are hidden from
    /// view.
    ///
    /// On Windows, this function will cloak the window rather than hide it. Read [`Window::is_hidden`](Window::is_hidden)
    /// for more information.
    #[inline]
    pub fn show(&self) -> Result<(), WindowError> {
        self.0.show()
    }

    /// Hide the window.
    ///
    /// On macOS, the default behavior is to minimize the window.
    ///
    /// On Windows, this function will cloak the window rather than hide it. Read [`Window::is_hidden`](Window::is_hidden)
    /// for more information.
    #[inline]
    pub fn hide(&self) -> Result<(), WindowError> {
        self.0.hide()
    }

    /// Bring the window to the front.
    ///
    /// This function does not focus the window.
    #[inline]
    pub fn bring_to_front(&self) -> Result<(), WindowError> {
        self.0.bring_to_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn needs_send<T: Send>() {}
    fn needs_sync<T: Sync>() {}

    #[test]
    fn test_window_send() {
        needs_send::<Window>();
    }

    #[test]
    fn test_window_sync() {
        needs_sync::<Window>();
    }
}
