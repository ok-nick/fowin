// TODO: use https://github.com/cross-rs/cross

use std::{thread, time::Duration};

#[test]
fn test_trust() {
    let is_trusted = fowin::trusted();
    let check_is_trusted = fowin::request_trust();

    // if it's false, we can't test selecting the prompt
    // if it's true, then requesting trust should always be true anyways
    if is_trusted {
        assert!(check_is_trusted.unwrap())
    }
}

#[test]
fn test_iter_windows() {
    // TODO: create 3 windows using winit (make sep executable?)

    for window in fowin::iter_windows() {
        let mut window = window.unwrap();
        println!(
            "
            id: {:?}
            title: {:?}
            size: {:?}
            position: {:?}
            fullscreened: {:?}
            minimized: {:?}",
            window.id(),
            window.title(),
            window.size(),
            window.position(),
            window.fullscreened(),
            window.minimized()
        );
        // loop {
        //     println!("id: {:?}\ntitle: {:?}\n", window.id(), window.title());
        //     thread::sleep(Duration::from_millis(50));
        // }
    }

    println!("\n");
}
