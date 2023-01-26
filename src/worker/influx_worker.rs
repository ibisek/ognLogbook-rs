
use std::time::Duration;
use std::sync::Arc;

use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam::channel::{unbounded, Sender, Receiver};
use chrono::{DateTime, Utc, NaiveDateTime};
use log::{debug, error};
use rinfluxdb::line_protocol::{LineBuilder, Line};
use rinfluxdb::line_protocol::blocking::Client;
use url::Url;

use ogn_client::data_structures::AircraftBeacon;

use crate::configuration::{INFLUX_SERIES_NAME, get_influx_url, get_influx_db_name};

// #[derive(InfluxDbWriteable, Clone)]
#[derive(Clone)]
pub struct Position {
    pub time: DateTime<Utc>,
    pub addr: String,
    pub agl: i32,
    pub alt: i32,
    pub gs: u32,
    pub lat: f64,
    pub lon: f64,
    pub tr: f64,
    pub vs: f64,
}

pub struct InfluxWorker {
    thread: Option<thread::JoinHandle<()>>,
    do_run: Arc<AtomicBool>,
    sender: Sender<AircraftBeacon>,
    receiver: Receiver<AircraftBeacon>,
}

impl InfluxWorker {
    pub fn new() -> InfluxWorker {
        let influx_url = get_influx_url();
        let influx_db_name = get_influx_db_name();
        debug!("InfluxDb at {influx_url}/{influx_db_name}");

        let (sender, receiver) = unbounded::<AircraftBeacon>();
        Self {
            thread: None,
            do_run: Arc::new(AtomicBool::new(true)),
            sender,
            receiver,
        }
    }

    fn influx_connect() -> Client {
        Client::new(Url::parse(&get_influx_url()).unwrap(), Some(("", ""))).unwrap()
    }

    pub fn stop(&mut self) {
        self.do_run.swap(false, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("joining the thread");
        }
    }

    pub fn start(&mut self) {
        if self.thread.is_some() {
            println!("[WARN] Refused to start influx_worker thread. The thread is already running!");
            return;
        }

        // vars used by the thread internally:
        let do_run = Arc::clone(&self.do_run);
        let mut influx_db_client = InfluxWorker::influx_connect();
        let incoming = self.receiver.clone();


        let thread = thread::spawn(move || {

            // let mut start_ts = Instant::now();  //Utc::now().timestamp();
            // let mut beacon_counter = 0;
            let mut lines: Vec<Line> = Vec::new();

            let mut tag = 0;
            while do_run.load(Ordering::Relaxed) {
                let beacon = incoming.recv();

                if beacon.is_err() {
                    thread::sleep(Duration::from_secs(1));      
                    continue;
                }

                let beacon = beacon.unwrap();
                let pos = beacon_into_position(beacon);

                // time                addr      agl alt gs lat       lon       tr vs
                // 1655046041000000000 OGN414931 0   504 0  49.368367 16.114133 0  0
                let line = LineBuilder::new(INFLUX_SERIES_NAME)
                    .insert_field("time", pos.time.timestamp_nanos())
                    .insert_field("addr", pos.addr)
                    .insert_field("agl", pos.agl as i64)
                    .insert_field("alt", pos.alt as i64)
                    .insert_field("gs", pos.gs as i64)
                    .insert_field("lat", pos.lat)
                    .insert_field("lon", pos.lon)
                    .insert_field("tr", pos.tr)
                    .insert_field("vs", pos.vs)
                    .insert_tag("tag", format!("{tag}"))
                    .build();
                
                // println!("[INFO] line: {}", line);
                tag += 1;   // TAG is here to allow multi-line / batch write into influx. It doesn't work without that!
                
                lines.push(line);
                if lines.len() >= 5000 || !do_run.load(Ordering::Relaxed) {    // https://docs.influxdata.com/influxdb/v2.1/write-data/best-practices/optimize-writes/
                    match influx_db_client.send(&get_influx_db_name(), &lines) {
                        Ok(_) => { lines.clear(); },
                        Err(e) => { 
                            error!("upon influx send: {:?}", e);
                            influx_db_client = InfluxWorker::influx_connect();
                        },
                    };
                }

                // beacon_counter += 1;

                // let elapsed = start_ts.elapsed();
                // if elapsed.as_secs() >= 60 {
                //     info!("InfluxWorker beacon rate: {}/min", beacon_counter);
                    
                //     start_ts = Instant::now();
                //     beacon_counter = 0;
                // }
            }
        });

        self.thread = Some(thread);
    }

    /// Enqueues a beacon for influx insertion.
    pub fn store(&mut self, beacon: &AircraftBeacon) {
        match self.sender.send(beacon.clone()) {
            Ok(_) => (),
            Err(e) => error!("When storing a beacon: {:?}", e),
        }
    }

}

pub(crate) fn beacon_into_position(beacon: AircraftBeacon) -> Position{
    // time                addr      agl alt gs lat       lon       tr vs
    // 1655046041000000000 OGN414931 0   504 0  49.368367 16.114133 0  0

    let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(beacon.ts as i64, 0).unwrap(), Utc);

    let position = Position {
        time: dt,
        addr: format!("{}{}", beacon.addr_type.as_long_str(), beacon.addr),
        agl: 0,
        alt: beacon.altitude,
        gs: beacon.speed,
        lat: beacon.lat,
        lon: beacon.lon,
        tr: beacon.turn_rate,
        vs: beacon.climb_rate,
    };

    position
}

