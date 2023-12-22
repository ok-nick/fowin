// #[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod platform;

pub use platform::*;
