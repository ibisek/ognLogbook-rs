#[warn(non_snake_case)]

use queues::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

use ogn_client::data_structures::{AircraftBeacon, Observer, AddressType};
use ogn_client::OgnClient;

mod worker;
use worker::Worker;

struct AircraftBeaconListener {
    beacon_counter: u32,
    ogn_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    icao_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    flarm_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    time: SystemTime,
}

impl AircraftBeaconListener {
    fn new(ogn_q: Arc<Mutex<Queue<AircraftBeacon>>>, 
        icao_q: Arc<Mutex<Queue<AircraftBeacon>>>, 
        flarm_q: Arc<Mutex<Queue<AircraftBeacon>>>) -> AircraftBeaconListener {
        Self {
            beacon_counter:0,
            ogn_q,
            icao_q,
            flarm_q,
            time: SystemTime::now(),
        }
    }
}

impl Observer<AircraftBeacon> for AircraftBeaconListener {
    fn notify(&mut self, beacon: AircraftBeacon) {
        self.beacon_counter += 1;

        if beacon.addr_type == AddressType::Ogn {
            self.ogn_q.lock().unwrap().add(beacon).unwrap();
        } else 
        if beacon.addr_type == AddressType::Icao {
            self.icao_q.lock().unwrap().add(beacon).unwrap();
        } else 
        if beacon.addr_type == AddressType::Flarm {
            self.flarm_q.lock().unwrap().add(beacon).unwrap();
        } 

        if self.time.elapsed().unwrap().as_secs() >= 60 {
            println!("[INFO] Beacon rate: {}/min, {} queued.", 
                self.beacon_counter, 
                self.ogn_q.lock().unwrap().size() + self.icao_q.lock().unwrap().size() +self.flarm_q.lock().unwrap().size(),
            );
            
            self.beacon_counter = 0;
            self.time = SystemTime::now();
        }
    }
}

fn main() -> std::io::Result<()> {
    let username = "blume2";
    let lat = 49.1234;
    let lon = 16.4567;
    let range = 999999;
    
    let mut client: OgnClient = OgnClient::new(username)?;
    client.set_aprs_filter(lat, lon, range);
    client.connect();

    let queue_ogn: Arc<Mutex<Queue<AircraftBeacon>>> = Arc::new(Mutex::new(Queue::new()));
    let queue_icao: Arc<Mutex<Queue<AircraftBeacon>>> = Arc::new(Mutex::new(Queue::new()));
    let queue_flarm: Arc<Mutex<Queue<AircraftBeacon>>> = Arc::new(Mutex::new(Queue::new()));
    
    let abl = AircraftBeaconListener::new(Arc::clone(&queue_ogn), Arc::clone(&queue_icao), Arc::clone(&queue_flarm));
    client.set_beacon_listener(abl);

    // create and run workers:
    let mut ogn_worker = Worker::new(AddressType::Ogn, queue_ogn);
    // let mut icao_worker = Worker::new(AddressType::Icao, queue_icao);
    // let mut flarm_worker = Worker::new(AddressType::Flarm, queue_flarm);
    
    println!("Entering the loop..");
    client.do_loop();

    println!("KOHEU.");
    Ok(())
}
