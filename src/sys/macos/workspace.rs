use std::{
    ffi::c_void,
    ptr,
    sync::mpsc::{self, Receiver, Sender},
};

use libc::pid_t;
use objc2::{define_class, msg_send, rc::Retained, runtime::AnyObject, AnyThread};
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_core_foundation::{
    kCFRunLoopDefaultMode, CFRunLoop, CFRunLoopSource, CFRunLoopSourceContext,
};
use objc2_foundation::{
    ns_string, NSArray, NSDictionary, NSKeyValueChangeKey, NSKeyValueChangeNewKey,
    NSKeyValueChangeOldKey, NSKeyValueObservingOptions, NSObject, NSString,
};

use crate::sys::platform::ffi::CFRetainedSafe;

use super::{application, filter_apps, iter_apps};

#[derive(Debug)]
pub enum AppEventKind {
    Existing,
    Launched,
    Terminated,
    Registered(application::AppWatcher),
}

#[derive(Debug)]
pub struct AppEvent {
    pub kind: AppEventKind,
    pub pid: pid_t,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[derive(Debug)]
    struct WorkspaceWatcherInner;

    impl WorkspaceWatcherInner {
        #[allow(non_snake_case)]
        #[unsafe(method(observeValueForKeyPath:ofObject:change:context:))]
        unsafe fn observeValueForKeyPath_ofObject_change_context(
            &self,
            _key_path: Option<&NSString>,
            _object: Option<&AnyObject>,
            change: Option<&NSDictionary<NSKeyValueChangeKey, AnyObject>>,
            context: *mut c_void,
        ) {
            if let Some(change) = change {
                let context = context as *mut Context;
                let mut sent = false;

                if let Some(new_apps) = change.objectForKey(NSKeyValueChangeNewKey) {
                    let new_apps = new_apps
                        .downcast::<NSArray>()
                        .unwrap()
                        .into_iter()
                        .map(|app| app.downcast::<NSRunningApplication>().unwrap());
                    for app in filter_apps(new_apps) {
                        let _ = (*context).sender.send(AppEvent {
                            kind: AppEventKind::Launched,
                            pid: app.processIdentifier(),
                        });

                        sent = true;
                    }
                }

                if let Some(old_apps) = change.objectForKey(NSKeyValueChangeOldKey) {
                    let old_apps = old_apps
                        .downcast::<NSArray>()
                        .unwrap()
                        .into_iter()
                        .map(|app| app.downcast::<NSRunningApplication>().unwrap());
                    for app in filter_apps(old_apps) {
                        let _ = (*context).sender.send(AppEvent {
                            kind: AppEventKind::Terminated,
                            pid: app.processIdentifier(),
                        });

                        sent = true;
                    }
                }

                if sent {
                    (*context).source.signal();
                    CFRunLoop::main().unwrap().wake_up();
                }
            }
        }
    }
);

#[derive(Debug)]
pub struct Context {
    pub sender: Sender<AppEvent>,
    // The reason we create a "dummy" source is because registering a KVO (AKA WorkspaceWatcherInner) does not trigger
    // a source as being "processed" thus not prompting CFRunLoopInMode to return.
    pub source: CFRetainedSafe<CFRunLoopSource>,
}

/// Whether to watch all existing apps or to skip them and only watch new ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExistingAppsBehavior {
    TriggerExisting,
    Skip,
}

/// Uses key-value observing (KVO) on `NSWorkspace::running_applications` to detect and report
/// app launches/terminations.
///
/// This requires the main thread's `CFRunLoop` to be executed, which is being done for callers
/// in [`Watcher::next_request`].
///
/// Numerous options were considered, see [#10] for more details.
///
/// [`Watcher::next_request`]: super::Watcher::next_request
/// [#10]: https://github.com/ok-nick/fowin/issues/10
#[derive(Debug)]
pub struct WorkspaceWatcher {
    inner: Retained<WorkspaceWatcherInner>,
    context: Box<Context>,
    receiver: Receiver<AppEvent>,
}

impl WorkspaceWatcher {
    pub fn new(existing_apps_behavior: ExistingAppsBehavior) -> WorkspaceWatcher {
        let source = unsafe {
            CFRunLoopSource::new(
                None,
                -1,
                &mut CFRunLoopSourceContext {
                    version: 0,
                    info: ptr::null_mut(),
                    retain: None,
                    release: None,
                    copyDescription: None,
                    equal: None,
                    hash: None,
                    schedule: None,
                    cancel: None,
                    perform: None,
                },
            )
        };

        unsafe {
            CFRunLoop::main()
                .unwrap()
                .add_source(source.as_deref(), kCFRunLoopDefaultMode);
        }

        let (sender, receiver) = mpsc::channel();
        let context = Box::into_raw(Box::new(Context {
            sender: sender.clone(),
            source: CFRetainedSafe(source.unwrap()),
        }));

        let inner: Retained<WorkspaceWatcherInner> =
            unsafe { msg_send![WorkspaceWatcherInner::alloc(), init] };
        unsafe {
            let _: () = msg_send![
                &NSWorkspace::sharedWorkspace(),
                addObserver: &*inner,
                forKeyPath: ns_string!("runningApplications"),
                options: NSKeyValueObservingOptions::New | NSKeyValueObservingOptions::Old,
                context: context as *const c_void
            ];
        }

        if existing_apps_behavior == ExistingAppsBehavior::TriggerExisting {
            for app in iter_apps() {
                sender
                    .send(AppEvent {
                        kind: AppEventKind::Existing,
                        pid: app.pid(),
                    })
                    // Sender + Receiver always exist.
                    .unwrap();
            }
        }

        WorkspaceWatcher {
            inner,
            context: unsafe { Box::from_raw(context) },
            receiver,
        }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    // Assumes the run loop is being ran.
    pub fn next_request(&self) -> Option<AppEvent> {
        // Impossible for disconnected error, only possible for empty error, in which case it should be an option.
        self.receiver.try_recv().ok()
    }
}

impl Drop for WorkspaceWatcher {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![
                &NSWorkspace::sharedWorkspace(),
                removeObserver: &*self.inner,
                forKeyPath: ns_string!("runningApplications"),
                context: &*self.context as *const _ as *const c_void
            ];
        }
    }
}
