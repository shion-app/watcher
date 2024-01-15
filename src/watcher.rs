use std::sync::Arc;
use std::thread;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::{AppHandle, Runtime};

use crate::event;
#[cfg(target_os = "windows")]
use crate::windows;

lazy_static! {
    pub static ref WATCHER_EVENT_CHANNEL: Arc<Mutex<(Sender<WatcherEvent>, Receiver<WatcherEvent>)>> =
        Arc::new(Mutex::new(crossbeam_channel::unbounded()));
}

pub struct Watcher {
    pool: Vec<Program>,
}

struct Program {
    path: String,
    is_audio: bool,
}

#[derive(Debug)]
pub struct WatcherEvent {
    pub path: String,
    pub is_audio: bool,
    pub active: bool,
}

impl Watcher {
    fn init() {}

    // pub fn run<R: Runtime>(app: AppHandle<R>) {
    pub fn run() {
        thread::spawn(|| {
            #[cfg(target_os = "windows")]
            if let Err(err) = windows::App::start() {
                error!("windows watcher error: {}", err);
            }
        });
        thread::spawn(|| {
            if let Err(err) = event::run() {
                error!("rdev error: {:?}", err);
            }
        });

        loop {
            if let Ok(event) = WATCHER_EVENT_CHANNEL.lock().1.try_recv() {
                println!("{:?}", event);
            }
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_run() {
        Watcher::run();
    }
}
