use fowin_test_lib::ChronologyBuilder;
use rand::{rngs::StdRng, SeedableRng};

fn main() {
    // TODO: CLI via clap

    // TODO: temp for testing, use const_random crate
    let mut rng = StdRng::seed_from_u64(1);
    ChronologyBuilder::new()
        .max_processes(5)
        .max_windows(10)
        .max_steps(20)
        .build(&mut rng)
        .execute();
}
