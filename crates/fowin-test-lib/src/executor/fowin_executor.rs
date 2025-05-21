use std::{collections::HashMap, fmt::Debug};

use fowin::Window;

use crate::{
    executor::{encode_title, ExecutionError, Executor, WindowProps},
    state::Mutation,
    timeline::{Action, ExecScope, Step},
    Position, Size, Timeline,
};

#[derive(Debug, Default)]
pub struct FowinExecutor {
    windows: HashMap<u32, Window>,
    namespace: Option<String>,
}

impl FowinExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_namespace<T: Into<String>>(namespace: T) -> Self {
        Self {
            windows: HashMap::new(),
            namespace: Some(namespace.into()),
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

            // std::thread::sleep(std::time::Duration::from_secs(3));
            println!("VALIDATING");

            match step.action {
                // If it's terminated, there's nothing to validate.
                Action::Terminate => {}
                Action::Spawn(mut state) => {
                    self.cache_window(step.id, &state.title)?;

                    state.title = encode_title(&self.namespace, step.id, &state.title);
                    states.insert(step.id, state);
                }
                Action::Mutate(mut mutation) => {
                    let state = states.get_mut(&step.id).unwrap();
                    state.apply(mutation.clone());

                    // TODO: this is a little bit of a hack, the title gets encoded before being set on the window,
                    //       so to validate it we need it in its encoded state. Not a huge fan of having it here, can
                    //       always move it into the validate function
                    if let Mutation::Title(ref mut title) = mutation {
                        *title = encode_title(&self.namespace, step.id, title)
                    }

                    println!("VALIDATING 1");
                    self.validate(step.id, &mutation)?;
                    println!("VALIDATING 2");
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

        let title = encode_title(&self.namespace, id, title);
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
                    Mutation::Title(_) => Err(ExecutionError::UnsupportedOperation(
                        "fowin set title".to_owned(),
                    )),
                    Mutation::Size(size) => {
                        window.resize((*size).into())?;
                        Ok(())
                    }
                    Mutation::Position(position) => {
                        window.reposition((*position).into())?;
                        Ok(())
                    }
                    Mutation::Fullscreen(fullscreen) => {
                        match fullscreen {
                            true => window.fullscreen()?,
                            false => window.unfullscreen()?,
                        }
                        Ok(())
                    }
                    Mutation::Hide(hidden) => {
                        match hidden {
                            true => window.hide()?,
                            false => window.show()?,
                        }
                        Ok(())
                    }
                    Mutation::Minimize(minimize) => {
                        match minimize {
                            true => window.minimize()?,
                            false => window.unminimize()?,
                        }
                        Ok(())
                    }
                    Mutation::BringToFront => {
                        window.bring_to_front()?;
                        Ok(())
                    }
                    Mutation::Focus => {
                        window.focus()?;
                        Ok(())
                    }
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
        Ok(Window::is_hidden(self)?)
    }

    fn is_minimized(&self) -> Result<bool, ExecutionError> {
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

impl From<Size> for fowin::Size {
    fn from(size: Size) -> Self {
        fowin::Size {
            width: size.width,
            height: size.height,
        }
    }
}

impl From<Position> for fowin::Position {
    fn from(position: Position) -> Self {
        fowin::Position {
            x: position.x,
            y: position.y,
        }
    }
}
