use serde::{Deserialize, Serialize};

use crate::executor::ValidationError;

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
    pub minimized: bool,
    // TODO: these two props can't be guaranteed, a new window will be focused/at_front..
    //       can introduce rules to define this behavior?
    pub at_front: bool,
    pub focused: bool,
}

impl State {
    pub fn initial() -> State {
        State {
            title: String::from("fowin window"),
            size: Size {
                width: 100.0,
                height: 100.0,
            },
            // TODO: account for top bar on macos
            position: Position { x: 0.0, y: 25.0 },
            fullscreen: false,
            hidden: false,
            minimized: false,
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
            Mutation::Hide(hidden) => self.hidden = hidden,
            Mutation::Minimize(minimized) => self.minimized = minimized,
            Mutation::BringToFront => self.at_front = true,
            Mutation::Focus => self.focused = true,
        }
    }

    pub fn validate(&self, expected: &State, mutation: &Mutation) -> Result<(), ValidationError> {
        match mutation {
            Mutation::Title(_) => {
                if self.title != expected.title {
                    return Err(ValidationError::TitleMismatch {
                        expected: expected.title.clone(),
                        actually: self.title.clone(),
                    });
                }
            }
            Mutation::Size(_) => {
                if self.size != expected.size {
                    return Err(ValidationError::SizeMismatch {
                        expected: expected.size,
                        actually: self.size,
                    });
                }
            }
            Mutation::Position(_) => {
                if self.position != expected.position {
                    return Err(ValidationError::PositionMismatch {
                        expected: expected.position,
                        actually: self.position,
                    });
                }
            }
            Mutation::Fullscreen(_) => {
                if self.fullscreen != expected.fullscreen {
                    return Err(ValidationError::FullscreenMismatch {
                        expected: expected.fullscreen,
                        actually: self.fullscreen,
                    });
                }
            }
            Mutation::Hide(_) => {
                if self.hidden != expected.hidden {
                    return Err(ValidationError::HiddenMismatch {
                        expected: expected.hidden,
                        actually: self.hidden,
                    });
                }
            }
            Mutation::Minimize(_) => {
                if self.minimized != expected.minimized {
                    return Err(ValidationError::MinimizedMismatch {
                        expected: expected.minimized,
                        actually: self.minimized,
                    });
                }
            }
            Mutation::BringToFront => {
                // TODO
            }
            Mutation::Focus => {
                // TODO
            }
        }

        Ok(())
    }
}

impl Default for State {
    fn default() -> Self {
        Self::initial()
    }
}

// TODO: need to share this enum w/ process crate, will prob make a sep lib crate
// TODO: make unminimize/minimize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mutation {
    Title(String),
    Size(Size),
    Position(Position),
    Fullscreen(bool),
    Hide(bool),
    Minimize(bool),
    BringToFront,
    Focus,
}
