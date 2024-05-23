use std::{
    io::{Read, Write},
    process::Child,
};

use interprocess::local_socket::{
    traits::Listener as ListenerExt, GenericNamespaced, Listener, ListenerOptions, ToNsName,
};
use serde::{Deserialize, Serialize};

use crate::state::Mutation;

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Spawn { id: u32 },
    Mutate { id: u32, mutation: Mutation },
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
// TODO: Spawn process of other binary crate dedicated to handling winit ops
// This crate will send commands over stdin, where the other crate will listen through stdin
// The other crate will report any errors that occur and a finished state
// This crate will wait for that response, if not (or a timeout hits), it will immediately error
// This struct is used for managing these foreign processes
#[derive(Debug)]
pub struct Process {
    inner: Child,
    listener: Listener,
}

impl Process {
    pub fn new() -> Result<Process, ()> {
        // TODO: generate a unique socket name that is passed to the process on construction
        // let name = "TODO".to_ns_name::<GenericNamespaced>().unwrap();
        // let listener = ListenerOptions::new().name(name).create_sync().unwrap();
        Ok(Process {
            inner: todo!(),
            listener: todo!(),
        })
    }

    pub fn execute(&mut self, command: Command) -> Result<(), ()> {
        // TODO: this unwrap should be safe?
        let mut stdin = self.inner.stdin.as_mut().unwrap();
        // TODO: use rkyv instead of serde_json
        stdin
            .write_all(&serde_json::to_string(&command).unwrap().into_bytes())
            .unwrap();

        let stdout = self.inner.stdout.as_mut().unwrap();

        Ok(())
    }

    // TODO: unwraps
    pub fn next(&self) -> Option<Command> {
        let mut stream = self.listener.accept().unwrap();

        let mut json = String::new();
        stream.read_to_string(&mut json).unwrap();

        Some(serde_json::from_str(&json).unwrap())
    }
}
