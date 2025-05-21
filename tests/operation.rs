use fowin_test_lib::{
    executor::{ExecutionError, FowinExecutor, WinitExecutor},
    Action, Mutation, Position, Size, State, Step, Timeline,
};

#[macro_use]
extern crate libtest_mimic_collect;

// TODO: in order to run multiple test cases, the winitexecutor needs to be shared (As
//       event loops cannot be created more than once). Might have to revert back to
//       libtest_mimic to share the executor.

// NOTE: on macOS, these tests MUST run on the main (UI) thread. Unfortuantely, it's no longer possible to do
//       with cargo test, so we use libtest_mimic and libtest_mimic_collect for the macros. Note that
//       --test-threads=1 must be passed to run on the main thread.
//
//       relevent issue: https://github.com/koekeishiya/yabai/issues/2190
fn main() {
    libtest_mimic_collect::TestCollection::run();
}

#[test]
fn test_title() -> Result<(), String> {
    FowinExecutor::new()
        .execute_all(
            &mut WinitExecutor::new(),
            Timeline::new(vec![
                Step::external(1, Action::Spawn(State::initial())),
                Step::external(1, Action::Mutate(Mutation::Title("title 1".to_owned()))),
                Step::external(1, Action::Mutate(Mutation::Title("title 2".to_owned()))),
                Step::external(1, Action::Terminate),
            ]),
        )
        .map_err(|err| err.to_string())
}

// TODO: unfortunately macos returns the window content size + the tilte bar size
//       so validation fails, need to handle this inconsistency in fowin
// #[test]
// fn test_size() -> Result<(), String> {
//     FowinExecutor::new()
//         .execute_all(
//             &mut WinitExecutor::new(),
//             Timeline::new(vec![
//                 Step::external(1, Action::Spawn(State::initial())),
//                 Step::external(
//                     1,
//                     Action::Mutate(Mutation::Size(Size {
//                         width: 200.0,
//                         height: 300.0,
//                     })),
//                 ),
//                 // Step::external(
//                 //     1,
//                 //     Action::Mutate(Mutation::Size(Size {
//                 //         width: 50.0,
//                 //         height: 50.0,
//                 //     })),
//                 // ),
//                 Step::external(1, Action::Terminate),
//             ]),
//         )
//         .map_err(|err| err.to_string())
// }

// #[test]
// fn test_position() -> Result<(), String> {
//     FowinExecutor::new()
//         .execute_all(
//             &mut WinitExecutor::new(),
//             Timeline::new(vec![
//                 Step::external(1, Action::Spawn(State::initial())),
//                 Step::external(
//                     1,
//                     Action::Mutate(Mutation::Position(Position { x: 200.0, y: 300.0 })),
//                 ),
//                 Step::external(1, Action::Terminate),
//             ]),
//         )
//         .map_err(|err| err.to_string())
// }

// TODO: broken, when unfullscreening fowin reports it a s fullscreened
// #[test]
// fn test_fullscreen() -> Result<(), String> {
//     FowinExecutor::new()
//         .execute_all(
//             &mut WinitExecutor::new(),
//             Timeline::new(vec![
//                 Step::external(1, Action::Spawn(State::initial())),
//                 Step::external(1, Action::Mutate(Mutation::Fullscreen(true))),
//                 Step::external(1, Action::Mutate(Mutation::Fullscreen(false))),
//                 Step::external(1, Action::Terminate),
//             ]),
//         )
//         .map_err(|err| err.to_string())
// }

// #[test]
// fn test_hide() -> Result<(), String> {
//     FowinExecutor::new()
//         .execute_all(
//             &mut WinitExecutor::new(),
//             Timeline::new(vec![
//                 Step::external(1, Action::Spawn(State::initial())),
//                 Step::external(1, Action::Mutate(Mutation::Hide(true))),
//                 Step::external(1, Action::Mutate(Mutation::Hide(false))),
//                 Step::external(1, Action::Terminate),
//             ]),
//         )
//         .map_err(|err| err.to_string())
// }

// #[test]
// fn test_minimize() -> Result<(), String> {
//     FowinExecutor::new()
//         .execute_all(
//             &mut WinitExecutor::new(),
//             Timeline::new(vec![
//                 Step::external(1, Action::Spawn(State::initial())),
//                 Step::external(1, Action::Mutate(Mutation::Minimize(true))),
//                 Step::external(1, Action::Mutate(Mutation::Minimize(false))),
//                 Step::external(1, Action::Terminate),
//             ]),
//         )
//         .map_err(|err| err.to_string())
// }
