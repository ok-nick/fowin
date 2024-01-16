use rand::{
    distributions::{Alphanumeric, Standard},
    prelude::Distribution,
    Rng,
};

use crate::state::{Mutation, State};

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

    pub fn mutation<R: Rng>(&self, rng: &mut R) -> Mutation {
        match self {
            Operation::Resize => Mutation::Size(rng.gen()),
            Operation::Move => Mutation::Position(rng.gen()),
            Operation::Fullscreen => Mutation::Fullscreen(true),
            Operation::Unfullscreen => Mutation::Fullscreen(false),
            Operation::Show => Mutation::Hidden(false),
            Operation::Hide => Mutation::Hidden(true),
            Operation::BringToFront => Mutation::AtFront(true),
            Operation::Focus => Mutation::Focused(true),
            Operation::Rename => Mutation::Title(
                // TODO: define str length somewhere
                String::from_utf8(rng.sample_iter(&Alphanumeric).take(16).collect()).unwrap(),
            ),
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
        match rng.gen_range(0..=8) {
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
