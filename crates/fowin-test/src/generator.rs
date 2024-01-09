use rand::Rng;

use crate::{operation::Operation, state::State};

// static ID: AtomicU32 = AtomicU32::new(0);
// id: ID.fetch_add(1, Ordering::SeqCst),

#[derive(Debug)]
pub struct Step {
    pub operation: Operation,
    pub expected_state: State,
}

pub fn gen_operations<R: Rng>(num: usize, rng: &mut R) -> Vec<Step> {
    //     // First things first, we need to create a window to get the ball rolling.
    //     let mut operations = vec![Step {
    //         operation: Operation::Create,
    //         expected_state: State::random_but_exists(rng),
    //     }];

    //     let mut available = Vec::new();
    //     for i in 1..num {
    //         // Previous element will always exist since we start at index 1.
    //         let state = &operations.get(i - 1).unwrap().expected_state;

    //         for constraint in Constraint::ALL {
    //             let mut satisfied = true;
    //             for property in constraint.properties {
    //                 if property == &state.get(property.key()) {
    //                     // If the constraint isn't satisfied, skip this operation.
    //                     satisfied = false;
    //                     break;
    //                 }
    //             }

    //             if satisfied {
    //                 available.push(constraint.operation);
    //             }
    //         }

    //         match available.choose(rng) {
    //             Some(&operation) => {
    //                 let mut state = state.clone();

    //                 state.set(operation.gen_random_property(rng));

    //                 operations.push(Step {
    //                     operation,
    //                     expected_state: state,
    //                 });

    //                 // Clear the vector so it can be reused.
    //                 available.clear();
    //             }
    //             // Constraints will never be strict to the point where there is no possible operation to generate.
    //             None => unreachable!(),
    //         }
    //     }

    //     operations
    todo!()
}

// TODO: impl imperative first
// pub fn gen_operations<const N: usize>() -> [Operation; N] {
//     [(); N].map(|_| {

//     })
// }
