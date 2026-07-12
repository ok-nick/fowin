<div align="center">

# fowin

[![crates.io](https://img.shields.io/crates/v/fowin.svg)](https://crates.io/crates/fowin)
[![docs.rs](https://docs.rs/fowin/badge.svg)](https://docs.rs/fowin)
[![build](https://github.com/ok-nick/fowin/actions/workflows/test.yml/badge.svg)](https://github.com/ok-nick/fowin/actions/workflows/test.yml)

</div>

`fowin` is a cross-platform foreign window handling library for Rust.

It is for inspecting and controlling windows owned by other applications: listing existing windows,
reading their properties, moving or resizing them, changing visibility, focusing them, and receiving
window lifecycle events.

## Platform Support

| Platform | Status |
| --- | --- |
| macOS | ✅ |
| Windows | ⚠️ Partial |
| Linux | ❌ |

Windows support is still in development and experimental. It is not guaranteed to work.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fowin = "<version>"
```

## Usage

### Listing Windows

Use `iter_windows` to inspect windows that already exist.

```rust
fn main() -> Result<(), fowin::WindowError> {
    for window in fowin::iter_windows() {
        let window = window?;
        println!("{:?}: {}", window.handle(), window.title()?);
    }

    Ok(())
}
```

### Watching Windows

Use `Watcher` to receive events for future window changes.

```rust
fn main() -> Result<(), fowin::WindowError> {
    let mut watcher = fowin::Watcher::new()?;

    loop {
        match watcher.next_request()? {
            fowin::WindowEvent::Opened(window) => {
                println!("opened: {}", window.title()?);
            }
            fowin::WindowEvent::Closed(handle) => {
                println!("closed: {handle:?}");
            }
            event => {
                println!("{event:?}");
            }
        }
    }
}
```

### Manipulating Windows

Use a `Window` handle to move, resize, focus, change visibility, etc.

```rust
fn main() -> Result<(), fowin::WindowError> {
    if let Some(window) = fowin::focused_window()? {
        window.reposition(fowin::Position { x: 100.0, y: 100.0 })?;
        window.resize(fowin::Size {
            width: 900.0,
            height: 600.0,
        })?;
    }

    Ok(())
}
```

## Supported Window Operations

`Window` supports:

- reading the title, size, and position
- checking focus, fullscreen, minimized, and hidden state
- resizing and repositioning
- focusing, maximizing, minimizing, and unminimizing
- fullscreening and unfullscreening
- showing, hiding, and bringing windows to the front

## FAQ

### How is this different from `winit`?

`winit` manages windows created by the local application. `fowin` manages foreign windows owned by
other applications. That requires platform APIs outside the scope of normal application windowing.
