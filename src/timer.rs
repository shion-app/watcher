use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct Timer {
    timeout: Duration,
    callback: Box<dyn Fn() + Send + Sync + 'static>,
    start_time: Mutex<Instant>,
}

impl Timer {
    pub fn new<F>(timeout: Duration, callback: F) -> Arc<Self>
    where
        F: Fn() + Send + Sync + 'static,
    {
        Arc::new(Self {
            timeout,
            callback: Box::new(callback),
            start_time: Mutex::new(Instant::now()),
        })
    }

    pub fn start(self: &Arc<Self>) {
        let timer = Arc::clone(self);
        thread::spawn(move || loop {
            let elapsed_time = Instant::now() - *timer.start_time.lock().unwrap();
            if elapsed_time >= timer.timeout {
                (timer.callback)();
                break;
            }
            thread::sleep(Duration::from_millis(100));
        });
    }

    pub fn reset(self: &Arc<Self>) {
        let timer = Arc::clone(self);
        *timer.start_time.lock().unwrap() = Instant::now();
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
        timer.start();
        println!("start");
        thread::sleep(Duration::from_secs(1));
        timer.reset();
        loop {
            
        }
    }
}
