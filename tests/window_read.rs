use fowin_test_lib::{
    executor::FowinExecutor, Action, Mutation, Position, Size, State, Step, Timeline,
};

mod common;

#[macro_use]
extern crate libtest_mimic_collect;

init_windowing!();

#[test]
fn test_read_title() -> Result<(), String> {
    WINIT_EXECUTOR.with_borrow_mut(|winit_executor| {
        FowinExecutor::new()
            .execute_all(
                winit_executor,
                Timeline::new(vec![
                    Step::external(1, Action::Spawn(State::initial())),
                    Step::external(1, Action::Mutate(Mutation::Title("title 1".to_owned()))),
                    Step::external(1, Action::Terminate),
                ]),
            )
            .map_err(|err| err.to_string())
    })
}

#[test]
fn test_read_size() -> Result<(), String> {
    WINIT_EXECUTOR.with_borrow_mut(|winit_executor| {
        FowinExecutor::new()
            .execute_all(
                winit_executor,
                Timeline::new(vec![
                    Step::external(1, Action::Spawn(State::initial())),
                    Step::external(
                        1,
                        Action::Mutate(Mutation::Size(Size {
                            width: 200.0,
                            height: 300.0,
                        })),
                    ),
                    Step::external(1, Action::Terminate),
                ]),
            )
            .map_err(|err| err.to_string())
    })
}

#[test]
fn test_read_position() -> Result<(), String> {
    WINIT_EXECUTOR.with_borrow_mut(|winit_executor| {
        FowinExecutor::new()
            .execute_all(
                winit_executor,
                Timeline::new(vec![
                    Step::external(1, Action::Spawn(State::initial())),
                    Step::external(
                        1,
                        Action::Mutate(Mutation::Position(Position { x: 200.0, y: 300.0 })),
                    ),
                    Step::external(1, Action::Terminate),
                ]),
            )
            .map_err(|err| err.to_string())
    })
}

// TODO: default fullscreen transition on macos takes a while, winit doesn't seem to provide a mechanism to detect when it's complete
//       see https://github.com/rust-windowing/winit/issues/2334
// #[test]
// fn test_read_fullscreen() -> Result<(), String> {
//     WINIT_EXECUTOR.with_borrow_mut(|winit_executor| {
//         FowinExecutor::new()
//             .execute_all(
//                 winit_executor,
//                 Timeline::new(vec![
//                     Step::external(1, Action::Spawn(State::initial())),
//                     Step::external(1, Action::Mutate(Mutation::Fullscreen(true))),
//                     Step::external(1, Action::Mutate(Mutation::Fullscreen(false))),
//                     Step::external(1, Action::Terminate),
//                 ]),
//             )
//             .map_err(|err| err.to_string())
//     })
// }

#[test]
fn test_read_hide() -> Result<(), String> {
    WINIT_EXECUTOR.with_borrow_mut(|winit_executor| {
        FowinExecutor::new()
            .execute_all(
                winit_executor,
                Timeline::new(vec![
                    Step::external(1, Action::Spawn(State::initial())),
                    Step::external(1, Action::Mutate(Mutation::Hide(true))),
                    Step::external(1, Action::Mutate(Mutation::Hide(false))),
                    Step::external(1, Action::Terminate),
                ]),
            )
            .map_err(|err| err.to_string())
    })
}

#[test]
fn test_read_minimize() -> Result<(), String> {
    WINIT_EXECUTOR.with_borrow_mut(|winit_executor| {
        FowinExecutor::new()
            .execute_all(
                winit_executor,
                Timeline::new(vec![
                    Step::external(1, Action::Spawn(State::initial())),
                    Step::external(1, Action::Mutate(Mutation::Minimize(true))),
                    Step::external(1, Action::Mutate(Mutation::Minimize(false))),
                    Step::external(1, Action::Terminate),
                ]),
            )
            .map_err(|err| err.to_string())
    })
}
