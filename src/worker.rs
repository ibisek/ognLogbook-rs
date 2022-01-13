
use std::time::Duration;
use std::sync::Arc;
use std::sync::Mutex;
use queues::*;
use std::thread;

use ogn_client::data_structures::{AircraftBeacon, AddressType};

pub struct Worker {
    thread: Option<thread::JoinHandle<()>>,
    do_run: bool,
    // worker_type: AddressType,
    // queue: Arc<Mutex<Queue<AircraftBeacon>>>,
}

impl Worker {
    pub fn new(worker_type: AddressType,  queue: Arc<Mutex<Queue<AircraftBeacon>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            loop {
                let size = queue.lock().unwrap().size();
                // println!("LOOP1");
                println!("LOOP1 {} {}", worker_type.to_string(), size);
                thread::sleep(Duration::from_secs(2));    
            }
        });

        Self {
            thread: Some(thread),
            do_run: true,
            // worker_type: worker_type,
            // queue: queue,
        }
    }

    pub fn stop(&mut self) {
        self.do_run = false;
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }

    // pub fn start(&mut self) {
    //     if self.thread.is_some() {
    //         println!("[WARN] Refused to start thread. The thread is already running!");
    //         return;
    //     }

    //     let thread = thread::spawn(move || loop {
    //         let qSize = self.queue.lock().unwrap().size();
    //         // println!("LOOP2");
    //         println!("LOOP2 {}", qSize);
    //         thread::sleep(Duration::from_secs(2));    
    //     });

    //     self.thread = Some(thread);
    //     println!("THREAD {} started.", self.worker_type);
    // }
}
