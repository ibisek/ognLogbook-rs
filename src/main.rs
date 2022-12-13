#[warn(non_snake_case)]

use queues::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

use log::{info};
use simplelog::{ConfigBuilder, LevelFilter, SimpleLogger};
use time::macros::format_description;

mod configuration;
use configuration::{LOG_LEVEL, OGN_USERNAME, OGN_APRS_FILTER_LAT, OGN_APRS_FILTER_LON, OGN_APRS_FILTER_RANGE};

use ogn_client::data_structures::{AircraftBeacon, Observer, AddressType};
use ogn_client::OgnClient;

mod worker;
use worker::Worker;

struct AircraftBeaconListener {
    beacon_counter: u32,
    ogn_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    icao_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    flarm_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    safesky_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    time: SystemTime,
}

impl AircraftBeaconListener {
    fn new(ogn_q: Arc<Mutex<Queue<AircraftBeacon>>>, 
        icao_q: Arc<Mutex<Queue<AircraftBeacon>>>, 
        flarm_q: Arc<Mutex<Queue<AircraftBeacon>>>,
        safesky_q: Arc<Mutex<Queue<AircraftBeacon>>>) -> AircraftBeaconListener {
        Self {
            beacon_counter:0,
            ogn_q,
            icao_q,
            flarm_q,
            safesky_q,
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
        } else
        if beacon.addr_type == AddressType::SafeSky {
            self.safesky_q.lock().unwrap().add(beacon).unwrap();
        } 

        if self.time.elapsed().unwrap().as_secs() >= 60 {
            let num_ogn = self.ogn_q.lock().unwrap().size();
            let num_icao = self.icao_q.lock().unwrap().size();
            let num_flarm = self.flarm_q.lock().unwrap().size();
            let num_sky = self.safesky_q.lock().unwrap().size();
            info!("Beacon rate: {}/min, {} queued (O {} / I {} / F {} / S {})", 
                self.beacon_counter, 
                num_ogn + num_icao + num_flarm + num_sky,
                num_ogn, num_icao, num_flarm, num_sky
            );
            
            self.beacon_counter = 0;
            self.time = SystemTime::now();
        }
    }
}

fn main() -> std::io::Result<()> {
    let config = ConfigBuilder::new()
        .set_target_level(LOG_LEVEL)
        .set_time_format_custom(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:2]"))
        .build();
    let _ = SimpleLogger::init(LOG_LEVEL, config);
    
    info!("\n\n## OGN LOGBOOK ##\n");

    let mut client: OgnClient = OgnClient::new(OGN_USERNAME)?;
    client.set_aprs_filter(OGN_APRS_FILTER_LAT, OGN_APRS_FILTER_LON, OGN_APRS_FILTER_RANGE);
    client.connect();

    let queue_ogn: Arc<Mutex<Queue<AircraftBeacon>>> = Arc::new(Mutex::new(Queue::new()));
    let queue_icao: Arc<Mutex<Queue<AircraftBeacon>>> = Arc::new(Mutex::new(Queue::new()));
    let queue_flarm: Arc<Mutex<Queue<AircraftBeacon>>> = Arc::new(Mutex::new(Queue::new()));
    let queue_safesky: Arc<Mutex<Queue<AircraftBeacon>>> = Arc::new(Mutex::new(Queue::new()));
    
    let abl = AircraftBeaconListener::new(
        Arc::clone(&queue_ogn), 
        Arc::clone(&queue_icao), 
        Arc::clone(&queue_flarm), 
        Arc::clone(&queue_safesky));
    client.set_beacon_listener(abl);

    // create and run workers:
    let mut ogn_worker = Worker::new(AddressType::Ogn, queue_ogn);
    ogn_worker.start();
    let mut icao_worker = Worker::new(AddressType::Icao, queue_icao);
    icao_worker.start();
    let mut flarm_worker = Worker::new(AddressType::Flarm, queue_flarm);
    flarm_worker.start();
    let mut safesky_worker = Worker::new(AddressType::SafeSky, queue_safesky);
    safesky_worker.start();

    info!("Entering the loop..");
    client.do_loop();

    info!("KOHEU.");
    Ok(())
}
