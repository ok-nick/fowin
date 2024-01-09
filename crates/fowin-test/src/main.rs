use rand::{rngs::StdRng, Rng, SeedableRng};

// mod constraint;
mod generator;
mod operation;
mod state;

// static ID: AtomicU32 = AtomicU32::new(0);
// id: ID.fetch_add(1, Ordering::SeqCst),

fn main() {
    // TODO: temp for testing
    let mut rng = StdRng::seed_from_u64(1);

    // TODO:
    // * gen timeline for each (random) # of windows
    // * group timelines randomly
    // * overlap timelines within groups
    // * overlap group timelines
    // * execute sequentially

    let num_windows = rng.gen_range(1..10);
    let num_processes = rng.gen_range(1..num_windows);
    for _ in 0..num_windows {
        let num_steps = rng.gen_range(1..30);
        let timeline = generator::gen_timeline(num_steps, &mut rng);
        // TODO:
        // * create window with initial state (timeline[0])
        // * find diff between current state and prev state
        // * perform operation based on diff
        // * destroy window
        for window in timeline.windows(2) {
            let diff = window[0].diff(&window[1]);
            //
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
