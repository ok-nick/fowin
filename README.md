<div align="center">

# fowin

[![crates.io](https://img.shields.io/crates/v/fowin.svg)](https://crates.io/crates/fowin)
[![docs.rs](https://docs.rs/fowin/badge.svg)](https://docs.rs/fowin)
[![check](https://github.com/ok-nick/fowin/actions/workflows/check.yml/badge.svg)](https://github.com/ok-nick/fowin/actions/workflows/check.yml)
<!-- TODO: replace check with test when test workflow is enabled -->
<!-- [![test](https://github.com/ok-nick/fowin/actions/workflows/test.yml/badge.svg)](https://github.com/ok-nick/fowin/actions/workflows/test.yml) -->

</div>

`fowin` is a cross-platform foreign window handling library for Rust.

It is for inspecting and controlling windows owned by other applications: listing existing windows,
reading their properties, moving or resizing them, changing visibility, focusing them, and receiving
window lifecycle events.

## Platform Support

| Platform | Status |
| --- | --- |
| macOS | ✓ |
| Windows | ⚠ |
| Linux | ✗ |

Windows support is experimental and under active development. It is not guaranteed to work.

> [!NOTE]
> macOS requires accessibility permissions to be granted before it can inspect or control other
windows. Call [`fowin::trusted`](https://docs.rs/fowin/latest/fowin/fn.trusted.html) to check whether
permission has been granted, and [`fowin::request_trust`](https://docs.rs/fowin/latest/fowin/fn.request_trust.html)
to prompt the user to grant it.

## Get Started

```bash
$ cargo add fowin
```

## Usage

### Listing Windows

Use [`iter_windows`](https://docs.rs/fowin/latest/fowin/fn.iter_windows.html) to inspect windows that already exist.

```rust
fn main() -> Result<(), fowin::WindowError> {
    for window in fowin::iter_windows() {
        let window = window?;
        println!("{:?}: {}", window.handle(), window.title()?);
    }

    Ok(())
}
```

See [`examples/iter_windows.rs`](examples/iter_windows.rs) for a more comprehensive example.

### Watching Windows

Use [`Watcher`](https://docs.rs/fowin/latest/fowin/struct.Watcher.html) to receive events for future window changes.

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

See [`examples/watch_windows.rs`](examples/watch_windows.rs) for a more comprehensive example.

### Manipulating Windows

Use a [`Window`](https://docs.rs/fowin/latest/fowin/struct.Window.html) handle to move, resize, focus, change visibility, etc.

```rust
fn main() -> Result<(), fowin::WindowError> {
    if let Some(window) = fowin::focused_window()? {
        window.focus()?;
        window.reposition(fowin::Position { x: 100.0, y: 100.0 })?;
        window.resize(fowin::Size {
            width: 900.0,
            height: 600.0,
        })?;
    }

    Ok(())
}
```

See the [`examples`](examples) folder for more.

## Supported Features

### Operations

[`Window`](https://docs.rs/fowin/latest/fowin/struct.Window.html) supports:

- reading the title, size, and position
- checking focus, fullscreen, minimized, and hidden state
- resizing and repositioning
- focusing, maximizing, minimizing, and unminimizing
- fullscreening and unfullscreening
- showing, hiding, and bringing windows to the front

### Events

[`Watcher`](https://docs.rs/fowin/latest/fowin/struct.Watcher.html) can report the following events:

- opened / closed
- hidden / shown
- minimized / unminimized
- focused
- moved / resized
- renamed

## FAQ

### How is this different from `winit`?

`winit` manages windows created by the local application. `fowin` manages windows created by
other applications. That requires platform APIs outside the scope of normal application windowing.

### How do I create a window?

`fowin` cannot create windows, it only inspects and controls windows owned by other applications.
To create a window, use a windowing library such as [`winit`](https://github.com/rust-windowing/winit).
