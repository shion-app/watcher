use rdev::{listen, Event, EventType};
use tauri::{AppHandle, Runtime, Manager};

#[cfg(target_os = "windows")]
use crate::windows;

fn throttle<F>(func: F, limit: u64) -> impl FnMut(String)
where
    F: FnMut(String) + 'static,
{
    let mut last_call = std::time::Instant::now() - std::time::Duration::from_millis(limit);
    let mut last_path = String::new();
    let mut func = Box::new(func);
    move |path| {
        if last_call.elapsed().as_millis() as u64 >= limit || path != last_path {
            func(path.clone());
            last_call = std::time::Instant::now();
            last_path = path.clone();
        }
    }
}

pub fn run<R: Runtime>(app: AppHandle<R>) {
    let mut activate = throttle(move |path: String| {
        app.emit("plugin:shion-watcher://window-activate", path).unwrap();
    }, 200);

    if let Err(error) = listen(move |event: Event| {
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
    }) {
        error!("rdev error: {:?}", error)
    }
}