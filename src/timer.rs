use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct Timer {
    start: Arc<Mutex<Instant>>,
}

impl Timer {
    pub fn timeout<F>(duration: Duration, callback: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        let start = Arc::new(Mutex::new(Instant::now()));

        let timer_start = Arc::clone(&start);
        thread::spawn(move || loop {
            let elapsed = timer_start.lock().unwrap().elapsed();
            if elapsed >= duration {
                callback();
                break;
            }
            thread::sleep(Duration::from_millis(1000 / 60));
        });

        Timer { start }
    }

    pub fn interval<F>(duration: Duration, callback: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        let start = Arc::new(Mutex::new(Instant::now()));

        let timer_start = Arc::clone(&start);
        thread::spawn(move || loop {
            let mut start = timer_start.lock().unwrap();
            let elapsed = start.elapsed();
            if elapsed >= duration {
                callback();
                *start = Instant::now();
            }
            thread::sleep(Duration::from_millis(1000 / 60));
        });

        Timer { start }
    }

    pub fn reset(&self) {
        let mut start = self.start.lock().unwrap();
        *start = Instant::now();
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_timeout() {
        let timer = Timer::timeout(Duration::from_secs(5), || {
            println!("end: {:?}", Instant::now());
        });
        println!("start: {:?}", Instant::now());
        thread::sleep(Duration::from_secs(1));
        println!("restart {:?}", Instant::now());
        timer.reset();
        loop {}
    }

    #[test]
    fn test_interval() {
        let timer = Timer::interval(Duration::from_secs(5), || {
            println!("now: {:?}", Instant::now());
        });
        loop {}
    }
}
