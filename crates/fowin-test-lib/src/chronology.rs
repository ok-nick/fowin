use rand::Rng;

use crate::{
    process::{Command, Process},
    timeline::{ExecScope, Step, Timeline},
};

#[derive(Debug)]
pub struct Chronology {
    // Since window ids are guaranteed to be sequential, we can store them in a vector.
    window_to_process: Vec<usize>,
    timeline: Timeline,
}

impl Chronology {
    pub fn execute(&self) {
        // TODO: inefficient, for now, I can definitely precompute this somewhere else
        let num_processes = *self.window_to_process.iter().max().unwrap();
        let mut processes = (0..num_processes)
            .map(|_| Process::new())
            .collect::<Result<Vec<Process>, ()>>()
            .unwrap();
        for (id, process_index) in self.window_to_process.iter().enumerate() {
            processes[*process_index]
                .execute(Command::Spawn { id: id as u32 })
                .unwrap();
        }

        for step in &self.timeline.steps {
            // TODO: verify props for fowin here, before we begin as well

            match step.details.scope {
                // Use fowin locally... kinda contradictory huh?
                ExecScope::Local => {
                    todo!()
                }
                ExecScope::Foreign => {
                    // TODO: unwraps
                    let process = processes
                        .get_mut(*self.window_to_process.get(step.id as usize).unwrap())
                        .unwrap();
                    process
                        .execute(Command::Mutate {
                            id: step.id,
                            mutation: step.details.mutation.clone(),
                        })
                        .unwrap();
                }
            }

            // TODO: verify props using fowin here
        }
    }
}

#[derive(Debug)]
pub struct ChronologyBuilder {
    max_processes: u32,
    max_windows: u32,
    max_steps: u32,
}

impl ChronologyBuilder {
    pub fn new() -> ChronologyBuilder {
        ChronologyBuilder {
            max_processes: 1,
            max_windows: 1,
            max_steps: 1,
        }
    }

    pub fn max_processes(mut self, max: u32) -> Self {
        self.max_processes = max;
        self
    }

    pub fn max_windows(mut self, max: u32) -> Self {
        self.max_windows = max;
        self
    }

    pub fn max_steps(mut self, max: u32) -> Self {
        self.max_steps = max;
        self
    }

    pub fn build<R: Rng>(self, rng: &mut R) -> Chronology {
        let num_processes = rng.gen_range(1..self.max_processes);
        let mut timelines = Vec::with_capacity(num_processes as usize);
        let mut window_to_process = Vec::new();

        let mut id = 0;
        for i in 0..num_processes {
            let num_windows = rng.gen_range(1..self.max_windows);
            let local_timelines = (0..num_windows)
                .map(|_| {
                    let num_steps = rng.gen_range(1..self.max_steps);
                    let steps = Timeline::gen_details(num_steps as usize, rng)
                        .into_iter()
                        .map(|step| Step { id, details: step })
                        .collect();

                    window_to_process.push(i as usize);
                    id += 1;

                    Timeline::new(steps)
                })
                .collect::<Vec<Timeline>>();

            // TODO: tack on a create/destroy to the start/end of each timeline
            let timeline = Timeline::overlap(&local_timelines, rng);
            timelines.push(timeline);
        }

        Chronology {
            window_to_process,
            timeline: Timeline::overlap(&timelines, rng),
        }
    }
}

impl Default for ChronologyBuilder {
    fn default() -> Self {
        Self::new()
    }
}
