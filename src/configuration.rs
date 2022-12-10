use std::env;

pub const OGN_USERNAME: &str = "rustbook";

pub const OGN_APRS_FILTER_LAT: f64 = 49.1234;
pub const OGN_APRS_FILTER_LON: f64 = 16.4567;
pub const OGN_APRS_FILTER_RANGE: u32 = 999999;

pub const GEOTIFF_FILEPATH: &str = "./data/mosaic-500m.TIF";

pub const AIRFIELDS_FILEPATH: &str = "./data/airfields.json";

pub const REDIS_RECORD_EXPIRATION: usize = 8*60*60;   // [s]

const REDIS_URL: &str = "redis://127.0.0.1:6379/";
pub fn get_redis_url() -> String {
    // let redis_url = env!("REDIS_URL", "Please set $REDIS_URL");
    let redis_url = env::var("REDIS_URL").unwrap_or(REDIS_URL.to_string());
   
    return redis_url
}
