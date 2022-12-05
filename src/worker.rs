
use std::time::Duration;
use std::sync::Arc;
use std::sync::Mutex;
use queues::*;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};

use ogn_client::data_structures::{AircraftBeacon, AddressType};

use crate::configuration::GEOTIFF_FILEPATH;

mod geo_file;
use geo_file::GeoFile;


pub struct Worker {
    thread: Option<thread::JoinHandle<()>>,
    do_run: Arc<AtomicBool>,
    worker_type: AddressType,
    queue: Arc<Mutex<Queue<AircraftBeacon>>>,
}

impl Worker {
    pub fn new(worker_type: AddressType,  queue: Arc<Mutex<Queue<AircraftBeacon>>>) -> Worker {
        // let thread = thread::spawn(move || loop {
        //     loop {
        //         let size = queue.lock().unwrap().size();
        //         // println!("LOOP1");
        //         println!("LOOP1 {} {}", worker_type.to_string(), size);
        //         thread::sleep(Duration::from_secs(2));    
        //     }
        // });

        Self {
            thread: None, //Some(thread),
            do_run: Arc::new(AtomicBool::new(true)),
            worker_type: worker_type,
            queue: queue,
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
            println!("[WARN] Refused to start thread. The thread is already running!");
            return;
        }

        // vars used by the thread internally:
        let q = Arc::clone(&self.queue);
        let do_run = Arc::clone(&self.do_run);

        let thread = thread::spawn(
            move || {
                let mut geo_file = GeoFile::new(GEOTIFF_FILEPATH);

                while do_run.load(Ordering::Relaxed) {
                    let num_queued = q.lock().unwrap().size();
                    if num_queued == 0 {
                        thread::sleep(Duration::from_millis(100));    
                        continue;
                    }
                    
                    while q.lock().unwrap().size() > 0 {

                        let beacon = q.lock().unwrap().remove().unwrap();
                        // self.process_beacon(&beacon);

                        let terrain_elevation = geo_file.get_value(beacon.lat, beacon.lon);
                        let agl = match terrain_elevation {
                            Some(e) => beacon.altitude - e as i32,
                            None => 0,
                        };
                        println!("beacon: {beacon} | agl: {agl}m");

                    }

                    //TODO the meat!
                }
        });

        self.thread = Some(thread);
        println!("[INFO] THREAD {} started.", self.worker_type);
    }

    fn process_beacon(&self, beacon: &AircraftBeacon) {
        println!("beacon: {} {} lat: {:.5} lon: {:.5} alt: {:.1}", beacon.prefix, beacon.addr, beacon.lat, beacon.lon, beacon.altitude);
        // let agl = geo_file.get_value(beacon.lat, beacon.lon);

    }
}
