use rand::{distributions::Standard, prelude::Distribution, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Distribution<Position> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Position {
        // TODO: size must be within monitor size bounds
        Position {
            x: rng.gen(),
            y: rng.gen(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Distribution<Size> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Size {
        // TODO: size must be within monitor size bounds
        Size {
            width: rng.gen(),
            height: rng.gen(),
        }
    }
}

// TODO: can impl this without a hashmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub title: String,
    pub size: Size,
    pub position: Position,
    pub fullscreen: bool,
    pub hidden: bool,
    // TODO: these two props can't be guaranteed, a new window will be focused/at_front..
    //       can introduce rules to define this behavior?
    pub at_front: bool,
    pub focused: bool,
}

impl State {
    pub fn new() -> State {
        State {
            title: String::new(),
            size: Size {
                width: 0.0,
                height: 0.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            fullscreen: false,
            hidden: false,
            at_front: false,
            focused: false,
        }
    }

    pub fn apply(&mut self, mutation: Mutation) {
        match mutation {
            Mutation::Title(title) => self.title = title,
            Mutation::Size(size) => self.size = size,
            Mutation::Position(position) => self.position = position,
            Mutation::Fullscreen(fullscreen) => self.fullscreen = fullscreen,
            Mutation::Hidden(hidden) => self.hidden = hidden,
            Mutation::AtFront(at_front) => self.at_front = at_front,
            Mutation::Focused(focused) => self.focused = focused,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: need to share this enum w/ process crate, will prob make a sep lib crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mutation {
    Title(String),
    Size(Size),
    Position(Position),
    Fullscreen(bool),
    Hidden(bool),
    AtFront(bool),
    Focused(bool),
}
