#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod macos;

// NOTE: the reason we explicitly use physical size rather than logical size is for consistency

// physical pixels are useful in a multi-monitor window manager so that res changes
// don't influence window position

#[derive(Debug)]
pub struct PhysicalSize {
    pub width: u64,
    pub height: u64,
}

#[derive(Debug)]
pub struct PhysicalPosition {
    pub x: u64,
    pub y: u64,
}

#[derive(Debug)]
pub struct LogicalSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug)]
pub struct Screen {}

impl Screen {
    pub fn size(&self) -> PhysicalSize {
        todo!()
    }

    pub fn position(&self) -> PhysicalPosition {
        todo!()
    }
}

// TODO: also provide events for displays, such as power on/off, etc.
