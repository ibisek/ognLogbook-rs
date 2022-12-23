
use std::vec;

use chrono::Utc;
use log::{info, warn};
use mysql::prelude::Queryable;
use mysql::Row;
use rinfluxdb::influxql::Query;
use rinfluxdb_influxql::ClientError;
use ogn_client::data_structures::{AddressType, AircraftType};

use crate::airfield_manager::AirfieldManager;
use crate::configuration::{INFLUX_DB_NAME, AIRFIELDS_FILEPATH, REDIS_RECORD_EXPIRATION};
use crate::db::dataframe::DataFrame;
use crate::db::mysql::MySQL;
use crate::db::influxdb;
use crate::db::redis;
use crate::db::data_structures::LogbookEvent;


pub struct RedisReaper {}

pub const RR_RUN_INTERVAL: u64 = 5*60;  // [s]
// pub const _RR_STALE_INTERVAL_1: i64 = 1 * 60 * 60;  // [s]
pub const RR_STALE_INTERVAL_2: i64 = 2 * 60 * 60;  // [s]
// pub const RR_TTL_LIMIT: i64 = REDIS_RECORD_EXPIRATION as i64 - _RR_STALE_INTERVAL_1;
pub const RR_GS_THRESHOLD: i64 = 20;    // [km/h] glider

impl RedisReaper {

    fn find_most_recent_takeoff(mysql: &mut MySQL, address: &str, address_type: AddressType) -> Option<LogbookEvent> {
        let addr_type = address_type.as_short_str();

        let sql = format!("SELECT id, ts, address, address_type, aircraft_type, lat, lon, location_icao 
            FROM logbook_events 
            WHERE address = '{address}' AND address_type='{addr_type}' AND event='T' 
            ORDER by ts DESC LIMIT 1;");

        let entries: Vec<LogbookEvent> = mysql.get_connection().query_map(sql, 
            |mut row: Row| {

                let mut takeoff_icao = "".to_string();  // jebacka s prevodem NULL v db na neco pouzitelneho tady
                let val = row.take_opt("location_icao");
                if let Some(val) = val {
                    match val {
                        Ok(val) => takeoff_icao = val,
                        Err(_) => {takeoff_icao = "".to_string()}
                    }
                }

                LogbookEvent {
                    id: row.take("id").unwrap(),
                    ts: row.take("ts").unwrap(),
                    event: "T".into(),
                    address: row.take("address").unwrap(),
                    address_type: AddressType::from_short_str(row.take("address_type").unwrap()),
                    aircraft_type: AircraftType::from(row.take("aircraft_type").unwrap()),
                    lat: row.take("lat").unwrap(),
                    lon: row.take("lon").unwrap(),
                    location_icao: takeoff_icao,
                }
            }
        ).unwrap();

        
        for entry in entries {
            return Some(entry);
        }

        None
    }

    pub fn do_work() {
        let mut redis = redis::get_client();
        let mut mysql = MySQL::new();
        let influx_db_client = influxdb::get_client();
        let airfield_manager = AirfieldManager::new(AIRFIELDS_FILEPATH);

        // list all airborne airplanes:
        let res = redis.keys("*status");
        let status_keys = match res {
            Ok(keys) => keys,
            Err(_) => vec![],
        };  // key in form address-status

        let mut airborne:Vec<String> = vec![];
        for status_key in status_keys {
            let rec: String = match redis.get(&status_key) {    
                Ok(str) => str,
                Err(_) => "".into(),
            };  // rec in form "status;ts" where status 1 == airborne

            if !rec.starts_with("1") { continue; }

            let addr = status_key.split("-").collect::<Vec<&str>>()[0]; // in fact addressTypeStr + addr (e.g. I123456, F123456, O123456, ..)
            airborne.push(addr.into());
        }

        let mut num_landed = 0;

        for addr in airborne {
            let prefix = &addr[..1];
            let addr = &addr[1..];
            let addr_type = AddressType::from_short_str(prefix.into());
            let addr_prefix_long = addr_type.as_long_str();

            // get last received beacon:
            let q = format!("SELECT agl, gs, lat, lon FROM {INFLUX_DB_NAME}..pos WHERE addr='{addr_prefix_long}{addr}' ORDER BY time DESC LIMIT 1;");
            let query = Query::new(&q);
            let res: Result<DataFrame, ClientError> = influx_db_client.fetch_dataframe(query);
            if res.is_err() {
                // warn!("RR: no last position in influx for '{addr}'.");
                continue;
            }

            let df = res.unwrap();
            // println!("DF:{}", df);
            let cols = df.columns;
            let ts = df.index[0].timestamp();    // utc ts
            let agl = cols.get("agl").unwrap().get_int_value(0).unwrap_or(0);
            let gs = cols.get("gs").unwrap().get_int_value(0).unwrap_or(0);
            let lat = cols.get("lat").unwrap().get_float_value(0).unwrap_or(0_f64);
            let lon = cols.get("lon").unwrap().get_float_value(0).unwrap_or(0_f64);

            let mut landing_suspected = false;
            if agl > 0 && agl < 100 && gs < RR_GS_THRESHOLD {
                landing_suspected = true;
            } else {
                let last_position_age = Utc::now().timestamp() - ts;
                if last_position_age > RR_STALE_INTERVAL_2 {
                    landing_suspected = true;
                }
            }

            if landing_suspected {
                // set status as onGround (0) in redis (or delete?):
                let key = format!("{prefix}{addr}-status");
                redis.set(&key, "0;0").unwrap();   // 0 = on-ground; ts=0 to indicate forced landing
                redis.expire(&key, REDIS_RECORD_EXPIRATION).unwrap();


                // look-up related takeoff record:
                let takeoff_event = RedisReaper::find_most_recent_takeoff(&mut mysql, addr, addr_type);
                if takeoff_event.is_some() {    // create a LANDING logbook_event -> a stored procedure then creates a logbook_entry (flight)
                    let takeoff_event = takeoff_event.unwrap();
                    // println!("TE: {:?}", takeoff_event);

                    let mut flight_time = ts - takeoff_event.ts;
                    if flight_time < 0 { flight_time = 0; };

                    let mut icao_location = takeoff_event.location_icao;
                    if icao_location == "" {
                        icao_location = match airfield_manager.get_nearest(lat, lon) {
                            Some(loc) => loc,
                            None => "".into(),
                        };
                    }

                    let location_icao_sql = if icao_location != "" { format!("'{icao_location}'") } else { "null".into() };
                    
                    let sql = format!("INSERT INTO logbook_events (ts, address, address_type, aircraft_type, event, lat, lon, location_icao, flight_time)
                        VALUES 
                        ({ts}, '{addr}', '{}', {}, 'L', {lat:.5}, {lon:.5}, {location_icao_sql}, {flight_time});",
                        takeoff_event.address_type.as_short_str(), takeoff_event.aircraft_type.value());

                    mysql.get_connection().exec_drop(sql, ()).unwrap();

                    // TODO..
                    // addrTypeNum = REVERSE_ADDRESS_TYPE.get(addrType, 1)
                    // EventWatcher.createEvent(redis=self.redis,
                    //                             ts=localTs, event='L', address=addr, addressType=addrTypeNum,
                    //                             lat=lat, lon=lon, icaoLocation=icaoLocation, flightTime=flightTime)

                    num_landed += 1;
                }
            }
        }

        if num_landed > 0 {
            info!("RedisReaper: cleared {num_landed} stale records");
        }
    }

}