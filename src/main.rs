#[warn(non_snake_case)]

use queues::*;
use std::sync::Arc;
use std::sync::Mutex;

use log::info;
use simplelog::{ConfigBuilder, SimpleLogger};
use time::macros::format_description;

mod airfield_manager;

mod aircraft_beacon_listener;
use aircraft_beacon_listener::AircraftBeaconListener;

mod configuration;
use configuration::{LOG_LEVEL, get_ogn_username, OGN_APRS_FILTER_LAT, OGN_APRS_FILTER_LON, OGN_APRS_FILTER_RANGE};

mod mqtt;

use ogn_client::data_structures::{AircraftBeacon, AddressType};
use ogn_client::OgnClient;

mod worker;
use worker::Worker;

mod cron;
use cron::CronJobs;

mod db;

fn main() -> std::io::Result<()> {
    let config = ConfigBuilder::new()
        .set_target_level(LOG_LEVEL)
        .set_time_format_custom(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:2]"))
        .build();
    let _ = SimpleLogger::init(LOG_LEVEL, config);
    
    info!("\n\n## OGN LOGBOOK ##\n");

    let client = Arc::new(Mutex::new(OgnClient::new(&get_ogn_username())?));
    client.lock().unwrap().set_aprs_filter(OGN_APRS_FILTER_LAT, OGN_APRS_FILTER_LON, OGN_APRS_FILTER_RANGE);
    client.lock().unwrap().connect();

    let queue_ogn: Arc<Mutex<Queue<AircraftBeacon>>> = Arc::new(Mutex::new(Queue::new()));
    let queue_icao: Arc<Mutex<Queue<AircraftBeacon>>> = Arc::new(Mutex::new(Queue::new()));
    let queue_flarm: Arc<Mutex<Queue<AircraftBeacon>>> = Arc::new(Mutex::new(Queue::new()));
    let queue_safesky: Arc<Mutex<Queue<AircraftBeacon>>> = Arc::new(Mutex::new(Queue::new()));
    
    let abl = AircraftBeaconListener::new(
        Arc::clone(&queue_ogn), 
        Arc::clone(&queue_icao), 
        Arc::clone(&queue_flarm), 
        Arc::clone(&queue_safesky));
    client.lock().unwrap().set_beacon_listener(abl);

    let mut workers = vec![];
    // create and run workers:
    let mut ogn_worker = Worker::new(AddressType::Ogn, queue_ogn);
    ogn_worker.start();
    workers.push(ogn_worker);
    let mut icao_worker = Worker::new(AddressType::Icao, queue_icao);
    icao_worker.start();
    workers.push(icao_worker);
    let mut flarm_worker = Worker::new(AddressType::Flarm, queue_flarm);
    flarm_worker.start();
    workers.push(flarm_worker);
    let mut safesky_worker = Worker::new(AddressType::SafeSky, queue_safesky);
    safesky_worker.start();
    workers.push(safesky_worker);

    // create and run cron jobs:
    let mut cron = CronJobs::new();
    cron.start();

    // configure the ctrl+c hook:
    // let cl = Arc::clone(&client);
    // ctrlc::set_handler(
    //     move || {
    //         info!("Stopping the app!");
    //         for w in workers.iter_mut() { w.stop(); }
    //         cron.stop();
    //         // cl.lock().unwrap().stop();
    //     }
    // ).expect("Error setting Ctrl-C handler");

    info!("Entering the loop..");
    client.lock().unwrap().do_loop();

    info!("KOHEU.");
    Ok(())
}
