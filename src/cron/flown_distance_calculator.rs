
use log::{info, warn};

use mysql::Pool;
use mysql::Row;
use mysql::prelude::Queryable;
use rinfluxdb::influxql::blocking::Client;
use rinfluxdb::dataframe::DataFrame;
use rinfluxdb::influxql::Query;
use rinfluxdb_influxql::ClientError;
use url::Url;

use crate::configuration::{INFLUX_DB_NAME, INFLUX_SERIES_NAME, get_influx_url, get_db_url};

use ogn_client::data_structures::AddressType;
use crate::airfield_manager::AirfieldManager;
// use crate::airfield_manager::AirfieldManager;

// pub const INFLUX_DB_URL: &str = "http://localhost:8086";
// pub const INFLUX_DB_NAME: &str = "ogn_logbook";
// pub const INFLUX_SERIES_NAME: &str = "pos";

#[derive(Debug)]
struct LogbookEntry {
    id: u64, 
    addr: String, 
    addr_type: AddressType, 
    takeoff_ts: i64, 
    landing_ts: i64,
    flown_dist: u64,
}

pub struct FlownDistanceCalculator {}

impl FlownDistanceCalculator {
    /// @param addr: ogn ID with prefix OGN/ICA/FLR
    fn calc_flown_distance(addr: &str, start_ts: i64, end_ts: i64) -> f64 {
        let influx_db_client = Client::new(Url::parse(&get_influx_url()).unwrap(), Some(("", ""))).unwrap();

        let q= format!("SELECT lat, lon FROM {INFLUX_DB_NAME}..{INFLUX_SERIES_NAME} WHERE addr='{addr}' AND time >= {start_ts}000000000 AND time <= {end_ts}000000000 LIMIT 10");
        let query = Query::new(q);
        let res: Result<DataFrame, ClientError> = influx_db_client.fetch_dataframe(query);

        if res.is_err() {
            warn!("No influx data for '{addr}' between {start_ts} and {end_ts}.");
            return 0_f64;
        }

        let df = res.unwrap();

        println!("DF:{}", df);

        let s = df.to_string();
        let rows = s.split("\n").collect::<Vec<&str>>();
        let mut prev_lat = 0_f64;
        let mut prev_lon = 0_f64;
        let mut total_dist = 0_f64;
        for (i, row) in rows.iter().enumerate() {
            if i < 2 || row.len() != 61 { continue }

            // println!("R [{i}] {row} {}", row.len());
            // let dt = &row[0..23].trim();
            let lat = *(&row[23..41].trim().parse::<f64>().unwrap().to_radians());
            let lon = *(&row[42..60].trim().parse::<f64>().unwrap().to_radians());
            if prev_lat == 0_f64 && prev_lon == 0_f64 {
                prev_lat = lat;
                prev_lon = lon;
                continue;
            }

            let dist = AirfieldManager::get_distance_in_km(prev_lat, prev_lon, lat, lon);
            total_dist += dist;

            prev_lat = lat;
            prev_lon = lon;
        }
        
        total_dist
    }

    pub fn calc_distances() {
        let mut update_sqls: Vec<String> = Vec::new();

        let str_sql = "SELECT e.id, e.address, e.address_type, e.takeoff_ts, e.landing_ts
        FROM logbook_entries as e
        WHERE e.flown_distance is null 
            AND e.address is not null AND e.address_type is not null AND e.takeoff_ts is not null AND e.landing_ts is not null 
        LIMIT 100";

        let binding = get_db_url();
        let db_url = binding.as_str();
        let pool = Pool::new(db_url).expect("Could not connect to MySQL db!");
        let mut conn = pool.get_conn().unwrap();

        let entries: Vec<LogbookEntry> = conn.query_map(str_sql, 
            |mut row: Row| {
                LogbookEntry {
                    id: row.take(0).unwrap(),
                    addr: row.take(1).unwrap(),
                    addr_type: AddressType::from_short_str(row.take(2).unwrap()),
                    takeoff_ts: row.take(3).unwrap(),
                    landing_ts: row.take(4).unwrap(),
                    flown_dist: 0,
                }
            }
        ).unwrap();

        for entry in entries {
            let addr = format!("{}{}", entry.addr_type.as_long_str(), entry.addr);
            let dist = FlownDistanceCalculator::calc_flown_distance(&addr, entry.takeoff_ts, entry.landing_ts);

            // save it even if the dist was 0 .. 0 will signalise there was no flight data available; null = to be still calculated
            let update_sql = format!("UPDATE logbook_entries SET flown_distance={} WHERE id = {};", dist.round(), entry.id);
            update_sqls.push(update_sql);
        }

        if update_sqls.len() > 0 {
            info!("Updated {} flown distance(s)", update_sqls.len());

            for sql in update_sqls {
                conn.query_drop(sql).unwrap();
            }
        }

    }

}
