use rand::{distributions::Standard, prelude::Distribution, seq::SliceRandom, Rng};
use serde::Serialize;

use crate::{
    operation::{Operation, Scope},
    state::{Mutation, State},
};

#[derive(Debug, Clone, Serialize)]
pub struct Step {
    // Local id corresponding to the window the operation should be performed on.
    pub id: u32,
    pub details: StepDetails,
}

// ExecScope is used to determine if an operation is executed as local or foreign. In
// contrast, Scope is used to determine the possible scopes of an operation.
#[derive(Debug, Clone, Serialize)]
pub enum ExecScope {
    Local,
    Foreign,
}

impl Distribution<ExecScope> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ExecScope {
        match rng.gen_range(0..=1) {
            0 => ExecScope::Local,
            _ => ExecScope::Foreign,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StepDetails {
    pub mutation: Mutation,
    pub scope: ExecScope,
    pub state: State,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub steps: Vec<Step>,
}

impl Timeline {
    // Overlaps timelines randomly, yet chronologically.
    pub fn overlap<R: Rng>(timelines: &[Timeline], rng: &mut R) -> Timeline {
        // TODO: this whole thing is super inefficient, need to rewrite optimized
        let mut timelines = timelines.to_vec();
        let mut steps = Vec::new();

        while !timelines.is_empty() {
            let i = rng.gen_range(0..timelines.len());
            let timeline = &mut timelines[i].steps;
            if timeline.is_empty() {
                timelines.remove(i);
            } else {
                steps.push(timeline.remove(timeline.len() - 1));
            }
        }

        Timeline::new(steps)
    }
}

impl Timeline {
    pub fn new(steps: Vec<Step>) -> Timeline {
        Timeline { steps }
    }

    // TODO: can return iterator and merge with the TimelineBuilder::build
    pub(crate) fn gen_details<R: Rng>(num: usize, rng: &mut R) -> Vec<StepDetails> {
        let mut timeline = Vec::with_capacity(num);

        let mut available = Vec::with_capacity(Operation::ALL.len());
        let mut state = State::new();

        for _ in 0..num {
            for operation in Operation::ALL {
                if operation.satisfied(&state) {
                    available.push(operation);
                }
            }

            // Choose a random operation from the list of available operations after constraints are satisfied.
            match available.choose(rng) {
                Some(&operation) => {
                    // Apply the operation to the state so we can handle future constraints.
                    let mutation = operation.mutation(rng);
                    state.apply(mutation.clone());
                    timeline.push(StepDetails {
                        mutation,
                        scope: match operation.scope() {
                            Scope::Local => ExecScope::Local,
                            Scope::Foreign => ExecScope::Foreign,
                            Scope::Global => rng.gen(),
                        },
                        state: state.clone(),
                    });

                    // Clear the vector so it can be reused.
                    available.clear();
                }
                // Constraints will never be strict to the point where there is no possible operation to generate.
                None => unreachable!(),
            }
        }

        timeline
    }
}


