/**
 * https://doc.rust-lang.org/stable/std/ops/trait.Fn.html
 * ..
 */

use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use log::{info, warn};

pub struct PeriodicTimer <F> {
    name: String,
    handler: F,
    interval: u64,  // [s]
    thread: Option<thread::JoinHandle<()>>,
    do_run: Arc<AtomicBool>,
}

impl <F>PeriodicTimer <F> {
    pub fn new(name: String, handler: F, interval: u64) -> PeriodicTimer <F>
        where F: Fn() -> ()
        {
        PeriodicTimer {
            name,
            handler: F,
            interval,
            thread: None,
            do_run: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn stop(&mut self) {
        self.do_run.swap(false, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }

    pub fn start(&mut self) {
        if self.thread.is_some() {
            warn!("Refused to start thread. The thread is already running!");
            return;
        }

        // vars used by the thread internally:
        let interval = &self.interval;
        let function = &self.handler;
        let do_run = Arc::clone(&self.do_run);

        let thread = thread::Builder::new().name(self.name).spawn(
            move || {
                while do_run.load(Ordering::Relaxed) {

                    info!("Running PeriodicTimer fn..");
                    // function("ahoj!".into());

                    for i in 0..*interval {
                        thread::sleep(Duration::from_millis(1000));    
                        if !do_run.load(Ordering::Relaxed) {
                            break;
                        }
                    }
                }
        }).unwrap();

        self.thread = Some(thread);
    }
}