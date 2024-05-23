use std::{io, ptr, sync::LazyLock};

use flume::{Receiver, Sender};
use windows_sys::Win32::{
    Foundation::{BOOL, HANDLE, HWND, LPARAM, TRUE},
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        WindowsAndMessaging::{
            EnumWindows, GetForegroundWindow, EVENT_MAX, EVENT_MIN, EVENT_OBJECT_CLOAKED,
            EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_FOCUS, EVENT_OBJECT_HIDE,
            EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
            EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
            EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART, OBJID_WINDOW,
            WINEVENT_OUTOFCONTEXT,
        },
    },
};

use crate::{protocol, WindowError, WindowEvent};

pub use window::Window;

mod window;

pub type WindowHandle = HWND;

type Event = Result<WindowEvent, WindowError>;

// We need a multi-producer, multi-consumer channel to support the "local" nature of a Watcher, where
// multiple watchers can be created and manage their own state. Thus the reason we use flume, for
// MPMC channels.
static EVENT_SENDER: LazyLock<(Sender<Event>, Receiver<Event>)> = LazyLock::new(flume::unbounded);

#[derive(Debug)]
pub struct Watcher {
    handle: HWINEVENTHOOK,
    receiver: Receiver<Event>,
}

// NOTE: UnhookWinEvent must be executed on the same thread the hook was created
//       so do some additional runtime checks to ensure that
impl !Send for Watcher {}
impl !Sync for Watcher {}

impl Watcher {
    pub fn new() -> Result<Watcher, WindowError> {
        Ok(Watcher {
            handle: unsafe {
                // TODO: can also regiser multiple hooks with specific event ids
                SetWinEventHook(
                    EVENT_MIN,
                    EVENT_MAX,
                    ptr::null::<HANDLE>() as HANDLE,
                    Some(window_event),
                    0,
                    0,
                    WINEVENT_OUTOFCONTEXT, // TODO: also can try WINEVENT_INCONTEXT
                )
            },
            receiver: EVENT_SENDER.1.clone(),
        })
    }

    // What a beautiful sight in comparison to the macOS backend.
    pub fn next_request(&self) -> Result<WindowEvent, WindowError> {
        // Impossible to error, the sender lives as long as the program.
        self.receiver.recv().unwrap()
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        unsafe {
            UnhookWinEvent(self.handle);
        }
    }
}

// TODO: not sure if we need to request any perms for windows
pub fn trusted() -> bool {
    true
}

pub fn request_trust() -> Result<bool, WindowError> {
    Ok(true)
}

pub fn iter_windows() -> impl Iterator<Item = Result<Window, WindowError>> {
    // TODO: I can also do something w/ a sender here to get a more on-demand iterator
    let mut windows: Vec<HWND> = Vec::new();
    let result = unsafe { EnumWindows(Some(enum_windows), windows.as_mut_ptr() as LPARAM) };
    if result == TRUE {
        windows.into_iter().map(|window| Ok(Window::new(window)))
    } else {
        // TODO: need to either return iterator surrounded by error or return enum that impls Iterator
        // iter::once(WindowError::last_os_error())
        todo!()
    }
}

pub fn focused_window() -> Result<Option<Window>, WindowError> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd == 0 {
        Ok(None)
    } else {
        Ok(Some(Window::new(hwnd)))
    }
}

unsafe extern "system" fn enum_windows(hwnd: HWND, lParam: LPARAM) -> BOOL {
    let windows = lParam as *mut Vec<HWND>;
    (*windows).push(hwnd);
    TRUE
}

unsafe extern "system" fn window_event(
    hwineventhook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    idobject: i32,
    idchild: i32,
    ideventthread: u32,
    dwmseventtime: u32,
) {
    if idobject == OBJID_WINDOW {
        // TODO: https://github.com/LGUG2Z/komorebi/issues/151
        //       supposedly events are "guaranteed to be in sequential order" as described
        //       by SetWinEventHook docs, but komorebi says different? I wonder if the timestamp would
        //       notice the discrepency?
        // let timestamp = Duration::from_millis(dwmseventtime as u64);

        let window = protocol::Window(Window::new(hwnd));
        let event = match event {
            EVENT_OBJECT_CREATE => WindowEvent::Opened(window),
            EVENT_OBJECT_DESTROY => WindowEvent::Closed(protocol::WindowHandle(hwnd)),
            EVENT_OBJECT_HIDE | EVENT_OBJECT_CLOAKED | EVENT_SYSTEM_MINIMIZESTART => {
                WindowEvent::Hidden(window)
            }
            EVENT_OBJECT_SHOW | EVENT_OBJECT_UNCLOAKED | EVENT_SYSTEM_MINIMIZEEND => {
                WindowEvent::Shown(window)
            }
            EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND => WindowEvent::Focused(window),
            EVENT_SYSTEM_MOVESIZESTART | EVENT_SYSTEM_MOVESIZEEND => {
                // TODO: is there really no way to differentiate resizing/positioning events without caching?
                todo!()
            }
            EVENT_OBJECT_NAMECHANGE => WindowEvent::Renamed(window),
            _ => return,
        };

        let _ = EVENT_SENDER.0.send(Ok(event));
    }
}

impl WindowError {
    pub(self) fn last_os_error() -> WindowError {
        WindowError::OsError(io::Error::last_os_error())
    }
}
