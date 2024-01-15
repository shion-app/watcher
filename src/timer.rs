use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

pub struct Timer {
    start_time: Arc<Mutex<Instant>>,
    done: Arc<AtomicBool>,
}

impl Timer {
    pub fn new<F>(timeout: Duration, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        let start_time = Arc::new(Mutex::new(Instant::now()));
        let done = Arc::new(AtomicBool::new(false));

        thread::spawn({
            let start_time = Arc::clone(&start_time);
            let done = done.clone();
            move || {
                while !done.load(Ordering::Relaxed) {
                    let elapsed_time = Instant::now() - *start_time.lock();
                    if elapsed_time >= timeout {
                        callback();
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });

        Self { start_time, done }
    }

    pub fn reset(&self) {
        *self.start_time.lock() = Instant::now();
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.done.store(true, Ordering::Relaxed);
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_run() {
        let timer = Timer::new(Duration::from_secs(5), || {
            println!("end");
            println!("{:?}", Instant::now());
        });
        println!("{:?}", Instant::now());
        println!("start");
        thread::sleep(Duration::from_secs(1));
        println!("restart {:?}", Instant::now());
        timer.reset();
        loop {}
    }
}
