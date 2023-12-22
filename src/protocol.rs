use std::time::Instant;

use crate::sys::Window;

pub type WindowId = u32;

#[derive(Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

pub trait WindowManagerBackend {
    // generic interface over backend, provides functions like:
    // window resized, window moved, etc. is it possible to be generic over this for all platforms?

    // Request accessibility (take inspiration):
    // https://github.com/next-slide-please/macos-accessibility-client

    // Keybinds:
    // https://github.com/Narsil/rdev
    // https://github.com/obv-mikhail/inputbot
    // https://github.com/tauri-apps/global-hotkey

    // Windowing/keybinds:
    // https://github.com/RustAudio/baseview/tree/master

    // Windowing:
    // https://github.com/LGUG2Z/komorebi/blob/master/komorebi/src/window.rs
    //    https://github.com/LGUG2Z/komorebi/blob/master/komorebi/src/windows_api.rs#L361

    // Events:
    // https://developer.apple.com/documentation/applicationservices/axnotificationconstants_h/miscellaneous_defines?changes=l_1_7_5&language=objc
    // Aero:
    // kAXFocusedWindowChangedNotification
    // kAXMovedNotification
    // kAXResizedNotification
    // kAXUIElementDestroyedNotification
    // kAXWindowCreatedNotification
    // kAXWindowDeminiaturizedNotification
    // kAXWindowMiniaturizedNotification

    // yabai:
    // kAXFocusedWindowChangedNotification
    // *kAXMenuClosedNotification
    // *kAXMenuOpenedNotification
    // *kAXTitleChangedNotification
    // kAXUIElementDestroyedNotification
    // kAXWindowDeminiaturizedNotification
    // kAXWindowMiniaturizedNotification
    // *=kAXWindowMovedNotification (worse than kAXMovedNotification)
    // *=kAXWindowResizedNotification (worse than kAXResizedNotification)

    // komorebi:
    // EVENT_OBJECT_DESTROY
    // EVENT_OBJECT_HIDE
    // EVENT_OBJECT_CLOAKED
    // EVENT_SYSTEM_MINIMIZESTART
    // EVENT_OBJECT_SHOW | EVENT_SYSTEM_MINIMIZEEND
    // EVENT_OBJECT_UNCLOAKED
    // EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND
    // EVENT_SYSTEM_MOVESIZESTART
    // EVENT_SYSTEM_MOVESIZEEND
    // EVENT_SYSTEM_CAPTURESTART | EVENT_SYSTEM_CAPTUREEND
    // EVENT_OBJECT_NAMECHANGE

    // same:
    // kAXWindowCreatedNotification | EVENT_OBJECT_CREATE
    // kAXUIElementDestroyedNotification | EVENT_OBJECT_DESTROY
    // kAXWindowMiniaturizedNotification | EVENT_OBJECT_HIDE | EVENT_OBJECT_CLOAKED | EVENT_SYSTEM_MINIMIZESTART
    // kAXWindowDeminiaturizedNotification | EVENT_OBJECT_SHOW | EVENT_SYSTEM_MINIMIZEEND | EVENT_OBJECT_UNCLOAKED
    // kAXFocusedWindowChangedNotification | EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND
    // kAXMovedNotification | EVENT_SYSTEM_MOVESIZESTART | EVENT_SYSTEM_MOVESIZEEND
    // (can be handled with diff macos api) | EVENT_SYSTEM_CAPTURESTART | EVENT_SYSTEM_CAPTUREEND
    // kAXTitleChangedNotification | EVENT_OBJECT_NAMECHANGE

    // NOTE: komorebi doesn't listen to EVENT_OBJECT_CREATE because "some apps like firefox" don't send them
    // https://github.com/LGUG2Z/komorebi/blob/42ac13e0bd24c2775874cac891826024054e4e3c/komorebi/src/window_manager_event.rs#L127

    // NOTE: on windows the events are not processed in the correct order, This should be handled on the windows backend, and if needbe, the windows backend should buffer events every 0.x seconds so every event is processed in order (read komorebi server for more)
    // fn event_receiver(&self) -> Receiver<WindowEvent>;

    fn show_window(&self, id: WindowId);

    fn hide_window(&self, id: WindowId);

    fn focus_window(&self, id: WindowId);

    fn move_window(&self, id: WindowId, position: Position);

    fn resize_window(&self, id: WindowId, size: Size);

    // NOTE: returns the displays screen resolution so that the backend can calculate window positions relative to it
    // fn resolution(&self, id: DisplayId) -> Size;

    // NOTE: this returns the position of the display relative to other displays, used when finding the focus cross-display
    // fn position(&self, id: DisplayId) -> Position;
}

// event can either happen on event or after event (assume after event)
#[derive(Debug, PartialEq, Eq)]
pub enum WindowEvent {
    Opened,
    Closed,
    Hidden,
    Shown,
    Focused,
    Moved,
    Resized,
    Renamed,
}

// TODO: unpub
#[derive(Debug)]
pub struct WindowEventInfo {
    pub event: WindowEvent,
    pub timestamp: Instant,
    pub window: Window,
}
