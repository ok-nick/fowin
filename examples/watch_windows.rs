use fowin::{Watcher, WindowError, WindowEvent};

fn main() -> Result<(), WindowError> {
    if !fowin::request_trust()? {
        return Err(WindowError::NotTrusted);
    }

    println!("Searching for windows...");
    let mut watcher = Watcher::new()?;
    println!("Windows found, now watching.");

    loop {
        match watcher.next_request() {
            Ok(event) => {
                let (name, kind) = match event {
                    WindowEvent::Opened(window) => (window.title(), "opened"),
                    WindowEvent::Closed(id) => (Ok(id.to_string()), "closed"),
                    WindowEvent::Hidden(window) => (window.title(), "hidden"),
                    WindowEvent::Shown(window) => (window.title(), "shown"),
                    WindowEvent::Focused(window) => (window.title(), "focused"),
                    WindowEvent::Moved(window) => (window.title(), "moved"),
                    WindowEvent::Resized(window) => (window.title(), "resized"),
                    WindowEvent::Renamed(window) => (window.title(), "renamed"),
                };
                let name = name.as_deref().unwrap_or("UNKNOWN");

                println!("Window `{name}` has been {kind}!");
            }
            Err(err) => {
                println!("TODO `{err}`");
            }
        }
    }
}
