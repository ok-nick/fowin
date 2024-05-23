use std::{
    io,
    mem::{self, MaybeUninit},
    ptr, thread,
};

use windows_sys::Win32::{
    Foundation::{SetLastError, BOOL, FALSE, HWND, RECT, S_OK, TRUE},
    Graphics::{
        Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED},
        Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    },
    System::Threading::{AttachThreadInput, GetCurrentThreadId},
    UI::{
        Input::KeyboardAndMouse::SetFocus,
        WindowsAndMessaging::{
            GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GetWindowTextLengthW,
            GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetWindowLongPtrW,
            SetWindowPos, ShowWindow, GWL_STYLE, HWND_TOP, HWND_TOPMOST, SWP_FRAMECHANGED,
            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE,
            SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, SW_SHOW, WS_OVERLAPPEDWINDOW,
        },
    },
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
        unsafe {
            SetLastError(0);
        }

        let len = unsafe { GetWindowTextLengthW(self.inner) };
        if len != 0 {
            let mut title = Vec::with_capacity(len as usize + 1);
            let len = unsafe { GetWindowTextW(self.inner, title.as_mut_ptr(), len as i32 + 1) };
            if len != 0 {
                // For cross-platform sake we coerce strings to UTF-8.
                Ok(String::from_utf16_lossy(&title[..(len as usize)]))
            } else {
                // Could mean a few things according to docs, but we can't differentiate, they're all errors.
                Err(WindowError::last_os_error())
            }
        } else if len == 0 {
            // The window has no title.
            // TODO: return an Option? WindowError::Unavailable? Empty string?
            Ok(String::new())
        } else {
            Err(WindowError::last_os_error())
        }
    }

    pub fn size(&self) -> Result<Size, WindowError> {
        let mut rect: MaybeUninit<RECT> = MaybeUninit::uninit();
        if unsafe { GetWindowRect(self.inner, rect.as_mut_ptr()) } == TRUE {
            let rect = unsafe { rect.assume_init() };
            Ok(Size {
                width: (rect.right - rect.left) as f64,
                height: (rect.bottom - rect.top) as f64,
            })
        } else {
            Err(WindowError::last_os_error())
        }
    }

    // TODO: dedup with above
    pub fn position(&self) -> Result<Position, WindowError> {
        let mut rect: MaybeUninit<RECT> = MaybeUninit::uninit();
        if unsafe { GetWindowRect(self.inner, rect.as_mut_ptr()) } == TRUE {
            let rect = unsafe { rect.assume_init() };
            Ok(Position {
                x: rect.left as f64,
                y: rect.top as f64,
            })
        } else {
            Err(WindowError::last_os_error())
        }
    }

    pub fn is_focused(&self) -> Result<bool, WindowError> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd == 0 {
            Ok(false)
        } else {
            Ok(hwnd == self.inner)
        }
    }

    //  https://devblogs.microsoft.com/oldnewthing/20100412-00/?p=14353
    pub fn is_fullscreen(&self) -> Result<bool, WindowError> {
        let style = unsafe { GetWindowLongPtrW(self.inner, GWL_STYLE) };
        if style == 0 {
            Err(WindowError::last_os_error())
        } else {
            Ok(style & WS_OVERLAPPEDWINDOW as isize != WS_OVERLAPPEDWINDOW as isize)
        }
    }

    pub fn is_minimized(&self) -> Result<bool, WindowError> {
        Ok(unsafe { IsIconic(self.inner) != 0 })
    }

    // https://devblogs.microsoft.com/oldnewthing/20200302-00/?p=103507
    pub fn is_hidden(&self) -> Result<bool, WindowError> {
        if unsafe { IsWindowVisible(self.inner) } == FALSE {
            return Ok(false);
        }

        let mut is_cloaked = FALSE;
        let result = unsafe {
            DwmGetWindowAttribute(
                self.inner,
                DWMWA_CLOAKED as u32,
                &mut is_cloaked as *mut _ as *mut _,
                mem::size_of::<BOOL>() as u32,
            )
        };
        if result == S_OK {
            Ok(is_cloaked == FALSE)
        } else {
            Err(WindowError::OsError(io::Error::from_raw_os_error(result)))
        }
    }

    pub fn resize(&self, size: Size) -> Result<(), WindowError> {
        let result = unsafe {
            SetWindowPos(
                self.inner,
                HWND_TOP,
                0,
                0,
                size.width as i32,
                size.height as i32,
                SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOZORDER,
            )
        };
        if result != 0 {
            Ok(())
        } else {
            Err(WindowError::last_os_error())
        }
    }

    // TODO: dedup above
    // TODO: DeferWindowPos for bulk manipulations
    pub fn reposition(&self, position: Position) -> Result<(), WindowError> {
        if unsafe {
            SetWindowPos(
                self.inner,
                HWND_TOP,
                position.x as i32,
                position.y as i32,
                0,
                0,
                // TODO: SWP_NOSENDCHANGING | SWP_NOCOPYBITS | SWP_FRAMECHANGED
                SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER,
            )
        } != 0
        {
            Ok(())
        } else {
            Err(WindowError::last_os_error())
        }
    }

    pub fn focus(&self) -> Result<(), WindowError> {
        // Start by spawning a new thread so that attaching the window thread's
        // input doesn't affect the caller's code.
        thread::scope(|s| {
            s.spawn(|| {
                let current_thread_id = unsafe { GetCurrentThreadId() };

                let target_thread_id =
                    unsafe { GetWindowThreadProcessId(self.inner, ptr::null_mut()) };
                if target_thread_id == 0 {
                    return Err(WindowError::last_os_error());
                }

                if unsafe { AttachThreadInput(current_thread_id, target_thread_id, TRUE) } == 0 {
                    // Note that this cannot be called on Windows Server 2003 and Windows XP.
                    // However, Rust does not support these versions anyways.
                    return Err(WindowError::last_os_error());
                }

                if unsafe { SetFocus(self.inner) } == 0 {
                    return Err(WindowError::last_os_error());
                }

                if unsafe { AttachThreadInput(current_thread_id, target_thread_id, FALSE) } == 0 {
                    // Same here as above.
                    return Err(WindowError::last_os_error());
                }

                Ok(())
            })
            .join()
            // TODO: if the thread panics what should we do?
            .unwrap()
        })
    }

    // https://devblogs.microsoft.com/oldnewthing/20100412-00/?p=14353
    pub fn fullscreen(&self) -> Result<(), WindowError> {
        let style = unsafe { GetWindowLongPtrW(self.inner, GWL_STYLE) };
        if style == 0 {
            return Err(WindowError::last_os_error());
        }

        // TODO: need this check?
        if style & WS_OVERLAPPEDWINDOW as isize == WS_OVERLAPPEDWINDOW as isize {
            let mut info = MONITORINFO {
                cbSize: mem::size_of::<MONITORINFO>() as u32,
                rcMonitor: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                rcWork: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                dwFlags: 0,
            };

            if unsafe {
                GetMonitorInfoW(
                    MonitorFromWindow(self.inner, MONITOR_DEFAULTTONEAREST),
                    &mut info,
                )
            } == 0
            {
                return Err(WindowError::last_os_error());
            }

            if unsafe {
                SetWindowLongPtrW(
                    self.inner,
                    GWL_STYLE,
                    style & !(WS_OVERLAPPEDWINDOW as isize),
                )
            } == 0
            {
                return Err(WindowError::last_os_error());
            }

            if unsafe {
                SetWindowPos(
                    self.inner,
                    HWND_TOP,
                    info.rcMonitor.left,
                    info.rcMonitor.top,
                    info.rcMonitor.right - info.rcMonitor.left,
                    info.rcMonitor.bottom - info.rcMonitor.top,
                    SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
                )
            } == 0
            {
                return Err(WindowError::last_os_error());
            }
        }

        Ok(())
    }

    // https://devblogs.microsoft.com/oldnewthing/20100412-00/?p=14353
    pub fn unfullscreen(&self) -> Result<(), WindowError> {
        let style = unsafe { GetWindowLongPtrW(self.inner, GWL_STYLE) };
        if style == 0 {
            return Err(WindowError::last_os_error());
        }

        if unsafe { SetWindowLongPtrW(self.inner, GWL_STYLE, style | WS_OVERLAPPEDWINDOW as isize) }
            == 0
        {
            return Err(WindowError::last_os_error());
        }

        if unsafe {
            SetWindowPos(
                self.inner,
                0,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
            )
        } == 0
        {
            return Err(WindowError::last_os_error());
        }

        Ok(())
    }

    // To "unmaximize" a window, callers can explicitly set the size/position.
    pub fn maximize(&self) -> Result<(), WindowError> {
        unsafe {
            ShowWindow(self.inner, SW_MAXIMIZE);
        }

        Ok(())
    }

    pub fn minimize(&self) -> Result<(), WindowError> {
        unsafe {
            ShowWindow(self.inner, SW_MINIMIZE);
        }

        Ok(())
    }

    pub fn unminimize(&self) -> Result<(), WindowError> {
        unsafe {
            ShowWindow(self.inner, SW_RESTORE);
        }

        Ok(())
    }

    // TODO: read hide
    pub fn show(&self) -> Result<(), WindowError> {
        unsafe {
            ShowWindow(self.inner, SW_SHOW);
        }

        Ok(())
    }

    // TODO: use cloaking
    // https://github.com/LGUG2Z/komorebi/issues/151#issuecomment-1428027101
    // https://github.com/LGUG2Z/komorebi/commit/80c98596dd2cb4e28666e49c753ffc7e5137e09e
    // https://github.com/Ciantic/AltTabAccessor/issues/1#issuecomment-1894378295
    //
    // TODO: https://github.com/LGUG2Z/komorebi/blob/master/komorebi/src/com/mod.rs
    pub fn hide(&self) -> Result<(), WindowError> {
        unsafe {
            ShowWindow(self.inner, SW_HIDE);
        }

        Ok(())
    }

    pub fn bring_to_front(&self) -> Result<(), WindowError> {
        if unsafe {
            SetWindowPos(
                self.inner,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE,
            )
        } != 0
        {
            Ok(())
        } else {
            Err(WindowError::last_os_error())
        }
    }
}
