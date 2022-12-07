use chrono;
use simple_redis::client::Client;
use simple_redis::RedisResult;

use ogn_client::data_structures::{AircraftBeacon, AircraftType};

use crate::configuration::{GEOTIFF_FILEPATH, REDIS_RECORD_EXPIRATION, get_redis_url};
use crate::worker::data_structures::AircraftStatus;
use crate::worker::geo_file::GeoFile;

pub struct BeaconProcessor {
    geo_file: GeoFile,
    redis: Client,
}

impl BeaconProcessor {

    pub fn new() -> BeaconProcessor {
        BeaconProcessor { 
            geo_file: GeoFile::new(GEOTIFF_FILEPATH), 
            redis: simple_redis::create(&get_redis_url()).unwrap(),
        }
    }

    fn get_agl(&mut self, beacon: &AircraftBeacon) -> i64 {
        let terrain_elevation = self.geo_file.get_value(beacon.lat, beacon.lon);

        match terrain_elevation {
            Some(e) => {
                let mut agl = beacon.altitude as i64 - e;
                if agl < 0 {
                    agl = 0;
                }
                return agl;
            },
            None => 0,
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
        self.redis.run_command::<String>("SET", vec![&key, &value]).unwrap();
        self.redis.expire(&key, expiration).unwrap();   // REDIS_RECORD_EXPIRATION
    }

    pub fn process(&mut self, beacon: &AircraftBeacon) {
        //we are not interested in para, baloons, uavs and other crazy flying stuff:
        if !vec![AircraftType::Undefined, AircraftType::Glider, AircraftType::HangGlider, AircraftType::PistonPlane, AircraftType::JetPlane, AircraftType::Unknown].contains(&beacon.aircraft_type) {
            return;
        }

        println!("beacon: {beacon}");
        let ts = beacon.ts as i64; // UTC [s]
        let now = chrono::offset::Utc::now().timestamp();
        if ts - now > 30 {
            print!("[WARN] Timestamp from the future: {ts}, now is {now}");
            return;
        }

        // skip beacons we received for the second time and got already processed:
        // key = f"{addressTypeStr}{address}-{lat:.4f}{lon:.4f}{altitude}{groundSpeed:.1f}{verticalSpeed:.1f}"
        // if key in self.beaconDuplicateCache:
        //     del self.beaconDuplicateCache[key]
        //     return
        // else:
        //     self.beaconDuplicateCache[key] = True   # store a marker in the cache .. will be dropped after TTL automatically later

        if beacon.speed > 400 { // ignore fast (icao) airliners and jets
            return;
        }

        // get altitude above ground level (AGL):
        let agl = self.get_agl(beacon);

        // TODO insert into influx (done by ogn_short_time_memory thread?)

        let addres_type_c = beacon.addr_type.as_short_str();
        let address = &beacon.addr;
        let status_key = format!("{addres_type_c}{address}-status");
        let prev_status = match self.get_from_redis(&status_key) {
            Ok(ps) => {
                let items = ps.split(";").collect::<Vec<&str>>();   // parse "ps;ts"
                AircraftStatus::from_i8(items[0].parse().unwrap_or(-1)) // -1 = Unknown
            },
            Err(_) => AircraftStatus::Unknown,
        };
        
        let mut gs = beacon.speed as f64;  // [km/h]
        let gs_key = format!("{addres_type_c}{address}-gs");
        if prev_status == AircraftStatus::Unknown { // we have no prior information
            let status = format!("0;{ts}"); // 0 = AircraftStatus::OnGround
            self.save_to_redis(&status_key, &status, REDIS_RECORD_EXPIRATION);
            self.save_to_redis(&gs_key, &0.to_string(), 120); // gs = 0
        }

        let prev_gs = match self.get_from_redis(&gs_key) {
            Ok(gs) => gs.parse().unwrap_or(0_f64),
            Err(_) => 0_f64
        };
        if prev_gs > 0_f64 { // filter speed change a bit (sometimes there are glitches in speed with badly placed gps antenna):
            gs = gs * 0.7 + prev_gs * 0.3;
        }
        self.save_to_redis(&gs_key, &format!("{:.0}", gs.round()), 3600);

        println!("GS {gs:.0} km/h");
        // TODO.. radek 240+

    }

}

    