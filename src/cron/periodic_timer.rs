/**
 * A tasker to execute callbacks in periodic intervals.
 */

use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use log::{info, warn};

pub trait PeriodicTimerTask {
     fn tick(&self);
 }
 
 pub struct PeriodicTimer {
     name: String,
     interval: u64,  // [s]
     thread: Option<thread::JoinHandle<()>>,
     do_run: Arc<AtomicBool>,
     handler: Arc<fn()>,
    //  handler_argument: Arc<Option<_>>,
    //  task: Arc<dyn PeriodicTimerTask + Sync + Send + 'static>,
 }
 
 impl PeriodicTimer {
     pub fn new(name: String, interval: u64, handler: fn() /*, handler_argument: Option<&T>*/) -> PeriodicTimer {
         PeriodicTimer {
             name,
             interval,
             thread: None,
             do_run: Arc::new(AtomicBool::new(true)),
             handler: Arc::new(handler),
            //  handler_argument: Arc::new(handler_argument),
         }
     }

    //  pub fn new(name: String, interval: u64, task: impl PeriodicTimerTask + Sync + Send + 'static) -> PeriodicTimer //  + DerefMut
    //     // where T: PeriodicTimerTask + 'static,
    //     {
    //     PeriodicTimer {
    //         name,
    //         interval,
    //         thread: None,
    //         do_run: Arc::new(AtomicBool::new(true)),
    //         handler: None,
    //         // task: Box::new(task), 
    //         task: Arc::new(task) as Arc<dyn PeriodicTimerTask + Sync + Send>,
    //     }
    // }
 
     pub fn stop(&mut self) {
         self.do_run.swap(false, Ordering::Relaxed);
         if let Some(thread) = self.thread.take() {
             thread.join().unwrap();
         }
         
         info!("Stopped timer for '{}'", self.name);
     }
 
     pub fn start(&mut self) {
         if self.thread.is_some() {
             warn!("Refused to start thread. The thread is already running!");
             return;
         }

         info!("Starting a timer for '{}' with interval of {}s", self.name, self.interval);
 
         // vars used by the thread internally:
         let interval = self.interval.clone();
         let do_run = Arc::clone(&self.do_run);
         let handler = Arc::clone(&self.handler);
        //  let task = Arc::clone(&self.task);
        
         let thread = thread::Builder::new().name(self.name.clone()).spawn(
             move || {
                 while do_run.load(Ordering::Relaxed) {
                    handler();
                    // task.tick();
 
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