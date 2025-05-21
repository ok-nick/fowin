use std::{
    cell::RefCell,
    error::Error,
    fmt::{self, Display},
    io::{self, BufRead, BufReader, Write},
    process::{Child, Command},
};

use interprocess::local_socket::{
    traits::Stream as StreamExt, GenericNamespaced, Stream, ToNsName,
};
use serde::{Deserialize, Serialize};

use crate::{
    executor::{ExecutionError, Executor, WindowProps},
    Mutation, Position, Size, Step,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Step(Step),
    Prop { id: u32, prop: RequestProp },
    Init,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestProp {
    Title,
    Size,
    Position,
    IsFullscreen,
    IsHidden,
    IsMinimized,
    IsAtFront,
    IsFocused,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Property(Mutation),
    Success,
    Init,
}

#[derive(Debug)]
pub struct BinaryExecutor {
    namespace: String,
    process: Child,
    stream: RefCell<BufReader<Stream>>,
}

impl BinaryExecutor {
    pub fn new(namespace: String) -> Result<Self, ExecutionError> {
        let mut stream = BufReader::new(Stream::connect(
            namespace.clone().to_ns_name::<GenericNamespaced>()?,
        )?);

        let process = Command::new("fowin-test-executor")
            .arg(&namespace)
            .spawn()?;

        let mut response = String::new();
        stream.read_line(&mut response)?;
        let response: Response = serde_json::from_str(&response)?;
        if !matches!(response, Response::Init) {
            return Err(IpcError::InitFailure.into());
        }

        Ok(Self {
            process,
            stream: RefCell::new(stream),
            namespace,
        })
    }

    fn request(&self, command: &Request) -> Result<Response, ExecutionError> {
        let mut stream = self.stream.borrow_mut();

        let bytes = serde_json::to_string(command)?.into_bytes();
        stream.get_mut().write_all(&bytes)?;

        let mut buffer = String::new();
        stream.read_line(&mut buffer)?;

        Ok(serde_json::from_str(&buffer)?)
    }
}

impl Executor for BinaryExecutor {
    fn window_props(&self, id: u32) -> Result<impl WindowProps, ExecutionError> {
        Ok(Window { executor: self, id })
    }

    // TODO: add a timeout to fowin executor
    fn execute(&mut self, step: &Step) -> Result<(), ExecutionError> {
        self.request(&Request::Step(step.to_owned())).map(|_| ())
    }
}

#[derive(Debug)]
pub struct Window<'a> {
    executor: &'a BinaryExecutor,
    id: u32,
}

// TODO: lots of boilerplate
impl WindowProps for Window<'_> {
    fn title(&self) -> Result<String, ExecutionError> {
        let request = Request::Prop {
            id: self.id,
            prop: RequestProp::Title,
        };
        match self.executor.request(&request)? {
            Response::Property(Mutation::Title(title)) => Ok(title),
            _ => Err(IpcError::InvalidProp("expected title".to_owned()).into()),
        }
    }

    fn size(&self) -> Result<Size, ExecutionError> {
        let request = Request::Prop {
            id: self.id,
            prop: RequestProp::Size,
        };
        match self.executor.request(&request)? {
            Response::Property(Mutation::Size(size)) => Ok(size),
            _ => Err(IpcError::InvalidProp("expected size".to_owned()).into()),
        }
    }

    fn position(&self) -> Result<Position, ExecutionError> {
        let request = Request::Prop {
            id: self.id,
            prop: RequestProp::Position,
        };
        match self.executor.request(&request)? {
            Response::Property(Mutation::Position(position)) => Ok(position),
            _ => Err(IpcError::InvalidProp("expected position".to_owned()).into()),
        }
    }

    fn is_fullscreen(&self) -> Result<bool, ExecutionError> {
        let request = Request::Prop {
            id: self.id,
            prop: RequestProp::IsFullscreen,
        };
        match self.executor.request(&request)? {
            Response::Property(Mutation::Fullscreen(fullscreen)) => Ok(fullscreen),
            _ => Err(IpcError::InvalidProp("expected fullscreen".to_owned()).into()),
        }
    }

    fn is_hidden(&self) -> Result<bool, ExecutionError> {
        let request = Request::Prop {
            id: self.id,
            prop: RequestProp::IsHidden,
        };
        match self.executor.request(&request)? {
            Response::Property(Mutation::Hide(hidden)) => Ok(hidden),
            _ => Err(IpcError::InvalidProp("expected is_hidden".to_owned()).into()),
        }
    }

    fn is_minimized(&self) -> Result<bool, ExecutionError> {
        let request = Request::Prop {
            id: self.id,
            prop: RequestProp::IsMinimized,
        };
        match self.executor.request(&request)? {
            Response::Property(Mutation::Minimize(minimized)) => Ok(minimized),
            _ => Err(IpcError::InvalidProp("expected is_minimized".to_owned()).into()),
        }
    }

    // TODO: fix is_at_front and is_focused return types
    fn is_at_front(&self) -> Result<bool, ExecutionError> {
        let request = Request::Prop {
            id: self.id,
            prop: RequestProp::IsAtFront,
        };
        match self.executor.request(&request)? {
            Response::Property(Mutation::BringToFront) => Ok(true),
            _ => Err(IpcError::InvalidProp("expected is_at_front".to_owned()).into()),
        }
    }

    fn is_focused(&self) -> Result<bool, ExecutionError> {
        let request = Request::Prop {
            id: self.id,
            prop: RequestProp::IsFocused,
        };
        match self.executor.request(&request)? {
            Response::Property(Mutation::Focus) => Ok(true),
            _ => Err(IpcError::InvalidProp("expected is_focused".to_owned()).into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcError {
    InitFailure,
    Execution(String),
    InvalidProp(String),
    StreamIo(String),
    Serde(String),
}

impl Error for IpcError {}

impl Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcError::Execution(string)
            | IpcError::InvalidProp(string)
            | IpcError::StreamIo(string)
            | IpcError::Serde(string) => {
                write!(f, "{string}")
            }
            IpcError::InitFailure => {
                write!(f, "failed to initialize stream")
            }
        }
    }
}

impl From<ExecutionError> for IpcError {
    fn from(err: ExecutionError) -> Self {
        IpcError::Execution(err.to_string())
    }
}

impl From<io::Error> for ExecutionError {
    fn from(err: io::Error) -> Self {
        ExecutionError::Ipc(IpcError::StreamIo(err.to_string()))
    }
}

impl From<serde_json::Error> for ExecutionError {
    fn from(err: serde_json::Error) -> Self {
        ExecutionError::Ipc(IpcError::Serde(err.to_string()))
    }
}

impl From<IpcError> for ExecutionError {
    fn from(err: IpcError) -> Self {
        ExecutionError::Ipc(err)
    }
}
