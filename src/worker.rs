
use std::time::Duration;
use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time};

use std::sync::atomic::{AtomicBool, Ordering};

use log::{info, warn};
use queues::*;

use ogn_client::data_structures::{AircraftBeacon, AddressType};

mod beacon_processor;
use beacon_processor::BeaconProcessor;
mod data_structures;
mod db_thread;
mod expiring_dict;
mod geo_file;
mod influx_worker;
mod permanent_storage;
mod utils;
mod python_influx_bridge;

pub struct Worker {
    thread: Option<thread::JoinHandle<()>>,
    do_run: Arc<AtomicBool>,
    worker_type: AddressType,
    queue: Arc<Mutex<Queue<AircraftBeacon>>>,
}

impl Worker {
    pub fn new(worker_type: AddressType,  queue: Arc<Mutex<Queue<AircraftBeacon>>>) -> Worker {
        Self {
            thread: None, //Some(thread),
            do_run: Arc::new(AtomicBool::new(true)),
            worker_type,
            queue: queue,
        }
    }

    pub fn stop(&mut self) {
        info!("Stopping worker for {}", self.worker_type.as_long_str());
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
        let q = Arc::clone(&self.queue);
        let do_run = Arc::clone(&self.do_run);
        let worker_name = self.worker_type.as_long_str();
        let worker_type = self.worker_type.clone();

        let thread = thread::Builder::new().name(self.worker_type.as_long_str()).spawn(
            move || {
                // let mut geo_file = GeoFile::new(GEOTIFF_FILEPATH);
                let mut bp = BeaconProcessor::new(&worker_type);

                while do_run.load(Ordering::Relaxed) {
                    let num_queued = q.lock().unwrap().size();
                    if num_queued == 0 {
                        thread::sleep(Duration::from_millis(100));    
                        continue;
                    }
                    
                    // while q.lock().unwrap().size() > 0 {
                    //     let mut beacon = q.lock().unwrap().remove().unwrap();
                    //     bp.process(&mut beacon);
                    // }

                    match q.try_lock() {
                        Ok(mut q_mutex_guard) => {
                            match q_mutex_guard.remove() {
                                Ok(mut beacon) => bp.process(&mut beacon),
                                _ => (),
                            };
                        },
                        Err(_) => {
                            thread::yield_now();
                        },
                    }
                }

                info!("{} worker thread terminated.", worker_name);
        }).unwrap();

        self.thread = Some(thread);
        info!("Thread {} started.", self.worker_type);
    }
    
}
