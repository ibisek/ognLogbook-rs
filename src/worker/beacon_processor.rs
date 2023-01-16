use chrono::prelude::*;
use log::{debug, info, error};
use simple_redis::client::Client;
use simple_redis::RedisResult;

use ogn_client::data_structures::{AircraftBeacon, AircraftType, AddressType};

use crate::configuration::{GEOTIFF_FILEPATH, REDIS_RECORD_EXPIRATION, AIRFIELDS_FILEPATH, get_db_url, AGL_LANDING_LIMIT};
use crate::airfield_manager::AirfieldManager;
use crate::db::redis;
use crate::worker::data_structures::{AircraftStatus, AircraftStatusWithTs};
use crate::worker::geo_file::GeoFile;
use crate::worker::db_thread::DbThread;
use crate::worker::expiring_dict::ExpiringDict;
use crate::worker::utils::get_groundspeed_threshold;
use crate::worker::influx_worker::InfluxWorker;

static UNSUPPORTED_CRAFTS: [AircraftType; 7] = [AircraftType::Undefined, AircraftType::Unknown, AircraftType::Baloon, AircraftType::Airship, AircraftType::Uav, AircraftType::Reserved, AircraftType::Obstacle];

pub struct BeaconProcessor {
    geo_file: GeoFile,
    redis: Client,
    airfield_manager: AirfieldManager,
    db_thread: DbThread,
    beacon_duplicate_cache:ExpiringDict<String, bool>,
    influx_worker: InfluxWorker,
    t: i64,
}

impl BeaconProcessor {

    pub fn new() -> BeaconProcessor {
        let mut db_thread = DbThread::new(&get_db_url());
        db_thread.start();

        let mut influx_worker = InfluxWorker::new();
        influx_worker.start();

        BeaconProcessor { 
            geo_file: GeoFile::new(GEOTIFF_FILEPATH), 
            redis: redis::get_client(),
            airfield_manager: AirfieldManager::new(AIRFIELDS_FILEPATH),
            db_thread: db_thread,
            beacon_duplicate_cache: ExpiringDict::new(1000),
            influx_worker,
            t: 0,
        }
    }

    fn get_agl(&mut self, beacon: &AircraftBeacon) -> Option<i32> {
        let terrain_elevation = self.geo_file.get_value(beacon.lat, beacon.lon);

        match terrain_elevation {
            Some(e) => {
                let mut agl = beacon.altitude - (e as i32);
                if agl < 0 {
                    agl = 0;
                }
                return Some(agl);
            },
            None => None,
        }
    }

    fn get_from_redis(&mut self, key: &String) -> RedisResult<String> {
        // match self.redis.get::<String>(&key) {
        //     Ok(val) => val,
        //     Err(_) => "".to_string()
        // }
        self.redis.get::<String>(&key)
    }

    fn save_to_redis(&mut self, key: &String, value: &String, expiration: usize) {
        // self.redis.set(&key, as_redis_arg!(value));  // TODO az jim to bude jednou fungovat
        match self.redis.run_command::<String>("SET", vec![&key, &value]) {
            Ok(_) => (),
            Err(e) => { error!("upon redis set: {:?}", e); },
        };
        match self.redis.expire(&key, expiration) {
            Ok(_) => (),
            Err(e) => { error!("upon redis expiration: {:?}", e); },
        };
    }

    fn del_in_redis(&mut self, key: &str) {
        match self.redis.del(key) {
            Ok(_) => (),
            Err(e) => { error!("upon redis del: {:?}", e); },
        };
    }

    fn xstart(&mut self, addr_type: &AddressType) {
        if AddressType::Icao.eq(addr_type) {
            self.t = Utc::now().timestamp_micros();
        }
    }

    fn xstop(&mut self, addr_type: &AddressType, label: &str) {
        if AddressType::Icao.eq(addr_type) {
            let now = Utc::now().timestamp_micros(); 
            let dur = now - self.t;
            if dur > 1000 {
                // println!("TT {label} {dur} us");
            }
            self.t = now;
        }
    }

    pub fn process(&mut self, beacon: &mut AircraftBeacon) {
        //we are not interested in para, baloons, uavs and other crazy flying stuff:
        if UNSUPPORTED_CRAFTS.contains(&beacon.aircraft_type) {
            debug!("Skipping AT: {}", &beacon.aircraft_type);
            return;
        }

        // println!("beacon: {beacon}");
        let ts = beacon.ts as i64; // UTC [s]
        let now = chrono::offset::Utc::now().timestamp();
        if ts - now > 120 {
            debug!("Timestamp from the future for {}: {ts}, now is {now} ({}s)", &beacon.addr, ts-now);
            return;
        }

        self.xstart(&beacon.addr_type);
        // get altitude above ground level (AGL):
        let agl = self.get_agl(&beacon);
        if agl.is_some() {
            beacon.set_agl(agl.unwrap());
        }
        self.xstop(&beacon.addr_type,"U1");

                
        // store the beacon into influxdb:
        self.influx_worker.store(&beacon);
        self.xstop(&beacon.addr_type,"U2");

        let addres_type_c = beacon.addr_type.as_short_str();
        let address = &beacon.addr;
        
        // skip beacons we received for the second time and got already processed:
        let key = format!("{addres_type_c}{address}-{0:.4}{1:.4}{2}{3:.1}{4:.1}", beacon.lat, beacon.lon, beacon.altitude, beacon.speed, beacon.climb_rate);
        if self.beacon_duplicate_cache.contains_key(&key) {
            return;
        } else {
            self.beacon_duplicate_cache.insert(key, true);  // store a marker in the cache .. will be dropped after TTL automatically later
        };

        if beacon.speed > 400 { // ignore fast (icao) airliners and jets
            return;
        }
        self.xstop(&beacon.addr_type,"U3");

        let status_key = format!("{addres_type_c}{address}-status");
        let prev_status = match self.get_from_redis(&status_key) {
            Ok(ps) => AircraftStatusWithTs::from_redis_str(&ps),
            Err(_) => AircraftStatusWithTs::new(AircraftStatus::Unknown, beacon.ts),
        };
        self.xstop(&beacon.addr_type,"U4");
        
        let mut gs = beacon.speed as f64;  // [km/h]
        let gs_key = format!("{addres_type_c}{address}-gs");
        if prev_status.is(AircraftStatus::Unknown) { // we have no prior information
            // let status = format!("0;{ts}"); // 0 = AircraftStatus::OnGround
            let status = AircraftStatusWithTs::new(AircraftStatus::OnGround, beacon.ts);
            self.save_to_redis(&status_key, &status.as_redis_str(), REDIS_RECORD_EXPIRATION);
            self.save_to_redis(&gs_key, &0.to_string(), 120); // gs = 0
            self.xstop(&beacon.addr_type,"U5");
        }

        let prev_gs = match self.get_from_redis(&gs_key) {
            Ok(gs) => gs.parse().unwrap_or(0_f64),
            Err(_) => 0_f64
        };
        if prev_gs > 0_f64 { // filter speed change a bit (sometimes there are glitches in speed with badly placed gps antenna):
            gs = gs * 0.7 + prev_gs * 0.3;
        }
        if gs > 0_f64 {
            self.save_to_redis(&gs_key, &format!("{:.0}", gs.round()), 3600);
        }
        self.xstop(&beacon.addr_type,"U6");

        let mut current_status = AircraftStatusWithTs::new(AircraftStatus::Unknown, beacon.ts);
        if prev_status.is(AircraftStatus::OnGround) {
            current_status.status = if gs > get_groundspeed_threshold(&beacon.aircraft_type, 'T') { AircraftStatus::Airborne } else { AircraftStatus::OnGround };
        } else {    // when airborne
            current_status.status = if gs <= get_groundspeed_threshold(&beacon.aircraft_type, 'L') { AircraftStatus::OnGround } else { AircraftStatus::Airborne };
        }
        self.xstop(&beacon.addr_type,"U7");

        if current_status.status != prev_status.status {
            let event = if current_status.is(AircraftStatus::OnGround) {'L'} else {'T'}; // L = landing, T = take-off
            let mut flight_time: i64 = 0;

            if event == 'L' {
                flight_time = (current_status.ts - prev_status.ts) as i64;   // [s]
                if flight_time < 120 { return }  // [s]

                if flight_time > 12 * 3600 {    // some relic from the previous day
                    self.del_in_redis(&status_key);
                    self.del_in_redis(&gs_key);
                    return;
                }

                // check altitude above ground level:
                if agl.is_some() && agl.unwrap() > AGL_LANDING_LIMIT { return };    // most likely a false detection

            } else if event == 'T' {
                // check altitude above ground level:
                if agl.is_some() && agl.unwrap() < 50 { return }; // most likely a false detection
            }
            self.xstop(&beacon.addr_type,"U8");

            self.save_to_redis(&status_key, &current_status.as_redis_str(), REDIS_RECORD_EXPIRATION);
            self.xstop(&beacon.addr_type,"U9");

            let icao_location = self.airfield_manager.get_nearest(beacon.lat, beacon.lon);
            self.xstop(&beacon.addr_type,"U10");

            let naive = NaiveDateTime::from_timestamp_opt(beacon.ts as i64, 0).unwrap();
            let dt_str = DateTime::<Utc>::from_utc(naive, Utc).format("%H:%M:%S");
            let icao_location_str = if icao_location.is_some() {icao_location.clone().unwrap()} else {"?".into()};
            let flight_time_str = if flight_time > 0 { format!("{flight_time}s") } else { "".into() };
            info!("EVENT: {dt_str}; loc: {icao_location_str} [{addres_type_c}] {} {event} {flight_time_str}", beacon.addr);

            let icao_location_str = match icao_location {
                Some(loc) => format!("'{loc}'"),
                None => "null".into()
            };

            let str_sql = format!("INSERT INTO logbook_events \
                     (ts, address, address_type, aircraft_type, event, lat, lon, location_icao, flight_time) \
                     VALUES \
                     ({ts}, '{address}', '{addres_type_c}', '{0}', \
                     '{event}', {1:.5}, {2:.5}, {icao_location_str}, {flight_time});", 
                     beacon.aircraft_type.value(), beacon.lat, beacon.lon);
            self.xstop(&beacon.addr_type,"U11");

            // debug!("str_sql: {str_sql}");
            self.db_thread.add_statement(str_sql);
            self.xstop(&beacon.addr_type,"U12");
            // panic!("BREAK!");

            // TODO.. radek 289+

            // icaoLocation = icaoLocation.replace("'", '')
            // EventWatcher.createEvent(redis=self.redis,
            //                          ts=ts, event=event, address=address, addressType=addressType,
            //                          lat=lat, lon=lon, icaoLocation=icaoLocation, flightTime=flightTime)

        }

        self.beacon_duplicate_cache.tick();    // cleanup the cache (cannot be called from PeriodicTimer due to subprocess/threading troubles :|)

    }

}

    