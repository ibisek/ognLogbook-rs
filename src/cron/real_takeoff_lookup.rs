
use chrono::Utc;
use log::{info, warn, error};
use mysql::Row;
use mysql::prelude::Queryable;
use rinfluxdb::influxql::blocking::Client;
use rinfluxdb::influxql::Query;
use rinfluxdb_influxql::ClientError;
use url::Url;

use ogn_client::data_structures::{AddressType, AircraftType};

use crate::configuration::{INFLUX_DB_NAME, INFLUX_SERIES_NAME, get_influx_url, get_db_url};
use crate::db::mysql::MySQL;
use crate::db::dataframe::{Column, DataFrame};

#[derive(Debug, Clone)]
struct LogbookItem {
    id: u64, 
    addr: String, 
    addr_type: AddressType, 
    
    takeoff_ts: i64, 
    takeoff_lat: f64, 
    takeoff_lon: f64, 
    takeoff_icao: String,
                 
    landing_ts: i64,
    landing_lat: f64, 
    landing_lon: f64, 
    landing_icao: String,
                 
    flight_time: i64, 
    flown_distance: u64, 
    device_type: String,
                 
    registration: String, 
    cn: String, 
    aircraft_type: AircraftType, 
    tow_id: i64,
}

impl LogbookItem {
    pub fn new(id: u64, addr: String, addr_type: AddressType, takeoff_ts: i64, takeoff_icao: String) -> LogbookItem {
        LogbookItem { 
            id, 
            addr, 
            addr_type,

            takeoff_ts, 
            takeoff_lat: 0_f64, 
            takeoff_lon: 0_f64, 
            takeoff_icao: "".into(), 

            landing_ts: 0_i64, 
            landing_lat: 0_f64, 
            landing_lon: 0_f64, 
            landing_icao: "".into(), 
            
            flight_time: 0_i64, 
            flown_distance: 0_u64, 
            device_type: "".into(), 
            
            registration: "".into(), 
            cn: "".into(), 
            aircraft_type: AircraftType::Unknown, 
            tow_id: 0_i64,
         }
    }
}

pub const RTL_RUN_INTERVAL: u64 = 60;    // [s]

pub struct RealTakeoffLookup {
}

impl RealTakeoffLookup {

    fn list_takeoffs(ts: i64, mysql: &mut MySQL) -> Vec<LogbookItem> {
        let sql = format!("SELECT id, address, address_type, ts, location_icao FROM logbook_events WHERE ts >= {} AND event='T';", ts - RTL_RUN_INTERVAL as i64);    // RUN_INTERVAL = 60

        let mut conn = mysql.get_connection();

        let entries: Vec<LogbookItem> = conn.query_map(sql, 
            |mut row: Row| {
                let id = row.take("id").unwrap();
                let addr = row.take("address").unwrap();
                let addr_type = AddressType::from_short_str(row.take("address_type").unwrap());
                let ts = row.take("ts").unwrap();

                let mut takeoff_icao = "".to_string();  // jebacka s prevodem NULL v db na neco pouzitelneho tady
                let val = row.take_opt("location_icao");
                if let Some(val) = val {
                    match val {
                        Ok(val) => takeoff_icao = val,
                        Err(_) => {takeoff_icao = "".to_string()}
                    }
                }

                LogbookItem::new(id, addr, addr_type, ts, takeoff_icao)
            }
        ).unwrap();
        
        entries
    }

    pub fn check_takeoffs() {
        let mut mysql = MySQL::new();

        let ts = Utc::now().timestamp();
        let takeoffs = RealTakeoffLookup::list_takeoffs(ts, &mut mysql);

        let influx_db_client = Client::new(Url::parse(&get_influx_url()).unwrap(), Some(("", ""))).unwrap();

        let mut num_modified_takeoffs = 0_u64;
        for logbook_item in takeoffs.iter() {
            let addr = format!("{}{}", logbook_item.addr_type.as_long_str(), logbook_item.addr);
            let window_end_ts = logbook_item.takeoff_ts - 2;    // [s]
            let window_start_ts = window_end_ts - 59;           // [s]

            // get flight data from influx:
            let q = format!("SELECT lat, lon, gs FROM {INFLUX_DB_NAME}..{INFLUX_SERIES_NAME} WHERE addr='{addr}' AND time >= {window_start_ts}000000000 AND time <= {window_end_ts}000000000 ORDER BY time DESC");
            let query = Query::new(q);
            let res: Result<DataFrame, ClientError> = influx_db_client.fetch_dataframe(query);

            if res.is_err() {
                // warn!("RTL: no influx data for '{addr}' between {window_start_ts} and {window_end_ts}.");
                continue;
            }

            let df = res.unwrap();

            let index = df.index;

            let cols = df.columns;
            let latitudes: &Column = cols.get("lat").unwrap();
            let longitudes = cols.get("lon").unwrap();
            let ground_speeds = cols.get("gs").unwrap();

            // find minimal ground speed index:
            let mut dirty = false;
            let mut min_gs = i64::MAX;
            let mut min_gs_index = 0;
            for i in 0..index.len() {
                let gs = ground_speeds.get_int_value(i).unwrap();
                if gs <= min_gs {
                    min_gs = gs;
                    min_gs_index = i;
                    dirty = true;

                    if gs <= 40_i64 {   // TODO getGroundSpeedThreshold(logbookItem.aircraft_type, forEvent='T'):
                        break;
                    } 
                }
            }

            if dirty {
                let takeoff_ts = index[min_gs_index].timestamp();
                let lat = latitudes.get_float_value(min_gs_index).unwrap();
                let lon = longitudes.get_float_value(min_gs_index).unwrap();

                if logbook_item.takeoff_icao == "" {
                    //TODO dohledat
                } 

                let location_icao_sql = if logbook_item.takeoff_icao != "" { format!("'{}'", logbook_item.takeoff_icao) } else { "null".into() };

                let update_sql = format!("UPDATE logbook_events SET ts={}, lat={:.5}, lon={:.5}, location_icao={location_icao_sql} WHERE id={};", takeoff_ts, lat, lon, logbook_item.id);

                let mut conn = mysql.get_connection();
                match conn.query_drop(&update_sql) {
                    Ok(_) => (),
                    Err(e) => error!("Error when executing query '{update_sql}': {e}")
                };

                num_modified_takeoffs += 1;
            }
        }

        if num_modified_takeoffs > 0 {
            info!("Num take-off amendments: {num_modified_takeoffs}/{}", takeoffs.len());
        }
    }
}

