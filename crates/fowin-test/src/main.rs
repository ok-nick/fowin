use std::{io::Write, process::Child};

use generator::{overlap_timelines, ExecScope, Step};
use operation::{Operation, Scope};
use rand::{rngs::StdRng, Rng, SeedableRng};
use state::State;

// mod constraint;
mod generator;
mod operation;
mod state;

#[derive(Debug)]
pub struct Group {
    pub timeline: Vec<Step>,
    pub num_windows: u32,
}

fn main() {
    // TODO: temp for testing, use const_random crate
    let mut rng = StdRng::seed_from_u64(1);

    let num_groups = rng.gen_range(1..10);
    let mut groups = Vec::with_capacity(num_groups);

    for _ in 0..num_groups {
        let num_windows = rng.gen_range(1..10);
        let timelines = (0..num_windows)
            .map(|_| {
                let num_steps = rng.gen_range(1..30);
                generator::gen_timeline(num_steps, &mut rng)
            })
            .collect::<Vec<Vec<Operation>>>();

        let timeline = overlap_timelines(&timelines, &mut rng);
        groups.push(Group {
            timeline,
            num_windows,
        });
    }

    // TODO: spawn processes, cache them, create windows w/ state, then execute (overlapped group) timeline,
    // to preserve window local ids, ensure steps are labeled with their group ids
    for group in groups {
        let mut process = Process::new().unwrap(); // TODO: handle unwrap

        let mut states = Vec::with_capacity(group.num_windows as usize);
        for step in group.timeline {
            match states.get_mut(step.id as usize) {
                Some(state) => {
                    step.operation.apply(state, &mut rng);
                    match step.scope {
                        // Use fowin... kinda contradictory huh?
                        ExecScope::Local => {
                            todo!()
                        }
                        ExecScope::Foreign => {
                            process.execute(state).unwrap();
                        }
                    }

                    // TODO: verify props using fowin here
                }
                None => {
                    // TODO: ensure index exists
                    states[step.id as usize] = State::new();
                    // TODO: apply state to window
                }
            }
        }

        // TODO: destroy window
    }
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

    // TODO:
    // * pass the entire state?
    // * pass the modified property?
    pub fn execute(&mut self, state: &State) -> Result<(), ()> {
        // TODO: this unwrap should be safe?
        let mut stdin = self.inner.stdin.as_mut().unwrap();
        // TODO: use rkyv instead of serde_json
        stdin
            .write_all(&serde_json::to_string(state).unwrap().into_bytes())
            .unwrap();

        // TODO: wait for error/ok response from process stdout
        let stdout = self.inner.stdout.as_mut().unwrap();

        Ok(())
    }
}
