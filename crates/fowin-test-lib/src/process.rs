use std::{io::Write, process::Child};

use serde::{Deserialize, Serialize};

use crate::state::Mutation;

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Spawn { id: u32 },
    Mutate { id: u32, mutation: Mutation },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Err(),
}

// TODO: Spawn process of other binary crate dedicated to handling winit ops
// This crate will send commands over stdin, where the other crate will listen through stdin
// The other crate will report any errors that occur and a finished state
// This crate will wait for that response, if not (or a timeout hits), it will immediately error
// This struct is used for managing these foreign processes
#[derive(Debug)]
pub struct Process {
    inner: Child,
}

impl Process {
    pub fn new() -> Result<Process, ()> {
        Ok(Process { inner: todo!() })
    }

    // TODO: include window id in command
    pub fn execute(&mut self, command: Command) -> Result<(), ()> {
        // TODO: this unwrap should be safe?
        let mut stdin = self.inner.stdin.as_mut().unwrap();
        // TODO: use rkyv instead of serde_json
        stdin
            .write_all(&serde_json::to_string(&command).unwrap().into_bytes())
            .unwrap();

        // TODO: wait for error/ok response from process stdout, I don't think there is a reliable way to do this, may have to do IPC
        let stdout = self.inner.stdout.as_mut().unwrap();

        Ok(())
    }
}
