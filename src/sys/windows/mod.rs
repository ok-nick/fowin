// TODO: windows backend

#[derive(Debug)]
pub struct Watcher {}

impl Watcher {
    pub fn new() -> Result<Watcher, WindowError> {
        Watcher {}
    }

    pub fn next_request(&self) -> Result<WindowEvent, WindowError> {
        todo!()
    }
}
