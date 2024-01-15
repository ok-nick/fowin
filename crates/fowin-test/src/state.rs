use std::collections::HashMap;

use rand::{distributions::Standard, prelude::Distribution, Rng};
use serde::{Deserialize, Serialize};

// TODO: temp for serde impls
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

// TODO: can impl this without a hashmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub title: String,
    pub size: Size,
    pub position: Position,
    pub fullscreen: bool,
    pub hidden: bool,
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
            // TODO: these properties may vary, include them in state?
            at_front: false,
            focused: false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
