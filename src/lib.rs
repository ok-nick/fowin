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

// impl WindowManagerBackend for WindowManager {}
