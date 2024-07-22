use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Debug},
    io::{BufRead, BufReader, Read, Write},
    process::{self, Child},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use fowin::{Window, WindowError};
use interprocess::local_socket::{
    traits::Listener as ListenerExt, GenericNamespaced, Listener, ListenerOptions, ToNsName,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    state::Mutation,
    timeline::{Action, ExecScope, Step},
    Position, Size, State, Timeline,
};

pub trait Executor {
    fn validate(&self, id: u32, state: &State) -> Result<(), ExecutionError>;

    fn execute(&mut self, step: &Step) -> Result<(), ExecutionError>;
}

#[derive(Debug)]
pub struct FowinExecutor {
    windows: HashMap<u32, Window>,
}

impl FowinExecutor {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    pub fn execute_all<E: Executor>(
        &mut self,
        executor: &mut E,
        timeline: Timeline,
    ) -> Result<(), ExecutionError> {
        let mut states = HashMap::new();
        for step in timeline.into_steps() {
            println!("SENT {:?}", step);

            match step.scope {
                ExecScope::Fowin => {
                    self.execute(&step)?;
                }
                ExecScope::External => {
                    executor.execute(&step)?;
                }
            }

            std::thread::sleep(Duration::from_secs(3));
            println!("VALIDATING");

            match step.action {
                // If it's terminated, there's nothing to validate.
                Action::Terminate => {}
                Action::Spawn(mut state) => {
                    self.cache_window(step.id, &state.title)?;

                    state.title = encode_title(step.id, &state.title);
                    states.insert(step.id, state);
                }
                Action::Mutate(mutation) => {
                    let state = states.get_mut(&step.id).unwrap();
                    state.apply(mutation.to_owned());

                    self.validate(step.id, state)?;
                    executor.validate(step.id, state)?;
                }
            }
        }

        Ok(())
    }

    fn cache_window(&mut self, id: u32, title: &str) -> Result<(), ExecutionError> {
        if self.windows.contains_key(&id) {
            return Ok(());
        }

        let title = encode_title(id, title);
        for window in fowin::iter_windows() {
            match window {
                Ok(window) => {
                    if window.title().unwrap() == title {
                        self.windows.insert(id, window);
                    }
                }
                Err(err) => {
                    // TODO: need to fix the arbitraryerror stuff
                    // println!("ERR: {:?}", err);
                }
            }
        }

        Ok(())
    }
}

impl Executor for FowinExecutor {
    // TODO: need to filter certain properties based on others, e.g. if minimized don't verify size/fullscreen, etc.
    fn validate(&self, id: u32, state: &State) -> Result<(), ExecutionError> {
        let window = self
            .windows
            .get(&id)
            .ok_or(ExecutionError::UnknownWindowId(id))?;

        let actual_state = State {
            title: window.title()?,
            size: {
                let size = window.size()?;
                Size {
                    width: size.width,
                    // TODO: it's including the top bar, is this guaranteed to be 28?
                    height: size.height - 28.0,
                }
            },
            position: {
                // TODO: need to fix
                // let position = window.position()?;
                // Position {
                //     x: position.x,
                //     y: position.y,
                // }
                Position { x: 0.0, y: 0.0 }
            },
            fullscreen: window.is_fullscreen()?,
            hidden: window.is_minimized().unwrap(),
            at_front: false, // TODO:
            focused: false,  // TODO
        };

        actual_state.validate(state)?;

        Ok(())
    }

    fn execute(&mut self, step: &Step) -> Result<(), ExecutionError> {
        match &step.action {
            Action::Mutate(mutation) => {
                let window = self.windows.get(&step.id).unwrap();
                match mutation {
                    Mutation::Title(_) => todo!(),
                    Mutation::Size(_) => todo!(),
                    Mutation::Position(_) => todo!(),
                    Mutation::Fullscreen(_) => todo!(),
                    // TODO: add minimize mutation
                    Mutation::Hidden(hidden) => {
                        match hidden {
                            true => window.minimize().unwrap(),
                            false => window.unminimize().unwrap(),
                        }

                        Ok(())
                    }
                    Mutation::AtFront(_) => todo!(),
                    Mutation::Focused(_) => todo!(),
                }
            }
            Action::Spawn(_) => Err(ExecutionError::UnsupportedOperation(
                "fowin spawn window".to_owned(),
            )),
            // TODO: I believe this can be done on macos via kAXCloseButtonAttribute and kAXPressButton
            Action::Terminate => Err(ExecutionError::UnsupportedOperation(
                "fowin terminate window".to_owned(),
            )),
        }
    }
}

// The only way we can check if two unique windows are the same is by title comparison. Thus,
// we use a custom title format that combines the unique ID w/ the actual (mutated) title.
pub fn encode_title(id: u32, title: &str) -> String {
    format!("{id}: {title}")
}

#[derive(Debug)]
pub enum ExecutionError {
    UnknownWindowId(u32),
    UnsupportedOperation(String),
    Validation(ValidationError),
    Fowin(fowin::WindowError),
}

impl Error for ExecutionError {}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::UnknownWindowId(id) => {
                write!(f, "attempted to operate on unknown window id `{}`", id)
            }
            ExecutionError::UnsupportedOperation(operation) => {
                write!(
                    f,
                    "attempted to execute unsupported operation `{}`",
                    operation
                )
            }
            ExecutionError::Validation(validation_error) => fmt::Display::fmt(validation_error, f),
            ExecutionError::Fowin(window_err) => fmt::Display::fmt(window_err, f),
        }
    }
}

impl From<ValidationError> for ExecutionError {
    fn from(err: ValidationError) -> Self {
        Self::Validation(err)
    }
}

impl From<WindowError> for ExecutionError {
    fn from(err: WindowError) -> Self {
        Self::Fowin(err)
    }
}

#[derive(Debug)]
pub enum ValidationError {
    TitleMismatch {
        expected: String,
        actually: String,
    },
    SizeMismatch {
        expected: Size,
        actually: Size,
    },
    PositionMismatch {
        expected: Position,
        actually: Position,
    },
    FullscreenMismatch {
        expected: bool,
        actually: bool,
    },
    HiddenMismatch {
        expected: bool,
        actually: bool,
    },
    AtFrontMismatch {
        expected: bool,
        actually: bool,
    },
    FocusedMismatch {
        expected: bool,
        actually: bool,
    },
}

impl Error for ValidationError {}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::TitleMismatch { expected, actually } => {
                write!(
                    f,
                    "mismatched title, expected `{}`, got `{}`",
                    expected, actually
                )
            }
            ValidationError::SizeMismatch { expected, actually } => {
                write!(
                    f,
                    "mismatched size, expected `{:?}`, got `{:?}`",
                    expected, actually
                )
            }
            ValidationError::PositionMismatch { expected, actually } => {
                write!(
                    f,
                    "mismatched position, expected `{:?}`, got `{:?}`",
                    expected, actually
                )
            }
            ValidationError::FullscreenMismatch { expected, actually } => {
                write!(
                    f,
                    "mismatched fullscreen, expected `{}`, got `{}`",
                    expected, actually
                )
            }
            ValidationError::HiddenMismatch { expected, actually } => {
                write!(
                    f,
                    "mismatched hidden, expected `{}`, got `{}`",
                    expected, actually
                )
            }
            ValidationError::AtFrontMismatch { expected, actually } => {
                write!(
                    f,
                    "mismatched at front, expected `{}`, got `{}`",
                    expected, actually
                )
            }
            ValidationError::FocusedMismatch { expected, actually } => {
                write!(
                    f,
                    "mismatched focus, expected `{}`, got `{}`",
                    expected, actually
                )
            }
        }
    }
}
