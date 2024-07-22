use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
};

use fowin_test_lib::{Action, Command, Mutation, State};
use interprocess::local_socket::{
    traits::Stream as StreamExt, GenericNamespaced, Stream, ToNsName,
};
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event_loop::EventLoop,
    window::{Fullscreen, Window, WindowBuilder},
};

// fs::write("/Users/nicky/Documents/repos/fowin/test.txt", id.as_bytes()).unwrap();
fn main() {
    let id = env::args().nth(1).unwrap();
    let stream = Stream::connect(id.to_ns_name::<GenericNamespaced>().unwrap()).unwrap();

    let mut json = serde_json::to_string::<Result<(), ()>>(&Ok(())).unwrap();
    json.push('\n');

    let mut reader = BufReader::new(stream);
    reader.get_mut().write_all(&json.into_bytes()).unwrap();

    let mut windows: HashMap<String, Window> = HashMap::new();
    let mut buffer = String::new();

    // TODO: listen for events on event loop and report back when operation is completed
    let event_loop = EventLoop::new().unwrap();
    loop {
        // A new command is sent only when the last command has finished (reported a result).
        match reader.read_line(&mut buffer) {
            Ok(bytes) if bytes > 0 => {
                println!("yerp {bytes} | {buffer}");
                let command: Command = serde_json::from_str(&buffer).unwrap();
                match command.action {
                    Action::Spawn => {
                        let state = State::new();
                        let window = WindowBuilder::new()
                            .with_title(state.title)
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
                            })
                            .build(&event_loop)
                            .unwrap();
                        window.set_minimized(state.hidden);
                        // TODO: at_front, focused

                        windows.insert(command.id, window);
                    }
                    Action::Terminate => {
                        todo!()
                    }
                    Action::Mutate(mutation) => {
                        let window = windows.get_mut(&command.id).unwrap();
                        match mutation {
                            Mutation::Title(title) => window.set_title(&title),
                            Mutation::Size(size) => {
                                let size = window.request_inner_size(LogicalSize {
                                    width: size.width,
                                    height: size.height,
                                });
                            }
                            Mutation::Position(position) => {
                                window.set_outer_position(LogicalPosition {
                                    x: position.x,
                                    y: position.y,
                                })
                            }
                            Mutation::Fullscreen(fullscreen) => {
                                window.set_fullscreen(match fullscreen {
                                    true => Some(Fullscreen::Borderless(None)),
                                    false => None,
                                })
                            }
                            Mutation::Hidden(hidden) => window.set_minimized(hidden),
                            Mutation::AtFront(at_front) => todo!(),
                            Mutation::Focused(focused) => todo!(),
                        }
                    }
                }
            }
            Ok(_) => {}
            Err(_) => todo!(),
        }

        buffer.clear();
    }
}
