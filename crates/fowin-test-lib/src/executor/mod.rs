use std::{
    error::Error,
    fmt::{self, Debug},
};

use fowin::WindowError;
use log::info;

use crate::{state::Mutation, timeline::Step, Position, Size};
#[cfg(feature = "binary_executor")]
pub use binary_executor::{BinaryExecutor, IpcError, Request, RequestProp, Response};
pub use fowin_executor::FowinExecutor;
#[cfg(feature = "winit_executor")]
pub use winit_executor::WinitExecutor;

#[cfg(feature = "binary_executor")]
mod binary_executor;
mod fowin_executor;
#[cfg(feature = "winit_executor")]
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
                let actual_title = window.title()?;
                if title != &actual_title {
                    Err(ValidationError::TitleMismatch {
                        expected: title.to_owned(),
                        actually: actual_title,
                    })?;
                }
            }
            Mutation::Size(size) => {
                let actual_size = window.size()?;
                if size != &actual_size {
                    Err(ValidationError::SizeMismatch {
                        expected: size.to_owned(),
                        actually: actual_size,
                    })?
                }
            }
            Mutation::Position(position) => {
                let actual_position = window.position()?;
                if position != &actual_position {
                    Err(ValidationError::PositionMismatch {
                        expected: position.to_owned(),
                        actually: actual_position,
                    })?
                }
            }
            Mutation::Fullscreen(fullscreen) => {
                let actual_fullscreen = window.is_fullscreen()?;
                if fullscreen != &actual_fullscreen {
                    Err(ValidationError::FullscreenMismatch {
                        expected: fullscreen.to_owned(),
                        actually: actual_fullscreen,
                    })?
                }
            }
            Mutation::Hide(hidden) => {
                let actual_hidden = window.is_hidden()?;
                if hidden != &actual_hidden {
                    Err(ValidationError::HiddenMismatch {
                        expected: hidden.to_owned(),
                        actually: actual_hidden,
                    })?
                }
            }
            Mutation::Minimize(minimized) => {
                let actual_minimized = window.is_hidden()?;
                if minimized != &actual_minimized {
                    Err(ValidationError::HiddenMismatch {
                        expected: minimized.to_owned(),
                        actually: actual_minimized,
                    })?
                }
            }
            Mutation::BringToFront => {
                // TODO
                todo!()
            }
            Mutation::Focus => {
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

    fn is_minimized(&self) -> Result<bool, ExecutionError>;

    fn is_at_front(&self) -> Result<bool, ExecutionError>;

    fn is_focused(&self) -> Result<bool, ExecutionError>;
}

// TODO: maybe in the future we can use window properties
// The only way we can check if two unique windows are the same is by title comparison. Thus,
// we use a custom title format that combines the unique ID w/ the actual (mutated) title.
pub fn encode_title(namespace: &Option<String>, id: u32, title: &str) -> String {
    match namespace {
        Some(namespace) => format!("{namespace}-{id}: {title}"),
        None => format!("{id}: {title}"),
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    UnknownWindowId(u32),
    UnsupportedOperation(String),
    Validation(ValidationError),
    Fowin(fowin::WindowError),
    #[cfg(feature = "binary_executor")]
    Ipc(IpcError),
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
            #[cfg(feature = "binary_executor")]
            ExecutionError::Ipc(err) => write!(f, "{}", err),
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
    MinimizedMismatch {
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
                    "mismatched minimized, expected `{}`, got `{}`",
                    expected, actually
                )
            }
            ValidationError::MinimizedMismatch { expected, actually } => {
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
