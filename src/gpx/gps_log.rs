use regex::Regex;
use std::str::FromStr;

// デバイスにかかわらず、最低限これだけは GPS ログとして保持しているであろう情報の構造体
#[derive(Debug)]
pub(crate) struct GPSLog {
    timestamp: String,
    latitude: f64,
    longitude: f64,
    speed: f64,
}

impl GPSLog {
    fn to_gpx_point(&self) -> GPXTrackPoint {
        GPXTrackPoint::new(
            self.latitude,
            self.longitude,
            0.0,
            self.speed,
            self.timestamp.clone(),
        )
    }
}

// impl FromStr for GPSLog {
//     type Err = String;

//     fn from_str(input: &str) -> Result<Self, Self::Err> {
//         let re = Regex::new(
//             r"(?x)
//             (?x)
//             (?P<device>[A-Za-z0-9]+):                              # Device name
//             (?P<timestamp>\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})\s  # Timestamp
//             X:\s?(?P<x_accel>-?\d+\.\d{2})\s                       # X acceleration
//             Y:\s?(?P<y_accel>-?\d+\.\d{2})\s                       # Y acceleration
//             Z:\s?(?P<z_accel>-?\d+\.\d{2})\s                       # Z acceleration
//             (?P<unknown_field_t>\S+T)\s                            # Unknown field T
//             (?P<supply_voltage>\d+\.\d)V\s                         # Supply voltage
//             (?P<event_type>[-\w]+)\s+                              # Event type
//             (?P<latitude>\d+\.\d+|[-\.]+)\s                        # Latitude
//             (?P<latitude_ns>[NS-])\s+                              # Latitude N or S
//             (?P<longitude>\d+\.\d+|[-\.]+)\s                       # Longitude
//             (?P<longitude_we>[WE-])\s                              # Longitude W or E
//             (?P<speed>\S+)km/h\s?                                  # Speed
//             E:(?P<e_value>\d+)\s                                   # E value
//             M:(?P<m_value>\d+)\s                                   # M value
//             EM:(?P<em_value>\d+)\s                                 # EM value
//             SA:(?P<sa_value>\d+)\s                                 # EA value
//             V:(?P<firmware_version>\d+)\s                          # Firmware version
//             S:(?P<s_value>\d+k)\s                                  # S value
//             (?P<unknown_field_tail>[\d,]+)                         # Unknown field tail

//             ",
//         )
//         .unwrap();

//         let caps = re.captures(input).ok_or("Failed to capture log data")?;

//         let x_accel: f64 = if caps["x_accel"].parse::<f64>().is_err() {
//             return Err("Failed to parse x_accel".to_string());
//         } else {
//             caps["x_accel"].parse().unwrap()
//         };
//         let y_accel: f64 = if caps["y_accel"].parse::<f64>().is_err() {
//             return Err("Failed to parse y_accel".to_string());
//         } else {
//             caps["y_accel"].parse().unwrap()
//         };
//         let z_accel: f64 = if caps["z_accel"].parse::<f64>().is_err() {
//             return Err("Failed to parse z_accel".to_string());
//         } else {
//             caps["z_accel"].parse().unwrap()
//         };
//         let supply_voltage: f64 = if caps["supply_voltage"].parse::<f64>().is_err() {
//             return Err("Failed to parse supply_voltage".to_string());
//         } else {
//             caps["supply_voltage"].parse().unwrap()
//         };
//         let latitude: f64 = if caps["latitude"].parse::<f64>().is_err() {
//             return Err("Failed to parse latitude".to_string());
//         } else {
//             if &caps["latitude_ns"] == "S" {
//                 -1.0 * caps["latitude"].parse::<f64>().unwrap()
//             } else {
//                 caps["latitude"].parse().unwrap()
//             }
//         };
//         let longitude: f64 = if caps["longitude"].parse::<f64>().is_err() {
//             return Err("Failed to parse longitude".to_string());
//         } else {
//             if &caps["longitude_we"] == "W" {
//                 -1.0 * caps["longitude"].parse::<f64>().unwrap()
//             } else {
//                 caps["longitude"].parse().unwrap()
//             }
//         };
//         let speed: f64 = if caps["speed"].parse::<f64>().is_err() {
//             return Err("Failed to parse speed".to_string());
//         } else {
//             caps["speed"].parse().unwrap()
//         };

//         Ok(GPSLog {
//             device: caps["device"].to_string(),
//             timestamp: caps["timestamp"].to_string(),
//             x_accel,
//             y_accel,
//             z_accel,
//             unknown_field_t: caps["unknown_field_t"].to_string(),
//             supply_voltage,
//             event_type: caps["event_type"].to_string(),
//             latitude,
//             longitude,
//             speed,
//             e_value: caps["e_value"].parse().unwrap(),
//             m_value: caps["m_value"].parse().unwrap(),
//             em_value: caps["em_value"].parse().unwrap(),
//             sa_value: caps["sa_value"].parse().unwrap(),
//             firmware_version: caps["firmware_version"].parse().unwrap(),
//             s_value: caps["s_value"].trim_end_matches('k').parse().unwrap(),
//             unknown_field_tail: caps["unknown_field_tail"].to_string(),
//         })
//     }
// }
