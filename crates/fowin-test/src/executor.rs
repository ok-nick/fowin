use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use fowin_test_lib::{
    Action, ExecutionError, Executor, Mutation, Position, Size, State, Step, ValidationError,
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::StartCause,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    platform::pump_events::EventLoopExtPumpEvents,
    window::{Fullscreen, Window},
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
    fn validate(&self, id: u32, state: &State) -> Result<(), ExecutionError> {
        let window = self
            .app
            .windows
            .get(&id)
            .ok_or(ExecutionError::UnknownWindowId(id))?;

        let actual_state = State {
            title: window.title(),
            size: {
                let size = window.inner_size();
                Size {
                    width: size.width.into(),
                    height: size.height.into(),
                }
            },
            position: {
                let position = window.outer_position().unwrap();
                Position {
                    x: position.x.into(),
                    y: position.y.into(),
                }
            },
            // fullscreen: matches!(window.fullscreen().unwrap(), Fullscreen::Borderless(_)),
            fullscreen: false, // TODO
            hidden: window.is_minimized().unwrap(),
            at_front: false, // TODO:
            focused: false,  // TODO
        };

        actual_state.validate(state)?;

        Ok(())
    }

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
        //       better if we had a continuous run loop?
        std::thread::sleep(Duration::from_millis(10));

        // Apply the new changes caused by the event.
        self.event_loop
            .pump_app_events(Some(Duration::ZERO), &mut self.app);

        Ok(())
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
                // println!("{:?}", state);
                let window = event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title(fowin_test_lib::encode_title(step.id, &state.title))
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
                    Mutation::AtFront(at_front) => todo!(),
                    Mutation::Focused(focused) => todo!(),
                    // TODO: since we use the title to identify via id it for fowin, then maybe we should
                    //       use a combination if id - title
                    // Mutation::Title(title) => window.set_title(&title),
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

// TODO: all it needs to be is a Result<(), E>, no need for separate enum
// #[derive(Debug, Serialize, Deserialize)]
// pub enum Response {
//     Err(),
// }

// TODO: need some kind of IPC, preferably simple:
// * iceoryx (extremely new and rather complex)
// * ipc-channel (only recently maintained)
// * interprocess (simple)
// #[derive(Debug)]
// pub struct BinaryExecutor {
//     inner: Child,
//     listener: Listener,
//     ready: AtomicBool,
// }

// impl BinaryExecutor {
//     pub fn new(mut spawner: process::Command) -> Result<BinaryExecutor, ()> {
//         let id = Uuid::new_v4().to_string();
//         let spawner = spawner.arg(&id);

//         let listener = ListenerOptions::new()
//             // TODO: check if GenericNamespaced::is_supported
//             .name(id.to_ns_name::<GenericNamespaced>().unwrap())
//             .create_sync()
//             .unwrap();
//         let process = spawner.spawn().unwrap();

//         Ok(BinaryExecutor {
//             inner: process,
//             listener,
//             ready: AtomicBool::new(false),
//         })
//     }

//     fn wait_until_ready(&self) -> Result<(), ()> {
//         if self.ready.load(Ordering::Relaxed) {
//             Ok(())
//         } else {
//             let stream = self.listener.accept().unwrap();
//             let mut reader = BufReader::new(stream);

//             let mut json = String::new();
//             reader.read_line(&mut json).unwrap();

//             // TODO: output should be result
//             match serde_json::from_str::<Result<(), ()>>(&json).unwrap() {
//                 Ok(_) => {
//                     self.ready.store(true, Ordering::Relaxed);
//                     Ok(())
//                 }
//                 Err(err) => Err(err),
//             }
//         }
//     }
// }

// impl Executor for BinaryExecutor {
//     //  TODO: unwraps
//     fn execute(&self, command: Command) -> Result<(), ()> {
//         self.wait_until_ready().unwrap();

//         let stream = self.listener.accept().unwrap();
//         let mut reader = BufReader::new(stream);

//         let mut json = serde_json::to_string(&command).unwrap();
//         json.push('\n');
//         println!("WRITING {json}");
//         reader.get_mut().write_all(&json.into_bytes()).unwrap();

//         // TODO: read until timeout?
//         let mut json = String::new();
//         reader.read_line(&mut json).unwrap();

//         // TODO: output response should be a result
//         serde_json::from_str(&json).unwrap()
//     }
// }
