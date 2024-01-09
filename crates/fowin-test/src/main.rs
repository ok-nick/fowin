use rand::{rngs::StdRng, SeedableRng};

// mod constraint;
mod generator;
mod operation;
mod state;

fn main() {
    // TODO: temp for testing
    let mut rng = StdRng::seed_from_u64(1);

    let steps = generator::gen_operations(30, &mut rng);
    for step in steps {
        match step.operation.scope() {
            operation::Scope::Local => {
                // use winit
            }
            operation::Scope::Foreign => {
                // use fowin
            }
            operation::Scope::Global => {
                // use corresponding window to determine scope
            }
        }
    }
}

// TODO: manages detached winit processes
#[derive(Debug)]
pub struct ProcessManager {}

// NOTE: Here's how it will work.
// * A randomized amount of windows will be created.
// * Windows will be grouped randomly into a random amount of groups (denoting a new process).
// * For each window, the generator will generate a random amount of operations.
// * The final step takes each array of operations and randomly overlaps them to simulate a live system.
//
// That's the data. Now all we have to do is execute it in order.
//
// Generating Operations:
// * It is possible to completely elimintate the idea of operations and instead rely on randomizing random properties. However,
// that would disallow our constraint model.
// * Constraints should be merged with operations, it makes sense.
// *
