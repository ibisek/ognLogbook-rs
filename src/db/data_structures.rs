use ogn_client::data_structures::{AddressType, AircraftType};

#[derive(Debug)]
struct LogbookEntry {
    pub id: u64, 
    pub addr: String, 
    pub addr_type: AddressType, 
    pub takeoff_ts: i64, 
    pub landing_ts: i64,
}

#[derive(Debug, Clone)]
pub struct LogbookEvent {
    pub id: u64, 
    pub ts: i64,
    pub event: String,     
    pub address: String, 
    pub address_type: AddressType, 
    pub aircraft_type: AircraftType,
    pub lat: f64,
    pub lon: f64,
    pub location_icao: String,
}

#[derive(Debug, Clone)]
pub struct LogbookItem {
    pub id: u64, 
    pub addr: String, 
    pub addr_type: AddressType, 
    
    pub takeoff_ts: i64, 
    pub takeoff_lat: f64, 
    pub takeoff_lon: f64, 
    pub takeoff_icao: String,
                 
    pub landing_ts: i64,
    pub landing_lat: f64, 
    pub landing_lon: f64, 
    pub landing_icao: String,
                 
    pub flight_time: i64, 
    pub flown_distance: u64, 
    pub device_type: String,
                 
    pub registration: String, 
    pub cn: String, 
    pub aircraft_type: AircraftType, 
    pub tow_id: i64,
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
            takeoff_icao: takeoff_icao, 

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
