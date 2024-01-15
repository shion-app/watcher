use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::{AppHandle, Runtime};

use crate::event;
use crate::timer::Timer;
#[cfg(target_os = "windows")]
use crate::windows;

lazy_static! {
    pub static ref WATCHER_EVENT_CHANNEL: Arc<Mutex<(Sender<WatcherEvent>, Receiver<WatcherEvent>)>> =
        Arc::new(Mutex::new(crossbeam_channel::unbounded()));
}

pub struct Watcher<R: Runtime> {
    app: AppHandle<R>,
    pool: Mutex<Vec<Program>>,
}

struct Program {
    path: String,
    is_audio: bool,
    timer: Timer,
}

#[derive(Debug)]
pub struct WatcherEvent {
    pub path: String,
    pub is_audio: bool,
    pub active: bool,
}

impl<R: Runtime> Watcher<R> {
    pub fn new(app: AppHandle<R>) -> Arc<Self> {
        Arc::new(Self {
            app,
            pool: Mutex::new(vec![]),
        })
    }

    pub fn run(self: &Arc<Self>) {
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
                self.handle(event);
            }
        }
    }

    fn handle(self: &Arc<Self>, event: WatcherEvent) {
        let mut pool = self.pool.lock();
        let index = pool.iter().position(|p| p.path == event.path);
        if !event.active {
            if let Some(index) = index {
                if event.is_audio || !pool[index].is_audio {
                    drop(pool);
                    self.remove(index);
                }
            }
            return;
        }
        if let Some(index) = index {
            let program = &mut pool[index];
            program.is_audio = event.is_audio;
            drop(pool);
            self.reset_timer(index);
        } else {
            let mut list = vec![];
            for (i, _) in pool.iter().enumerate() {
                if !pool[i].is_audio {
                    list.push(i);
                }
            }
            drop(pool);
            for i in list {
                self.remove(i)
            }
            let timer = Timer::new(Duration::from_secs(60), {
                let watcher = Arc::clone(&self);
                let path = event.path.clone();
                move || {
                    let pool = watcher.pool.lock();
                    let index = pool.iter().position(|p| p.path == path);
                    if let Some(index) = index {
                        drop(pool);
                        watcher.remove(index);
                    }
                }
            });
            self.add(Program {
                path: event.path,
                is_audio: event.is_audio,
                timer,
            })
        }
    }

    fn remove(&self, index: usize) {
        let mut pool = self.pool.lock();
        let path = pool[index].path.clone();
        pool.remove(index);
        debug!("remove program: {}", path);
    }

    fn add(&self, program: Program) {
        let mut pool = self.pool.lock();
        let path = program.path.clone();
        pool.push(program);
        debug!("add program: {}", path);
    }

    fn reset_timer(&self, index: usize) {
        let  pool = self.pool.lock();
        let program = &pool[index];
        program.timer.reset();
    }
}

mod tests {
    use super::*;

    // #[test]
    // fn test_watcher_run() {
    //     let watcher = Watcher::new();
    //     watcher.run()
    // }
}
