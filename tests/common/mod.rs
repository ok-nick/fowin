#[macro_export]
macro_rules! init_windowing {
    () => {
        // `winit` event loops cannot be created more than once, thus we cache it here.
        ::std::thread_local! {
            pub static WINIT_EXECUTOR: ::std::cell::RefCell<::fowin_test_lib::executor::WinitExecutor> =
                ::std::cell::RefCell::new(::fowin_test_lib::executor::WinitExecutor::new());
        }

        // NOTE: on macOS, these tests MUST run on the main (UI) thread. Unfortunately, it's no longer possible to do
        //       with cargo test, so we use libtest_mimic and libtest_mimic_collect for the macros. Note that
        //       --test-threads=1 must be passed to run on the main thread.
        //
        //       relevant issue: https://github.com/rust-lang/rust/issues/104053
        fn main() {
            if !::fowin::request_trust().unwrap() {
                panic!("{}", ::fowin::WindowError::NotTrusted);
            }

            ::libtest_mimic_collect::TestCollection::run();
        }
    };
}
