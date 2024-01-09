use rand::{seq::SliceRandom, Rng};

use crate::state::{PropertyKind, State};

pub fn overlap_timelines<R: Rng>(rng: &mut R) {
    // TODO: takes two timelines and overlaps them
}

pub fn gen_timeline<R: Rng>(num: usize, rng: &mut R) -> Vec<State> {
    let mut timeline = Vec::with_capacity(num);

    let mut available = Vec::with_capacity(PropertyKind::ALL.len());
    let mut state = State::random(rng);
    for _ in 0..num {
        for kind in PropertyKind::ALL {
            let not_satisfied = kind
                .constraints()
                .iter()
                .any(|constraint| state.get(constraint.kind()) != constraint);
            if !not_satisfied {
                available.push(kind);
            }

            // Choose a random operation from the list of available operations after constraints are satisfied.
            // One potential issue is generating a false value for a property that is already false. It may seem
            // redundant, but it is beneficial to test for cases like this, as the expected outcome should be the same.
            match available.choose(rng) {
                Some(&kind) => {
                    state.set(kind.random(rng));
                    timeline.push(state.clone());

                    // Clear the vector so it can be reused.
                    available.clear();
                }
                // Constraints will never be strict to the point where there is no possible operation to generate.
                None => unreachable!(),
            }
        }
    }

    timeline
}

// TODO: impl imperative first
// pub fn gen_operations<const N: usize>() -> [Operation; N] {
//     [(); N].map(|_| {

//     })
// }
