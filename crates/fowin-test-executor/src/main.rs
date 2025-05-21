use fowin_test_lib::{
    executor::{
        ExecutionError, Executor, IpcError, Request, RequestProp, Response, WindowProps,
        WinitExecutor,
    },
    Mutation,
};
use std::{
    env,
    io::{BufRead, BufReader, Write},
};

use interprocess::local_socket::{
    traits::Stream as StreamExt, GenericNamespaced, Stream, ToNsName,
};

// TODO: this whole thing should be in fowin-test?
fn main() -> Result<(), ExecutionError> {
    let id = env::args().nth(1).expect("Expected namespace as argument");
    let stream = Stream::connect(id.to_ns_name::<GenericNamespaced>()?)?;

    let mut stream = BufReader::new(stream);

    let response = serde_json::to_string(&Response::Init)?;
    stream.get_mut().write_all(&response.into_bytes())?;

    let mut executor = WinitExecutor::new();

    let mut buffer = String::new();
    loop {
        let bytes = stream.read_line(&mut buffer)?;
        if bytes > 0 {
            let request = serde_json::from_str(&buffer)?;
            let response = handle_request(&mut executor, request);
            let response = serde_json::to_string(&response)?;
            stream.get_mut().write_all(&response.into_bytes())?;
        }

        buffer.clear();
    }
}

fn handle_request(executor: &mut WinitExecutor, request: Request) -> Result<Response, IpcError> {
    match request {
        Request::Step(step) => executor.execute(&step)?,
        Request::Prop { id, prop } => {
            let props = executor.window_props(id)?;
            return Ok(match prop {
                RequestProp::Title => Response::Property(Mutation::Title(props.title()?)),
                RequestProp::Size => Response::Property(Mutation::Size(props.size()?)),
                RequestProp::Position => Response::Property(Mutation::Position(props.position()?)),
                RequestProp::IsFullscreen => {
                    Response::Property(Mutation::Fullscreen(props.is_fullscreen()?))
                }
                RequestProp::IsHidden => Response::Property(Mutation::Hide(props.is_hidden()?)),
                RequestProp::IsMinimized => {
                    Response::Property(Mutation::Minimize(props.is_minimized()?))
                }
                // TODO: these bottom two are incorrectly handled
                RequestProp::IsAtFront => Response::Property(Mutation::BringToFront),
                RequestProp::IsFocused => Response::Property(Mutation::Focus),
            });
        }
        Request::Init => {}
    }

    Ok(Response::Success)
}
