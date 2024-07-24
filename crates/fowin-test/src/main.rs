use fowin_test_lib::{Action, FowinExecutor, Mutation, State, Step, Timeline, WinitExecutor};

fn main() {
    // TODO: CLI via clap

    // TODO: NEXT NEXT HERE
    // DONE: - need to add heuristics to validation, e.g. if minimized, don't verify size, etc.
    //       - change hidden to minimized?
    //       - impl or cleanup at_front and focused
    //       - fixup logical/physical size distinctions
    //       - set window titles to GUID so when running integrationt tests they don't interefere
    //       - if it all works, we can start re-adding in the randomization functionality, checkout proptest
    //       - can also impl some integration tests for fowin that replicate the functionality below and call into the test lib (no randomization) for CI

    FowinExecutor::new()
        .execute_all(
            &mut WinitExecutor::new(),
            Timeline::new(vec![
                Step::external(1, Action::Spawn(State::initial())),
                Step::external(1, Action::Mutate(Mutation::Hidden(true))),
                Step::external(1, Action::Mutate(Mutation::Hidden(false))),
                Step::external(1, Action::Terminate),
            ]),
        )
        .unwrap();
}
