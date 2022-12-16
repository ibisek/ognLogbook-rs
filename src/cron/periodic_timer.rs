/**
 * https://doc.rust-lang.org/stable/std/ops/trait.Fn.html
 * ..
 */

 use std::time::Duration;
 use std::sync::atomic::{AtomicBool, Ordering};
 use std::sync::Arc;
 use std::thread;
 
 use log::{info, warn};
 
 // type PeriodicTimerHandler = fn() -> ();
 
 pub struct PeriodicTimer {
     name: String,
     handler: Arc<fn()>,
     interval: u64,  // [s]
     thread: Option<thread::JoinHandle<()>>,
     do_run: Arc<AtomicBool>,
 }
 
 impl PeriodicTimer {
     pub fn new(name: String, interval: u64, handler: fn()) -> PeriodicTimer {
         PeriodicTimer {
             name,
             handler: Arc::new(handler),
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
         let interval = self.interval.clone();
         let handler = Arc::clone(&self.handler);
         let do_run = Arc::clone(&self.do_run);
 
         let thread = thread::Builder::new().name(self.name.clone()).spawn(
             move || {
                 while do_run.load(Ordering::Relaxed) {
                     handler();
 
                     for _ in 0..interval {
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