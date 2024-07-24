use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event_loop::{ActiveEventLoop, EventLoop},
    platform::pump_events::EventLoopExtPumpEvents,
    window::{Fullscreen, Window},
};

use crate::{
    encode_title,
    state::Mutation,
    timeline::{Action, Step},
    ExecutionError, Executor, Position, Size, WindowProps,
};

#[derive(Debug)]
pub struct WinitExecutor {
    app: App,
    event_loop: EventLoop<Step>,
    receiver: Receiver<()>,
}

impl WinitExecutor {
    pub fn new() -> WinitExecutor {
        let (sender, receiver) = mpsc::channel();

        let mut app = App {
            sender,
            windows: HashMap::new(),
        };

        // Pump the initialization events.
        let mut event_loop = EventLoop::<Step>::with_user_event().build().unwrap();
        event_loop.pump_app_events(Some(Duration::ZERO), &mut app);

        WinitExecutor {
            app,
            event_loop,
            receiver,
        }
    }
}

impl Executor for WinitExecutor {
    // In a LocalExecutor, everything runs in the local program, so we don't need to map
    // window ids to separate processes as in the case of the BinaryExecutor.
    fn execute(&mut self, step: &Step) -> Result<(), ExecutionError> {
        // Send the new user event.
        self.event_loop
            .create_proxy()
            .send_event(step.to_owned())
            .unwrap();
        self.event_loop
            .pump_app_events(Some(Duration::ZERO), &mut self.app);

        self.receiver.recv().unwrap();

        // TODO: doesn't work properly without this. I wonder if it would be
        //       better if we had a continuous run loop? What's a reliable number to wait?
        //       maybe if a test fails we can rerun it with longer delay?
        std::thread::sleep(Duration::from_millis(50));

        // Apply the new changes caused by the event.
        self.event_loop
            .pump_app_events(Some(Duration::ZERO), &mut self.app);

        Ok(())
    }

    fn window_props(&self, id: u32) -> Result<impl WindowProps, ExecutionError> {
        self.app
            .windows
            .get(&id)
            .ok_or(ExecutionError::UnknownWindowId(id))
    }
}

impl WindowProps for &Window {
    fn title(&self) -> Result<String, ExecutionError> {
        Ok(Window::title(self))
    }

    // TODO: handle physical/logical size consistency
    fn size(&self) -> Result<Size, ExecutionError> {
        let size = self.outer_size();
        Ok(Size {
            width: size.width.into(),
            height: size.height.into(),
        })
    }

    fn position(&self) -> Result<Position, ExecutionError> {
        let position = self.outer_position().map_err(|_| {
            ExecutionError::UnsupportedOperation("winit window position".to_owned())
        })?;
        Ok(Position {
            x: position.x.into(),
            y: position.y.into(),
        })
    }

    fn is_fullscreen(&self) -> Result<bool, ExecutionError> {
        let fullscreen = self
            .fullscreen()
            .ok_or(ExecutionError::UnsupportedOperation(
                "winit fullscreen".to_owned(),
            ))?;
        Ok(matches!(fullscreen, Fullscreen::Borderless(_)))
    }

    fn is_hidden(&self) -> Result<bool, ExecutionError> {
        println!("{:?}", self.is_minimized());
        self.is_minimized()
            .ok_or(ExecutionError::UnsupportedOperation(
                "winit is_minimized".to_owned(),
            ))
    }

    fn is_at_front(&self) -> Result<bool, ExecutionError> {
        // TODO
        Ok(todo!())
    }

    fn is_focused(&self) -> Result<bool, ExecutionError> {
        Ok(self.has_focus())
    }
}

#[derive(Debug)]
struct App {
    windows: HashMap<u32, Window>,
    sender: Sender<()>,
}

impl ApplicationHandler<Step> for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, step: Step) {
        println!("RECEIVED {:?}", step);

        match step.action {
            Action::Spawn(state) => {
                let window = event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title(encode_title(step.id, &state.title))
                            // TODO: use physical?
                            .with_inner_size(LogicalSize {
                                width: state.size.width,
                                height: state.size.height,
                            })
                            .with_position(LogicalPosition {
                                x: state.position.x,
                                y: state.position.y,
                            })
                            // TODO: do exclusive
                            .with_fullscreen(match state.fullscreen {
                                true => Some(Fullscreen::Borderless(None)),
                                false => None,
                            }),
                    )
                    .unwrap();
                window.set_minimized(state.hidden);
                // TODO: at_front, focused

                self.windows.insert(step.id, window);
            }
            Action::Terminate => {
                self.windows.remove(&step.id);
            }
            Action::Mutate(mutation) => {
                let window = self.windows.get_mut(&step.id).unwrap();
                match mutation {
                    Mutation::Size(size) => {
                        let size = window.request_inner_size(LogicalSize {
                            width: size.width,
                            height: size.height,
                        });
                    }
                    Mutation::Position(position) => window.set_outer_position(LogicalPosition {
                        x: position.x,
                        y: position.y,
                    }),
                    Mutation::Fullscreen(fullscreen) => window.set_fullscreen(match fullscreen {
                        true => Some(Fullscreen::Borderless(None)),
                        false => None,
                    }),
                    Mutation::Hidden(hidden) => window.set_minimized(hidden),
                    // TODO: same as focus window?
                    Mutation::AtFront(at_front) => todo!(),
                    Mutation::Focused(focused) => window.focus_window(),
                    Mutation::Title(title) => window.set_title(&encode_title(step.id, &title)),
                    _ => {}
                }
            }
        }

        // Alert that we finished running the step.
        self.sender.send(()).unwrap();
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
    }
}
