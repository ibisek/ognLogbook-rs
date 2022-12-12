// use std::fmt;

#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy)]
pub struct AircraftStatusWithTs {
    pub ts: u64,
    pub status: AircraftStatus,
}

impl AircraftStatusWithTs {
    pub fn new(status: AircraftStatus, ts:u64) -> AircraftStatusWithTs {
        Self {
            ts,
            status,
        }
    }

    // format: "0;ts"
    pub fn as_redis_str(&self) -> String {
        format!("{};{}", self.status as i8, self.ts)
    }

    // format: "0;ts"
    pub fn from_redis_str(ps: &str) -> AircraftStatusWithTs {
        let items = ps.split(";").collect::<Vec<&str>>();   // parse "ps;ts"
        AircraftStatusWithTs { 
            ts: items[1].parse().unwrap(),
            status: AircraftStatus::from_i8(items[0].parse().unwrap_or(-1)), // -1 = Unknown
        }
    }

    //TODO tady by to chtelo "impl Eq" na porovnavani AircraftStatusWithTs.status ==? AircraftStatus
    pub fn is(&self, other_status: AircraftStatus) -> bool {
        self.status == other_status
    }
    
}

