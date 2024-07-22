// TODO: use https://github.com/cross-rs/cross

use fowin::WindowError;

fn main() -> Result<(), WindowError> {
    if !fowin::request_trust()? {
        return Err(WindowError::NotTrusted);
    }

    // Skip windows that aren't valid.
    for window in fowin::iter_windows().flatten() {
        println!(
            "handle: {:?}
title: {:?}
size: {:?}
position: {:?}
fullscreened: {:?}
minimized: {:?}
hidden: {:?}
focused: {:?}
",
            window.handle(),
            window.title(),
            window.size(),
            window.position(),
            window.is_fullscreen(),
            window.is_minimized(),
            window.is_hidden(),
            window.is_focused()
        );
    }

    Ok(())
}
