mod chronology;
mod executor;
mod operation;
mod state;
mod timeline;

// TODO: decide what needs to be public
#[cfg(feature = "winit")]
pub use executor::WinitExecutor;
pub use executor::{
    encode_title, ExecutionError, Executor, FowinExecutor, ValidationError, WindowProps,
};
pub use state::{Mutation, Position, Size, State};
pub use timeline::{Action, Step, Timeline};
