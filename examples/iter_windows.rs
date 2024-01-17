// TODO: use https://github.com/cross-rs/cross

use fowin::WindowError;

fn main() -> Result<(), WindowError> {
    if !fowin::request_trust()? {
        return Err(WindowError::NotTrusted);
    }

    for window in fowin::iter_windows() {
        let window = window?;
        println!(
            "id: {:?}
title: {:?}
size: {:?}
position: {:?}
fullscreened: {:?}
minimized: {:?}
focused: {:?}
",
            window.id(),
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
