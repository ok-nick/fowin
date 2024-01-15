use rand::{distributions::Standard, prelude::Distribution, Rng};

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
            Operation::Resize => state.fullscreen && !state.hidden,
            Operation::Move => state.fullscreen && !state.hidden,
            _ => true,
        }
    }

    pub fn apply<R: Rng>(&self, state: &mut State, rng: &mut R) {
        match self {
            Operation::Resize => {
                state.size = todo!(); // randomize
            }
            Operation::Move => {
                state.position = todo!() // randomize
            }
            Operation::Fullscreen => {
                state.fullscreen = true;
            }
            Operation::Unfullscreen => {
                state.fullscreen = false;
            }
            Operation::Show => {
                state.hidden = false;
            }
            Operation::Hide => {
                state.hidden = true;
            }
            Operation::BringToFront => {
                state.at_front = true;
            }
            Operation::Focus => {
                state.focused = true;
            }
            Operation::Rename => {
                state.title = todo!(); //randomize
            }
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

impl Distribution<Operation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Operation {
        match rng.gen_range(0..8) {
            0 => Operation::Resize,
            1 => Operation::Move,
            2 => Operation::Fullscreen,
            3 => Operation::Unfullscreen,
            4 => Operation::Show,
            5 => Operation::Hide,
            6 => Operation::BringToFront,
            7 => Operation::Focus,
            _ => Operation::Rename,
        }
    }
}
