use fowin::{WindowError, WindowEventInfo};

fn main() -> Result<(), WindowError> {
    if !fowin::request_trust()? {
        return Err(WindowError::NotTrusted);
    }

    println!("Searching for windows...");
    let watcher = fowin::watch()?; // TODO: takes too long, need to filter processes
    println!("Windows found, now watching.");

    loop {
        match watcher.next_request() {
            Ok(event) => {
                let (name, kind) = match event.info() {
                    WindowEventInfo::Opened(window) => (window.title(), "opened"),
                    // TODO: cache title for window
                    WindowEventInfo::Closed(id) => (Ok(id.to_string()), "closed"),
                    WindowEventInfo::Hidden(window) => (window.title(), "hidden"),
                    WindowEventInfo::Shown(window) => (window.title(), "shown"),
                    WindowEventInfo::Focused(window) => (window.title(), "focused"),
                    WindowEventInfo::Moved(window) => (window.title(), "moved"),
                    WindowEventInfo::Resized(window) => (window.title(), "resized"),
                    WindowEventInfo::Renamed(window) => (window.title(), "renamed"),
                };
                let name = name.as_deref().unwrap_or("UNKNOWN");

                println!("Window `{name}` has been {kind}!");
            }
            Err(err) => {
                // TODO: this case often occurs when a process is unsubscriptable,
                println!("TODO `{err}`");
            }
        }
    }
}
