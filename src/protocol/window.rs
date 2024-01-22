use crate::{
    protocol::{Position, Size, WindowError, WindowId},
    sys,
};

/// Representation of a single window that can be queried and operated on.
#[derive(Debug)]
pub struct Window(pub(crate) sys::Window);

impl Window {
    /// A unique identifier associated with the window.
    ///
    /// This id is guaranteed to be unique.
    #[inline]
    pub fn id(&self) -> Result<WindowId, WindowError> {
        self.0.id()
    }

    /// The title of the window.
    #[inline]
    pub fn title(&self) -> Result<String, WindowError> {
        self.0.title()
    }

    /// The size of the window.
    #[inline]
    pub fn size(&self) -> Result<Size, WindowError> {
        self.0.size()
    }

    /// The position of the window relative to the current display.
    #[inline]
    pub fn position(&self) -> Result<Position, WindowError> {
        self.0.position()
    }

    /// Whether or not the window is focused.
    #[inline]
    pub fn focused(&self) -> Result<bool, WindowError> {
        self.0.focused()
    }

    /// Whether or not the window is fullscreened.
    #[inline]
    pub fn fullscreened(&self) -> Result<bool, WindowError> {
        self.0.fullscreened()
    }

    /// Whether or not the window is minimized.
    #[inline]
    pub fn minimized(&self) -> Result<bool, WindowError> {
        self.0.minimized()
    }

    /// Whether or not the window is visible.
    #[inline]
    pub fn visible(&self) -> Result<bool, WindowError> {
        self.0.visible()
    }

    /// Whether or not the window still exists.
    #[inline]
    pub fn exists(&self) -> Result<bool, WindowError> {
        self.0.exists()
    }

    /// Change the size of the window.
    #[inline]
    pub fn resize(&self, size: Size) -> Result<(), WindowError> {
        self.0.resize(size)
    }

    /// Change the position of the window.
    #[inline]
    pub fn translate(&self, position: Position) -> Result<(), WindowError> {
        self.0.translate(position)
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

    /// Show the window.
    ///
    /// On macOS, an application may be considered hidden, causing all of its windows to also
    /// be hidden. This function will unhide the application then hide all of the other windows
    /// to show the current window.
    #[inline]
    pub fn show(&self) -> Result<(), WindowError> {
        self.0.show()
    }

    /// Hide the window.
    ///
    /// On macOS, hiding a window does not remove it from the dock.
    #[inline]
    pub fn hide(&self) -> Result<(), WindowError> {
        self.0.hide()
    }

    /// Bring the window to the front.
    ///
    /// However, this function does not focus the window.
    #[inline]
    pub fn bring_to_front(&self) -> Result<(), WindowError> {
        self.0.bring_to_front()
    }
}
