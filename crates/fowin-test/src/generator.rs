use rand::{seq::SliceRandom, Rng};

use crate::{
    operation::{Operation, Scope},
    state::State,
};

// ExecScope is used to determine if an operation is executed as local or foreign. In
// contrast, Scope is used to determine the possible scopes of an operation.
#[derive(Debug)]
pub enum ExecScope {
    Local,
    Foreign,
}

#[derive(Debug)]
pub struct Step {
    // Local id corresponding to the window the operation should be performed on.
    pub id: u32,
    pub operation: Operation,
    // TODO: randomize scope
    pub scope: ExecScope,
}

// TODO: eventually inner vec will be an array after I fix gen_timeline
pub fn overlap_timelines<R: Rng>(timelines: &[Vec<Operation>], rng: &mut R) -> Vec<Step> {
    let global_timeline = Vec::new();

    // TODO: randomly merge timelines whilst keeping chronological order

    global_timeline
}

// TODO: maybe I should return a vec of State and the entire state is applied to the windoe each step
pub fn gen_timeline<R: Rng>(num: usize, rng: &mut R) -> Vec<Operation> {
    let mut timeline = Vec::with_capacity(num);
    timeline.push(rng.gen());

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
                operation.apply(&mut state, rng);
                timeline.push(operation);

                // Clear the vector so it can be reused.
                available.clear();
            }
            // Constraints will never be strict to the point where there is no possible operation to generate.
            None => unreachable!(),
        }
    }

    timeline
}

// TODO: impl imperative first
// pub fn gen_operations<const N: usize>() -> [Operation; N] {
//     [(); N].map(|_| {

//     })
// }
