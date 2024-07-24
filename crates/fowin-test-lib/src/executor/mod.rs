use std::{
    borrow::Borrow,
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
pub use fowin_executor::FowinExecutor;
#[cfg(feature = "winit")]
pub use winit_executor::WinitExecutor;

mod fowin_executor;
#[cfg(feature = "winit")]
mod winit_executor;

pub trait Executor {
    // Originally, this function validated the entire state, but it turns out we can't reliably guarantee
    // window state won't be mutated by the OS. What we can guarantee is that the performed operaton (if
    // no error) will, in fact, occur.
    fn window_props(&self, id: u32) -> Result<impl WindowProps, ExecutionError>;

    fn execute(&mut self, step: &Step) -> Result<(), ExecutionError>;

    fn validate(&self, id: u32, mutation: &Mutation) -> Result<(), ExecutionError> {
        let window = self.window_props(id)?;
        match mutation {
            Mutation::Title(title) => {
                let expected_title = window.title()?;
                if title != &expected_title {
                    Err(ValidationError::TitleMismatch {
                        expected: expected_title,
                        actually: title.to_owned(),
                    })?;
                }
            }
            Mutation::Size(size) => {
                let expected_size = window.size()?;
                if size != &expected_size {
                    Err(ValidationError::SizeMismatch {
                        expected: expected_size,
                        actually: size.to_owned(),
                    })?
                }
            }
            Mutation::Position(position) => {
                let expected_position = window.position()?;
                if position != &expected_position {
                    Err(ValidationError::PositionMismatch {
                        expected: expected_position,
                        actually: position.to_owned(),
                    })?
                }
            }
            Mutation::Fullscreen(fullscreen) => {
                let expected_fullscreen = window.is_fullscreen()?;
                if fullscreen != &expected_fullscreen {
                    Err(ValidationError::FullscreenMismatch {
                        expected: expected_fullscreen,
                        actually: fullscreen.to_owned(),
                    })?
                }
            }
            Mutation::Hidden(hidden) => {
                let expected_hidden = window.is_hidden()?;
                if hidden != &expected_hidden {
                    Err(ValidationError::HiddenMismatch {
                        expected: expected_hidden,
                        actually: hidden.to_owned(),
                    })?
                }
            }
            Mutation::AtFront(_) => {
                // TODO
                todo!()
            }
            Mutation::Focused(_) => {
                // TODO
                todo!()
            }
        }

        Ok(())
    }
}

pub trait WindowProps {
    fn title(&self) -> Result<String, ExecutionError>;

    fn size(&self) -> Result<Size, ExecutionError>;

    fn position(&self) -> Result<Position, ExecutionError>;

    fn is_fullscreen(&self) -> Result<bool, ExecutionError>;

    fn is_hidden(&self) -> Result<bool, ExecutionError>;

    fn is_at_front(&self) -> Result<bool, ExecutionError>;

    fn is_focused(&self) -> Result<bool, ExecutionError>;
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
