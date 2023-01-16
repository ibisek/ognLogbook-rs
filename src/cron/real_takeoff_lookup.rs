
use chrono::Utc;
use log::{info, error};
use mysql::Row;
use mysql::prelude::Queryable;
use rinfluxdb::influxql::blocking::Client;
use rinfluxdb::influxql::Query;
use rinfluxdb_influxql::ClientError;
use url::Url;

use ogn_client::data_structures::AddressType;

use crate::airfield_manager::{AirfieldManager, self};
use crate::configuration::{AIRFIELDS_FILEPATH, INFLUX_SERIES_NAME, get_influx_url, get_influx_db_name};
use crate::db::mysql::MySQL;
use crate::db::dataframe::{Column, DataFrame};
use crate::db::data_structures::LogbookItem;

use super::CronJob;

pub const RTL_RUN_INTERVAL: u64 = 60;    // [s]

pub struct RealTakeoffLookup {}

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
        let airfield_manager = AirfieldManager::new(AIRFIELDS_FILEPATH);    // (50ms) !!ultra inefficient to read & parse the file every time!!
        let influx_db_name = get_influx_db_name();

        let ts = Utc::now().timestamp();
        let mut takeoffs = RealTakeoffLookup::list_takeoffs(ts, &mut mysql);

        let influx_db_client = Client::new(Url::parse(&get_influx_url()).unwrap(), Some(("", ""))).unwrap();

        let mut num_modified_takeoffs = 0_u64;
        for logbook_item in takeoffs.iter_mut() {
            let addr = format!("{}{}", logbook_item.addr_type.as_long_str(), logbook_item.addr);
            let window_end_ts = logbook_item.takeoff_ts - 2;    // [s]
            let window_start_ts = window_end_ts - 59;           // [s]

            // get flight data from influx:
            let q = format!("SELECT lat, lon, gs FROM {influx_db_name}..{INFLUX_SERIES_NAME} WHERE addr='{addr}' AND time >= {window_start_ts}000000000 AND time <= {window_end_ts}000000000 ORDER BY time DESC");
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
                logbook_item.takeoff_ts = index[min_gs_index].timestamp();
                logbook_item.takeoff_lat = latitudes.get_float_value(min_gs_index).unwrap_or(0_f64);
                logbook_item.takeoff_lon = longitudes.get_float_value(min_gs_index).unwrap_or(0_f64);

                if logbook_item.takeoff_icao == "" {
                    let takeoff_location = airfield_manager.get_nearest(logbook_item.takeoff_lat, logbook_item.takeoff_lon);
                    if takeoff_location.is_some() {
                        logbook_item.takeoff_icao = takeoff_location.unwrap();
                    }
                } 

                let location_icao_sql = if logbook_item.takeoff_icao != "" { format!("'{}'", logbook_item.takeoff_icao) } else { "null".into() };

                let update_sql = format!("UPDATE logbook_events SET ts={}, lat={:.5}, lon={:.5}, location_icao={location_icao_sql} WHERE id={};", 
                    logbook_item.takeoff_ts, logbook_item.takeoff_lat, logbook_item.takeoff_lon, logbook_item.id);

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

    // pub fn cron() -> impl Fn() -> () {
    //     let af = AirfieldManager::new(AIRFIELDS_FILEPATH);

    //     return move || -> () {
    //         RealTakeoffLookup::check_takeoffs(&af);
    //     };
    // }

}

// impl CronJob for RealTakeoffLookup {
//     fn cron(&self) -> () {
//         self.check_takeoffs();
//     }
// }
