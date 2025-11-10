use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::pump_events::EventLoopExtPumpEvents,
    window::{Fullscreen, Window, WindowId},
};

use crate::{
    executor::{encode_title, ExecutionError, Executor, WindowProps},
    state::Mutation,
    timeline::{Action, ExecScope, Step},
    Position, Size,
};

#[derive(Debug)]
pub struct WinitExecutor {
    app: App,
    event_loop: EventLoop<Step>,
    receiver: Receiver<()>,
}

impl WinitExecutor {
    pub fn new() -> Self {
        Self::new_with_namespace(None)
    }

    pub fn with_namespace<T: Into<String>>(namespace: T) -> Self {
        Self::new_with_namespace(Some(namespace.into()))
    }

    fn new_with_namespace(namespace: Option<String>) -> Self {
        let (sender, receiver) = mpsc::channel();

        let mut app = App {
            sender,
            windows: HashMap::new(),
            namespace,
        };

        // Pump the initialization events.
        let mut event_loop = EventLoop::<Step>::with_user_event().build().unwrap();
        event_loop.pump_app_events(Some(Duration::ZERO), &mut app);

        Self {
            app,
            event_loop,
            receiver,
        }
    }
}

impl Default for WinitExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for WinitExecutor {
    // In a LocalExecutor, everything runs in the local program, so we don't need to map
    // window ids to separate processes as in the case of the BinaryExecutor.
    //
    // Note that we ignore the ExecScope here because we want to pump app events even if
    // fowin executes a window operation so that they apply immediately.
    fn execute(&mut self, step: &Step) -> Result<(), ExecutionError> {
        if let ExecScope::External = step.scope {
            // self.receiver.try_iter().for_each(drop);
            println!("STARTED");

            // Send the new user event.
            self.event_loop
                .create_proxy()
                .send_event(step.to_owned())
                .unwrap();

            self.event_loop
                .pump_app_events(Some(Duration::ZERO), &mut self.app);

            println!("PENDING");
            self.receiver.recv().unwrap();
        }

        // TODO: doesn't work properly without this. I wonder if it would be
        //       better if we had a continuous run loop? What's a reliable number to wait?
        //       maybe if a test fails we can rerun it with longer delay? Can we listen
        //       to events and proceed after a timeout or when they respond? I found that
        //       anything less than 2ms and it frequently fails.
        // std::thread::sleep(Duration::from_millis(1000));

        // Apply the new changes caused by the event. This is quite the hack but it works
        // quite well. Ideally we'd yield until the event is fully applied, but I don't
        // believe that's possible.
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(10) {
            self.event_loop
                .pump_app_events(Some(Duration::ZERO), &mut self.app);
        }

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
        let size = self.outer_size().to_logical::<i32>(self.scale_factor());
        Ok(Size {
            width: size.width.into(),
            height: size.height.into(),
        })
    }

    fn position(&self) -> Result<Position, ExecutionError> {
        let position = self
            .outer_position()
            .map_err(|_| ExecutionError::UnsupportedOperation("winit window position".to_owned()))?
            .to_logical::<i32>(self.scale_factor());
        Ok(Position {
            x: position.x.into(),
            y: position.y.into(),
        })
    }

    fn is_fullscreen(&self) -> Result<bool, ExecutionError> {
        Ok(self.fullscreen().is_some())
    }

    fn is_hidden(&self) -> Result<bool, ExecutionError> {
        println!("{:?}", self.is_visible());
        self.is_visible()
            .map(|visible| !visible)
            .ok_or(ExecutionError::UnsupportedOperation(
                "winit is_visible".to_owned(),
            ))
    }

    fn is_minimized(&self) -> Result<bool, ExecutionError> {
        Window::is_minimized(self).ok_or(ExecutionError::UnsupportedOperation(
            "winit is_minimized".to_owned(),
        ))
    }

    fn is_at_front(&self) -> Result<bool, ExecutionError> {
        // TODO: set WindowLevel::AlwaysOnTop, then WindowLevel::Normal?
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
    namespace: Option<String>,
}

impl ApplicationHandler<Step> for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    // TODO: some events like request_inner_size are queued and the channel shouldn't be returned
    //       until we know it's been executed
    //       same with fullscreening, which takes time to transition
    fn user_event(&mut self, event_loop: &ActiveEventLoop, step: Step) {
        println!("RECEIVED {:?}", step);

        match &step.action {
            Action::Spawn(state) => {
                let window = event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title(encode_title(&self.namespace, step.id, &state.title))
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
                        // The macOS accessibility API (used in fowin) does two interesting things:
                        // 1. Includes the title bar size when getting and setting the window size.
                        // 2. Sets and gets logical sizes rather than physical.
                        let scale_factor = window.scale_factor();
                        let title_bar_size =
                            window.outer_size().to_logical::<f64>(scale_factor).height
                                - window.inner_size().to_logical::<f64>(scale_factor).height;

                        let result = window.request_inner_size(LogicalSize {
                            width: size.width,
                            height: size.height - title_bar_size,
                        });
                        println!("RESULT IS {:?}", result);
                    }
                    Mutation::Position(position) => window.set_outer_position(LogicalPosition {
                        x: position.x,
                        y: position.y,
                    }),
                    Mutation::Fullscreen(fullscreen) => window.set_fullscreen(match fullscreen {
                        true => Some(Fullscreen::Borderless(None)),
                        false => None,
                    }),
                    // TODO: if we call set_visible, it will call NSWindow.orderOut, which makes the AXUIElementRef on
                    //       macOS invalid. Therefore we must use another method of hiding a window (currently minimizing)
                    //       although not a huge fan. Aerospace moves windows to the bottom right corner. There may also
                    //       be a private API that can be used in fowin to detect hidden windows, although not too ideal
                    Mutation::Hide(hidden) => window.set_minimized(*hidden),
                    Mutation::Minimize(minimized) => window.set_minimized(*minimized),
                    // TODO: same as focus window? there isn't a way to query if the window is at the front.
                    Mutation::BringToFront => todo!(),
                    Mutation::Focus => window.focus_window(),
                    Mutation::Title(title) => {
                        window.set_title(&encode_title(&self.namespace, step.id, title))
                    }
                }
            }
        }

        // if !matches!(&step.action, Action::Mutate(Mutation::Size(..))) {
        // Alert that we finished running the step.
        self.sender.send(()).unwrap();
        // }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        println!("EVENT: {:?}", event);

        // if let WindowEvent::Resized(..) = event {
        // self.sender.send(()).unwrap();
        // }
    }
}
