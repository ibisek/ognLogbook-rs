use ogn_client::data_structures::AircraftType;

static SLOW_CRAFTS: [AircraftType; 8] = [AircraftType::Glider, AircraftType::Helicopter, AircraftType::Parachute, AircraftType::HangGlider, AircraftType::Paraglider, AircraftType::Baloon, AircraftType::Airship, AircraftType::Uav];

/**
 * :param AircraftType
 * :param forEvent: 'L' / 'T' threshold for expected event
 * :return: threshold speed for given event in km/h
 */
pub fn get_groundspeed_threshold(aircraft_type: &AircraftType, for_event: char) -> f64 {
    if for_event == 'L' {
        if SLOW_CRAFTS.contains(aircraft_type) {
            return 20_f64;  // [km/h] glider]
        } else {
            return 50_f64;  //[km/h] tow
        }
    } else {    // takeoff threshold
        return 80_f64;   // [km/h] all
    }
}
