// PLAN:
// * generate ops incrementally
//     * each op will have a constant constraint list, that says which constraints must or must not exist prior
// * iterate through each possible op and keep only those that satisfy constraints
// * generate random op based on available ops
//
// NOTE:
// * technically O(1) since there is a finite amount of operations
// * need to introduce inputs across multiple windows
// * perhaps not the most efficient solution, but is the most trivial

use crate::{operation::Operation, property::Property};

pub struct Constraint {
    pub operation: Operation,
    pub properties: &'static [Property],
}

impl Constraint {
    // TODO: make this a function of Operation
    pub const ALL: [Constraint; 2] = [
        // Constraint {
        //     operation: Operation::Create,
        //     properties: &[],
        // },
        // Constraint {
        //     operation: Operation::Destroy,
        //     properties: &[Property::Exists(true)],
        // },
        Constraint {
            operation: Operation::Resize,
            // TODO: should it be considered UB to move a fullscreened/minimized window or should we test it to do nothing?
            properties: &[Property::Fullscreened(false), Property::Hidden(false)],
        },
        Constraint {
            operation: Operation::Move,
            // TODO: same here
            properties: &[Property::Fullscreened(false), Property::Hidden(false)],
        },
        // TODO:  and more...
    ];
}
