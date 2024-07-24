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
    encode_title,
    state::Mutation,
    timeline::{Action, ExecScope, Step},
    ExecutionError, Executor, Position, Size, State, Timeline, WindowProps,
};

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

            // std::thread::sleep(Duration::from_secs(3));
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

                    self.validate(step.id, &mutation)?;
                    executor.validate(step.id, &mutation)?;
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

    fn window_props(&self, id: u32) -> Result<impl WindowProps, ExecutionError> {
        self.windows
            .get(&id)
            .ok_or(ExecutionError::UnknownWindowId(id))
    }
}

impl WindowProps for &Window {
    fn title(&self) -> Result<String, ExecutionError> {
        Ok(Window::title(self)?)
    }

    fn size(&self) -> Result<Size, ExecutionError> {
        Ok(Window::size(self).map(|size| Size {
            width: size.width,
            height: size.height,
        })?)
    }

    fn position(&self) -> Result<Position, ExecutionError> {
        Ok(Window::position(self).map(|position| Position {
            x: position.x,
            y: position.y,
        })?)
    }

    fn is_fullscreen(&self) -> Result<bool, ExecutionError> {
        Ok(Window::is_fullscreen(self)?)
    }

    fn is_hidden(&self) -> Result<bool, ExecutionError> {
        println!("FOWIN: {:?}", Window::is_minimized(self));
        Ok(Window::is_minimized(self)?)
    }

    fn is_at_front(&self) -> Result<bool, ExecutionError> {
        // TODO: is it possible that we can add an is_at_front to macos?
        Ok(todo!())
    }

    fn is_focused(&self) -> Result<bool, ExecutionError> {
        Ok(Window::is_focused(self)?)
    }
}
