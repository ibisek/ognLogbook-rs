use std::collections::HashMap;
use std::fmt;
use std::fs;

use log::{info};

// #[derive(Deserialize, Debug)]
#[derive(Clone, Debug)]
pub struct AirfieldRecord {
    code: String,
    lat: f64,
    lon: f64,
}

impl AirfieldRecord {

    ///lat + lon in degrees
    pub fn new(code: &str, lat: f64, lon:f64) -> AirfieldRecord {
        AirfieldRecord { 
            code: code.to_string(),
            lat: lat.to_radians(),
            lon: lon.to_radians(),
        }
    }
}

impl fmt::Display for AirfieldRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#AirfieldRecord: {0}; lat:{1:.4}; lon:{2:.4}", self.code, self.lat.to_degrees(), self.lon.to_degrees())
    }
}

pub struct AirfieldManager {
    airfields_in_quadrants: HashMap<i32, HashMap<i32, Vec<AirfieldRecord>>>,
}

impl AirfieldManager {
    pub fn new(filepath: &str) -> AirfieldManager {
        info!("Reading airfields from '{filepath}'");

        let mut airfields: Vec<AirfieldRecord> = AirfieldManager::load_airfields_from_file(filepath);
        airfields.sort_by(|a, b| a.lat.total_cmp(&b.lat) );

        let airfields_in_quadrants = AirfieldManager::split_airfields_into_quadrants(airfields);

        AirfieldManager {
            airfields_in_quadrants,
        }
    }

    fn load_airfields_from_file(filepath: &str) -> Vec<AirfieldRecord> {
        let data = fs::read_to_string(filepath).expect(&format!("Could not read '{filepath}'!"));
        let json: serde_json::Value = serde_json::from_str(&data).expect(&format!("Could not parse json from '{filepath}'!"));
        // let json: &Vec<AirfieldRecord> = serde_json::from_str(&data).unwrap();

        let mut airfields: Vec<AirfieldRecord> = Vec::new();

        for item in json.as_array().unwrap().into_iter() {
            let ar = AirfieldRecord::new(
                &item["code"].to_string(), 
                item["lat"].to_string().parse().unwrap(), 
                item["lon"].to_string().parse().unwrap()
            );

            airfields.push(ar);
        }

        airfields
    }

    /// all params in radians!
    pub fn get_distance_in_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let arg = lat1.sin() * lat2.sin() + lat1.cos() * lat2.cos() * (lon2 - lon1).cos();
        if arg >= 1.0 {
            return 0_f64
        }

        const R: f64 = 6371.0;   // [km]
        let dist = arg.acos() * R;

        return dist
    }

    fn split_airfields_into_quadrants(airfields: Vec<AirfieldRecord>) -> HashMap<i32, HashMap<i32, Vec<AirfieldRecord>>> {
        let mut airfields_quads = HashMap::from([
            (1, HashMap::from([
                (1, Vec::<AirfieldRecord>::new()),
                (-1, Vec::<AirfieldRecord>::new()),
            ])),
            (-1, HashMap::from([
                (1, Vec::<AirfieldRecord>::new()),
                (-1, Vec::<AirfieldRecord>::new()),
            ])),
        ]);

        for af in airfields.into_iter() {
            let lat_sign = if af.lat >= 0.0 {1}  else {-1};
            let lon_sign = if af.lon >= 0.0 {1}  else {-1};
            airfields_quads.get_mut(&lat_sign).unwrap().get_mut(&lon_sign).unwrap().push(af);
        }
    
        airfields_quads
    }

    /// arguments in degrees
    pub fn get_nearest(&self, lat: f64, lon:f64) -> Option<String> {
        let lat_rad = lat.to_radians();
        let lon_rad = lon.to_radians();

        // pick the appropriate airfields list (NE / NW / SE / SW):
        let lat_sign = if lat_rad >= 0_f64 {1} else {-1};
        let lon_sign = if lon_rad >= 0_f64 {1} else {-1};
        let airfields = self.airfields_in_quadrants.get(&lat_sign).unwrap().get(&lon_sign).unwrap();

        let mut start_i = 0;
        let mut end_i = airfields.len();
        let mut n = 0;
        loop {
            let i = start_i + ((end_i - start_i) / 2) as usize;
            if lat_rad < airfields.get(i).unwrap().lat {
                end_i = i;
            } else {
                start_i = i;
            }

            if end_i - start_i <= 100 {
                break;
            }

            n += 1;
            if n > 100 {
                break;
            }
        }

        let mut min_dist = 99999999999999_f64;
        let mut code: Option<String> = None;
        for rec in airfields[start_i..end_i+1].into_iter() {
            let dist = AirfieldManager::get_distance_in_km(lat_rad, lon_rad, rec.lat, rec.lon);
            if dist < min_dist {
                min_dist = dist;
                code = Some(rec.code.clone());
            }
        }

        code

    }

}