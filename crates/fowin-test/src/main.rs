fn main() {
    println!("Hello, world!");
}

// TODO: after each op verify all properties of the window are consistent
#[derive(Debug)]
pub enum Properties {
    Id, // TODO: remove this one?
    Title,
    Size,
    Position,
    Fullscreened,
    Maximized,
    Hidden,
    Layer, // aka at "bring to front"
}

// TODO: randomly generate a list of 1-100 ops combining local and foreign
//    * foreign ops = fowin
//    * local ops = winit
// after each op validation of properties must occur

#[derive(Debug)]
pub enum ForeignOperations {
    Resize,
    Move,
    Fullscreen,
    Maximize,
    Show,
    Hide,
    BringToFront,
    Focus,
}

// TODO: same as above but with some other stuff
#[derive(Debug)]
pub enum LocalOperations {
    Create,
    Destroy,
}

// TODO: a set of logcal rules that must be upheld to ensure proper testing
//       * make special rule that cleans up all windows at the end
// these rules will be applied after the creation of the operations list
#[derive(Debug)]
pub enum Rules {
    NoOperationsBeforeCreate,
    NoOperationsAfterDestroy,
    NoDestroyTwice,

    CantBringToFrontFullscreen,
    CantUnfocusFullscreen,
    CantMoveFullscreen,
    CantResizeFullscreen,
    // TODO: etc.
}
