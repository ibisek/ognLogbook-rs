use std::env;
use simplelog::LevelFilter;

pub const LOG_LEVEL: LevelFilter = LevelFilter::Info;

pub const OGN_USERNAME: &str = "rustbook";

pub const OGN_APRS_FILTER_LAT: f64 = 49.1234;
pub const OGN_APRS_FILTER_LON: f64 = 16.4567;
pub const OGN_APRS_FILTER_RANGE: u32 = 999999;

pub const GEOTIFF_FILEPATH: &str = "./data/mosaic-500m.TIF";

pub const AIRFIELDS_FILEPATH: &str = "./data/airfields.json";

pub const AGL_LANDING_LIMIT: i64 = 100; // [m]

const DB_HOST: &str = "localhost";
const DB_PORT: &str = "3306";
const DB_NAME: &str = "ogn_logbook";
const DB_USER: &str = "ibisek";
const DB_PASSWORD: &str = "heslo";
pub fn get_db_url() -> String {
    let db_host = env::var("DB_HOST").unwrap_or(DB_HOST.into());
    let db_port = env::var("DB_PORT").unwrap_or(DB_PORT.into());
    let db_name = env::var("DB_NAME").unwrap_or(DB_NAME.into());
    let db_user = env::var("DB_USER").unwrap_or(DB_USER.into());
    let db_password = env::var("DB_PASSWORD").unwrap_or(DB_PASSWORD.into());
    // url format: "mysql://user:password@host:port/db_name"
    format!("mysql://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}")
}


pub const REDIS_RECORD_EXPIRATION: usize = 8*60*60;   // [s]

const REDIS_URL: &str = "redis://127.0.0.1:6379/";
pub fn get_redis_url() -> String {
    // let redis_url = env!("REDIS_URL", "Please set $REDIS_URL");
    let redis_url = env::var("REDIS_URL").unwrap_or(REDIS_URL.to_string());
   
    return redis_url
}
