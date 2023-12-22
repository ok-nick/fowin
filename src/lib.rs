use crossbeam::channel::{self, Receiver, Sender};
use once_cell::sync::Lazy;
use protocol::{Position, Size, WindowEvent, WindowId, WindowManagerBackend};

mod protocol;
mod sys;

static GLOBAL_CHANNEL: Lazy<(Sender<WindowEvent>, Receiver<WindowEvent>)> =
    Lazy::new(channel::unbounded);

#[derive(Debug)]
pub struct WindowManager {
    sys: sys::WindowManager,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            sys: sys::WindowManager::new(),
        }
    }

    pub fn receiver<'a>() -> &'a Receiver<WindowEvent> {
        &GLOBAL_CHANNEL.1
    }

    pub fn set_event_handler() {
        // TODO: pass callback function
    }
}

impl WindowManagerBackend for WindowManager {
    fn show_window(&self, id: WindowId) {
        self.sys.show_window(id);
    }

    fn hide_window(&self, id: WindowId) {
        self.sys.hide_window(id);
    }

    fn focus_window(&self, id: WindowId) {
        self.sys.focus_window(id);
    }

    fn move_window(&self, id: WindowId, position: Position) {
        self.sys.move_window(id, position);
    }

    fn resize_window(&self, id: WindowId, size: Size) {
        self.sys.resize_window(id, size);
    }
}
