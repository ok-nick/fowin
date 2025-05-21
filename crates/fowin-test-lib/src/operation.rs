use crate::state::State;

#[derive(Debug)]
pub enum Scope {
    Local,
    Foreign,
    Global,
}

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Resize,
    Move,
    Fullscreen,
    Unfullscreen,
    Show,
    Hide,
    BringToFront,
    // TODO: this property also needs to somehow unfocus the last window
    Focus,
    Rename,
}

impl Operation {
    pub const ALL: [Operation; 9] = [
        Operation::Resize,
        Operation::Move,
        Operation::Fullscreen,
        Operation::Unfullscreen,
        Operation::Show,
        Operation::Hide,
        Operation::BringToFront,
        Operation::Focus,
        Operation::Rename,
    ];

    pub fn satisfied(&self, state: &State) -> bool {
        match self {
            Operation::Resize => !state.fullscreen && !state.hidden,
            Operation::Move => !state.fullscreen && !state.hidden,
            _ => true,
        }
    }

    pub const fn scope(&self) -> Scope {
        match self {
            Operation::Resize => Scope::Global,
            Operation::Move => Scope::Global,
            Operation::Fullscreen => Scope::Global,
            Operation::Unfullscreen => Scope::Global,
            Operation::Show => Scope::Global,
            Operation::Hide => Scope::Global,
            Operation::BringToFront => Scope::Global,
            Operation::Focus => Scope::Global,
            Operation::Rename => Scope::Local,
        }
    }
}
