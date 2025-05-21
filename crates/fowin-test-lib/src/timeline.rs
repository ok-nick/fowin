use serde::{Deserialize, Serialize};

use crate::state::{Mutation, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    // Local id corresponding to the window the operation should be performed on.
    pub id: u32,
    pub action: Action,
    pub scope: ExecScope,
}

impl Step {
    pub fn fowin(id: u32, mutation: Mutation) -> Self {
        Self {
            id,
            action: Action::Mutate(mutation),
            scope: ExecScope::Fowin,
        }
    }

    pub fn external<T: Into<Action>>(id: u32, action: T) -> Self {
        Self {
            id,
            action: action.into(),
            scope: ExecScope::External,
        }
    }
}

// ExecScope is used to determine if an operation is executed as local or foreign. In
// contrast, Scope is used to determine the possible scopes of an operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExecScope {
    Fowin,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Mutate(Mutation),
    Spawn(State),
    Terminate,
}

impl From<Mutation> for Action {
    fn from(mutation: Mutation) -> Self {
        Action::Mutate(mutation)
    }
}

#[derive(Debug, Clone)]
pub struct Timeline {
    steps: Vec<Step>,
}

impl Timeline {
    pub fn new(steps: Vec<Step>) -> Self {
        Self { steps }
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    pub fn into_steps(self) -> Vec<Step> {
        self.steps
    }
}

// impl Timeline {
//     // Overlaps timelines arbitrarily, yet chronologically.
//     pub fn overlap<R: Rng>(orig_timelines: &[Timeline], rng: &mut R) -> Timeline {
//         // TODO: this whole thing is super inefficient, need to rewrite optimized
//         let mut timelines = orig_timelines.to_vec();
//         let mut steps = Vec::new();

//         while !timelines.is_empty() {
//             let i = rng.gen_range(0..timelines.len());
//             let timeline = &mut timelines[i].steps;
//             if timeline.is_empty() {
//                 timelines.remove(i);
//             } else {
//                 // Naive method of inserting `Spawn` action before the first mutation.
//                 if timeline.len() == orig_timelines[i].steps.len() {
//                     steps.push(Step {
//                         id: timeline[0].id,
//                         action: Action::Spawn,
//                         scope: timeline[0].scope,
//                     })
//                 }

//                 // steps.push(timeline.remove(timeline.len() - 1));
//                 steps.push(timeline.remove(0));

//                 // Naive method of inserting `Terminate` action after the last mutation.
//                 if timeline.is_empty() {
//                     let last_step = steps.last().unwrap();
//                     steps.push(Step {
//                         id: last_step.id,
//                         action: Action::Terminate,
//                         scope: last_step.scope,
//                     })
//                 }
//             }
//         }

//         Timeline {
//             steps,
//             // TODO
//             scopes: HashMap::new(),
//         }
//     }
// }

// impl Timeline {
// // TODO: can return iterator and merge with the TimelineBuilder::build
//     pub(crate) fn gen_actions<R: Rng>(num: usize, rng: &mut R) -> Vec<Action> {
//         let mut timeline = Vec::with_capacity(num);

//         let mut available = Vec::with_capacity(Operation::ALL.len());
//         let mut state = State::new();

//         for _ in 0..num {
//             for operation in Operation::ALL {
//                 if operation.satisfied(&state) {
//                     available.push(operation);
//                 }
//             }

//             // Choose a random operation from the list of available operations after constraints are satisfied.
//             match available.choose(rng) {
//                 Some(&operation) => {
//                     // Apply the operation to the state so we can handle future constraints.
//                     let mutation = operation.mutation(rng);
//                     state.apply(mutation.clone());
//                     timeline.push(ActionDetails {
//                         action: Action::Mutate(mutation),
//                         scope: match operation.scope() {
//                             Scope::Local => ExecScope::Local,
//                             Scope::Foreign => ExecScope::Foreign,
//                             Scope::Global => rng.gen(),
//                         },
//                     });

//                     // Clear the vector so it can be reused.
//                     available.clear();
//                 }
//                 // Constraints will never be strict to the point where there is no possible operation to generate.
//                 None => unreachable!(),
//             }
//         }

//         timeline
//     }
// }

// #[derive(Debug)]
// pub struct TimelineBuilder {
//     max_processes: u32,
//     max_windows: u32,
//     max_steps: u32,
// }

// impl TimelineBuilder {
//     pub fn new() -> TimelineBuilder {
//         TimelineBuilder {
//             max_processes: 1,
//             max_windows: 1,
//             max_steps: 1,
//         }
//     }

//     pub fn max_windows(mut self, max: u32) -> Self {
//         self.max_windows = max;
//         self
//     }

//     pub fn max_steps(mut self, max: u32) -> Self {
//         self.max_steps = max;
//         self
//     }

//     pub fn build<R: Rng>(self, rng: &mut R) -> Timeline {
//         let num_processes = rng.gen_range(1..=self.max_processes);
//         let mut timelines = Vec::with_capacity(num_processes as usize);

//         let mut id = 0;
//         for i in 0..num_processes {
//             let num_windows = rng.gen_range(1..=self.max_windows);
//             let local_timelines = (0..num_windows)
//                 .map(|_| {
//                     let num_steps = rng.gen_range(1..=self.max_steps);
//                     let steps = Timeline::gen_actions(num_steps as usize, rng)
//                         .into_iter()
//                         .map(|step| Step { id, details: step })
//                         .collect();

//                     id += 1;

//                     Timeline::new(steps)
//                 })
//                 // TODO: assign windows within each timeline to a process id based on their index in the final vec
//                 .collect::<Vec<Timeline>>();

//             let timeline = Timeline::overlap(&local_timelines, rng);
//             timelines.push(timeline);
//         }

//         Timeline::overlap(&timelines, rng)
//     }
// }

// #[derive(Debug, Default)]
// pub struct TimelineBuilder {
//     steps: Vec<Step>,
// }

// impl TimelineBuilder {
//     pub fn new() -> TimelineBuilder {
//         Self::default()
//     }

//     pub fn spawn_local(mut self, id: u32, scope: ExecScope) -> Self {
//         self.steps.push(Step {
//             id,
//             action: Action::Spawn,
//             scope,
//         });
//         self
//     }

//     pub fn spawn_foreign(mut self, id: u32) -> Self {
//         self.scopes.insert(id, ExecScope::Foreign);
//         self.steps.push(Step {
//             id,
//             action: Action::Spawn,
//         });
//         self
//     }

//     pub fn title(mut self, id: u32, title: String) -> Self {
//         self.steps.push(Step {
//             id,
//             action: Action::Mutate(Mutation::Title(title)),
//         });
//         self
//     }

//     pub fn minimize(mut self, id: u32) -> Self {
//         self.steps.push(Step {
//             id,
//             action: Action::Mutate(Mutation::Hidden(true)),
//         });
//         self
//     }

//     pub fn unminimize(mut self, id: u32) -> Self {
//         self.steps.push(Step {
//             id,
//             action: Action::Mutate(Mutation::Hidden(false)),
//         });
//         self
//     }

//     // TODO

//     pub fn terminate(mut self, id: u32) -> Self {
//         self.steps.push(Step {
//             id,
//             action: Action::Terminate,
//         });
//         self
//     }

//     pub fn build(self) -> Timeline {
//         Timeline {
//             steps: self.steps,
//             scopes: self.scopes,
//         }
//     }
// }
