use rdev::{listen, Event, EventType, ListenError};

#[cfg(target_os = "windows")]
use crate::windows;

use crate::watcher::{WatcherEvent, WATCHER_EVENT_CHANNEL};

pub fn run() -> Result<(), ListenError> {
    let activate = |path: String| {
        let _ = WATCHER_EVENT_CHANNEL.lock().0.send(WatcherEvent {
            path,
            is_audio: false,
            active: true,
        });
    };

    listen(move |event: Event| {
        match event.event_type {
            EventType::KeyPress(_) | EventType::KeyRelease(_) => {
                #[cfg(target_os = "windows")]
                {
                    if let Some(path) = windows::get_foreground_program_path() {
                        activate(path);
                    }
                }
            }
            EventType::ButtonPress(_) | EventType::ButtonRelease(_) | EventType::Wheel { .. } => {
                #[cfg(target_os = "windows")]
                {
                    if let Some(path) = windows::get_mouse_area_program_path() {
                        activate(path);
                    }
                }
            }
            _ => {}
        };
    })
}
