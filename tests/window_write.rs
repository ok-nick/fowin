use fowin_test_lib::{
    executor::FowinExecutor, Action, Mutation, Position, Size, State, Step, Timeline,
};

mod common;

#[macro_use]
extern crate libtest_mimic_collect;

init_windowing!();

#[test]
fn write_size() -> Result<(), String> {
    WINIT_EXECUTOR.with_borrow_mut(|winit_executor| {
        FowinExecutor::new()
            .execute_all(
                winit_executor,
                Timeline::new(vec![
                    Step::external(1, Action::Spawn(State::initial())),
                    Step::fowin(
                        1,
                        Mutation::Size(Size {
                            width: 200.0,
                            height: 300.0,
                        }),
                    ),
                    Step::external(1, Action::Terminate),
                ]),
            )
            .map_err(|err| err.to_string())
    })
}

// #[test]
// fn write_fullscreen() -> Result<(), String> {
//     WINIT_EXECUTOR.with_borrow_mut(|winit_executor| {
//         FowinExecutor::new()
//             .execute_all(
//                 winit_executor,
//                 Timeline::new(vec![
//                     Step::external(1, Action::Spawn(State::initial())),
//                     Step::fowin(1, Mutation::Fullscreen(true)),
//                     Step::fowin(1, Mutation::Fullscreen(false)),
//                     Step::external(1, Action::Terminate),
//                 ]),
//             )
//             .map_err(|err| err.to_string())
//     })
// }

#[test]
fn write_position() -> Result<(), String> {
    WINIT_EXECUTOR.with_borrow_mut(|winit_executor| {
        FowinExecutor::new()
            .execute_all(
                winit_executor,
                Timeline::new(vec![
                    Step::external(1, Action::Spawn(State::initial())),
                    Step::fowin(1, Mutation::Position(Position { x: 200.0, y: 300.0 })),
                    Step::external(1, Action::Terminate),
                ]),
            )
            .map_err(|err| err.to_string())
    })
}

#[test]
fn write_hide() -> Result<(), String> {
    WINIT_EXECUTOR.with_borrow_mut(|winit_executor| {
        FowinExecutor::new()
            .execute_all(
                winit_executor,
                Timeline::new(vec![
                    Step::external(1, Action::Spawn(State::initial())),
                    Step::fowin(1, Mutation::Hide(true)),
                    Step::fowin(1, Mutation::Hide(false)),
                    Step::external(1, Action::Terminate),
                ]),
            )
            .map_err(|err| err.to_string())
    })
}

#[test]
fn write_minimize() -> Result<(), String> {
    WINIT_EXECUTOR.with_borrow_mut(|winit_executor| {
        FowinExecutor::new()
            .execute_all(
                winit_executor,
                Timeline::new(vec![
                    Step::external(1, Action::Spawn(State::initial())),
                    Step::fowin(1, Mutation::Minimize(true)),
                    Step::fowin(1, Mutation::Minimize(false)),
                    Step::external(1, Action::Terminate),
                ]),
            )
            .map_err(|err| err.to_string())
    })
}
