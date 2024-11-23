use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct Timer {
    start: Arc<Mutex<Instant>>,
    callback: Arc<Mutex<Box<dyn Fn() + Send + 'static>>>,
    done: Arc<Mutex<bool>>,
    duration: Arc<Mutex<Duration>>,
}

impl Timer {
    pub fn new<F>(duration: Duration, callback: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        Self {
            start: Arc::new(Mutex::new(Instant::now())),
            callback: Arc::new(Mutex::new(Box::new(callback))),
            done: Arc::new(Mutex::new(false)),
            duration: Arc::new(Mutex::new(duration)),
        }
    }

    pub fn timeout(&self) {
        let start = Arc::clone(&self.start);
        let callback = Arc::clone(&self.callback);
        let duration = Arc::clone(&self.duration);
        let done = Arc::clone(&self.done);
        thread::spawn(move || loop {
            let elapsed = start.lock().unwrap().elapsed();
            let duration = duration.lock().unwrap();
            let mut done = done.lock().unwrap();
            if elapsed >= *duration {
                (callback.lock().unwrap())();
                *done = true;
                break;
            }
            thread::sleep(Duration::from_millis(1000 / 60));
        });
    }

    pub fn interval(&self) {
        let start = Arc::clone(&self.start);
        let callback = Arc::clone(&self.callback);
        let duration = Arc::clone(&self.duration);
        thread::spawn(move || loop {
            let mut start = start.lock().unwrap();
            let duration = duration.lock().unwrap();
            let elapsed = start.elapsed();
            if elapsed >= *duration {
                (callback.lock().unwrap())();
                *start = Instant::now();
            }
            thread::sleep(Duration::from_millis(1000 / 60));
        });
    }

    pub fn reset(&self) {
        let mut start = self.start.lock().unwrap();
        let mut done = self.done.lock().unwrap();
        *start = Instant::now();
        if *done {
            *done = false;
            self.timeout();
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_timeout() {
        let timer = Timer::new(Duration::from_secs(2), || {
            println!("end: {:?}", Instant::now());
        });
        timer.timeout();
        println!("start: {:?}", Instant::now());
        thread::sleep(Duration::from_secs(1));
        println!("restart {:?}", Instant::now());
        timer.reset();
        thread::sleep(Duration::from_secs(3));
        println!("after timer done, reset  {:?}", Instant::now());
        timer.reset();
        loop {}
    }

    #[test]
    fn test_interval() {
        let timer = Timer::new(Duration::from_secs(5), || {
            println!("now: {:?}", Instant::now());
        });
        timer.interval();
        loop {}
    }
}
