use windows_sys::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW},
};

use crate::{Position, Size, WindowError};

use super::WindowHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    inner: HWND,
}

impl Window {
    pub(crate) fn new(hwnd: HWND) -> Window {
        Window { inner: hwnd }
    }

    pub fn handle(&self) -> WindowHandle {
        self.inner
    }

    pub fn title(&self) -> Result<String, WindowError> {
        let len = unsafe { GetWindowTextLengthW(self.inner) };
        if len != 0 {
            let mut title = Vec::with_capacity(len as usize + 1);
            let len = unsafe { GetWindowTextW(self.inner, title.as_mut_ptr(), len as i32 + 1) };
            if len != 0 {
                // For cross-platform sake we coerce strings to UTF-8.
                Ok(String::from_utf16_lossy(&title[..(len as usize)]))
            } else {
                // TODO: could mean no title bar, or handle invalid
                Err(todo!())
            }
        } else {
            // TODO: window has no text or error (read doc remarks)
            Err(todo!())
        }
    }

    pub fn size(&self) -> Result<Size, WindowError> {
        todo!()
    }

    pub fn position(&self) -> Result<Position, WindowError> {
        todo!()
    }

    pub fn focused(&self) -> Result<bool, WindowError> {
        todo!()
    }

    pub fn fullscreened(&self) -> Result<bool, WindowError> {
        todo!()
    }

    pub fn minimized(&self) -> Result<bool, WindowError> {
        todo!()
    }

    pub fn visible(&self) -> Result<bool, WindowError> {
        todo!()
    }

    pub fn resize(&self, size: Size) -> Result<(), WindowError> {
        todo!()
    }

    pub fn translate(&self, position: Position) -> Result<(), WindowError> {
        todo!()
    }

    pub fn focus(&self) -> Result<(), WindowError> {
        todo!()
    }

    pub fn fullscreen(&self) -> Result<(), WindowError> {
        todo!()
    }

    pub fn unfullscreen(&self) -> Result<(), WindowError> {
        todo!()
    }

    pub fn maximize(&self) -> Result<(), WindowError> {
        todo!()
    }

    pub fn show(&self) -> Result<(), WindowError> {
        todo!()
    }

    pub fn hide(&self) -> Result<(), WindowError> {
        todo!()
    }

    pub fn bring_to_front(&self) -> Result<(), WindowError> {
        todo!()
    }
}
