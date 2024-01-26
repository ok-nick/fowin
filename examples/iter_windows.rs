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
focused: {:?}
",
            window.handle(),
            window.title(),
            window.size(),
            window.position(),
            window.fullscreened(),
            window.minimized(),
            window.focused()
        );
    }

    Ok(())
}
