use std::fmt;

#[repr(i8)]
#[derive(Debug, Clone, PartialEq)]
pub enum AircraftStatus {
    Unknown = -1,
    OnGround = 0,
    Airborne = 1,
}

impl AircraftStatus {
    pub fn from_i8(value: i8) -> AircraftStatus {
        match value {
            -1 => AircraftStatus::Unknown,
            0 => AircraftStatus::OnGround,
            1 => AircraftStatus::Airborne,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

// impl fmt::Display for AircraftStatus {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.to_string())
//     }
// }
