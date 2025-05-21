use fowin_test_lib::{
    executor::{BinaryExecutor, ExecutionError, FowinExecutor, WinitExecutor},
    Action, Mutation, State, Step, Timeline,
};

fn main() -> Result<(), ExecutionError> {
    // TODO: CLI via clap

    // TODO: NEXT NEXT HERE
    // DONE: - need to add heuristics to validation, e.g. if minimized, don't verify size, etc.
    // DONE: - change hidden to minimized?
    // wip:  - impl or cleanup at_front and focused
    //       - fixup logical/physical size distinctions
    // DONE: - fowin exectue method
    // DONE: - set window titles to GUID so when running integration tests they don't interefere
    //       - if it all works, we can start re-adding in the randomization functionality, checkout proptest
    //       - can also impl some integration tests for fowin that replicate the functionality below and call into the test lib (no randomization) for CI
    //       - before integration tests, need to implement BinaryExecutor
    //
    // AFTER_REVIEW:
    //       - get BinaryExecutor functional and working so we can really put fowin to the test with unpredictable programs
    //       - create an ImpureExecutor, one that executes random (or specified) windows currently on the system
    //       - make a basic test suite (like the current integration tests) that run on main thread

    FowinExecutor::new().execute_all(
        // &mut BinaryExecutor::new("test".to_string())?,
        &mut WinitExecutor::new(),
        Timeline::new(vec![
            Step::external(1, Action::Spawn(State::initial())),
            Step::external(1, Mutation::Minimize(true)),
            Step::external(1, Mutation::Minimize(false)),
            Step::external(1, Action::Terminate),
        ]),
    )
}
