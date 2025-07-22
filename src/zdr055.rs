use regex::Regex;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path,
    str::FromStr,
};

use crate::{
    container::avi::{AVI, FILESIZE_FIELD_SIZE, IDX1_FOURCC, IDX1_INDEX_ENTRY_SIZE},
    gpx::track_point::GPXTrackPoint,
};

const GPS_DATA_CHUNK_ID: &[u8; 4] = b"02tx";

pub(crate) struct ZDR055MediaData {
    filename: path::PathBuf,
}

impl AVI for ZDR055MediaData {}

impl ZDR055MediaData {
    pub(crate) fn new(filename: &str) -> Self {
        let filename = path::PathBuf::from(filename);
        ZDR055MediaData { filename }
    }

    fn check_filename(&self) -> Result<(), String> {
        let ext = self.filename.extension().and_then(|s| s.to_str());
        if ext.is_none() {
            return Err("Invalid file extension".to_string());
        }
        let ext = ext.unwrap().to_ascii_lowercase();
        if ext != "avi" {
            return Err("Invalid file extension".to_string());
        }
        Ok(())
    }

    pub(crate) fn extract_stream_data(&self) -> Result<Vec<String>, String> {
        if !self.filename.exists() {
            return Err("File does not exist".to_string());
        }
        if !self.filename.is_file() {
            return Err("Path is not a file".to_string());
        }
        if let Err(e) = self.check_filename() {
            return Err(e);
        }

        let mut file =
            File::open(&self.filename).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer);
        let mut file_reader = std::io::Cursor::new(&buffer);

        // 適当にidx1チャンクを探す
        let index = memchr::memmem::rfind(&buffer, IDX1_FOURCC);
        if index.is_none() {
            return Err("Stream data not found".to_string());
        }

        // idx1チャンクとして読んでみる
        let index = index.unwrap();
        let idx1_index = self.parse_chunk_header(&buffer[index..]);
        if idx1_index.is_none() {
            return Err("Index data not found".to_string());
        }
        let idx1_index = idx1_index.unwrap();
        let idx1 = &buffer[index..index + FILESIZE_FIELD_SIZE + idx1_index.get_size()];

        let mut idx1_reader = std::io::Cursor::new(idx1);
        idx1_reader
            .seek(SeekFrom::Start(8))
            .map_err(|e| format!("Failed to seed IDX1 entry: {}", e))?; // idx1ヘッダのサイズ分をスキップ

        let mut stream_data = Vec::new();
        let mut index_data = Vec::new();
        while let Some(index_entry) = self.read_index_entry(&mut idx1_reader) {
            if &index_entry.get_chunk_id() == GPS_DATA_CHUNK_ID {
                index_data.push(index_entry);
            }
            let _ = idx1_reader.seek(SeekFrom::Current(IDX1_INDEX_ENTRY_SIZE as i64));
        }

        for index in index_data.iter() {
            let mut data = vec![0u8; index.get_size()];
            file_reader
                .seek(SeekFrom::Start(
                    index.get_offset() + IDX1_FOURCC.len() as u64 + FILESIZE_FIELD_SIZE as u64,
                ))
                .map_err(|e| format!("Failed to seek in file: {}", e))?;
            file_reader
                .read_exact(&mut data)
                .map_err(|e| format!("Failed to read file: {}", e))?;

            let str = String::from_utf8(data.clone())
                .map_err(|e| format!("Failed to convert data to string: {}. Data: {:?}", e, data));
            stream_data.push(str.unwrap());
        }

        Ok(stream_data)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ZDR055PositionData {
    device: String,
    timestamp: String,
    x_accel: f64,
    y_accel: f64,
    z_accel: f64,
    unknown_field_t: String,
    supply_voltage: f64,
    event_type: String,
    latitude: f64,
    longitude: f64,
    speed: f64,
    e_value: u8,
    m_value: u8,
    em_value: u8,
    sa_value: u8,
    firmware_version: u8,
    s_value: u32,
    unknown_field_tail: String,
    is_valid: bool,
}

impl FromStr for ZDR055PositionData {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(
            r"(?x)
            (?x)
            (?P<device>[A-Za-z0-9]+):                              # Device name
            (?P<timestamp>\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})\s  # Timestamp
            X:\s?(?P<x_accel>-?\d+\.\d{2})\s                       # X acceleration
            Y:\s?(?P<y_accel>-?\d+\.\d{2})\s                       # Y acceleration
            Z:\s?(?P<z_accel>-?\d+\.\d{2})\s                       # Z acceleration
            (?P<unknown_field_t>\S+T)\s                            # Temperature or unknown field
            (?P<supply_voltage>\d+\.\d)V\s                         # Supply voltage
            (?P<event_type>[-\w]+)\s+                              # Event type
            (?P<latitude>\d+\.\d+|[-\.]+)\s                        # Latitude
            (?P<latitude_ns>[NS-])\s+                              # Latitude N or S
            (?P<longitude>\d+\.\d+|[-\.]+)\s                       # Longitude
            (?P<longitude_we>[WE-])\s                              # Longitude W or E
            (?P<speed>\S+)km/h\s?                                  # Speed
            E:(?P<e_value>\d+)\s                                   # E value, 255
            M:(?P<m_value>\d+)\s                                   # M value, 255
            EM:(?P<em_value>\d+)\s                                 # EM value, 255
            SA:(?P<sa_value>\d+)\s                                 # SA value, 0
            V:(?P<firmware_version>\d+)\s                          # Firmware version
            S:(?P<s_value>\d+k)\s                                  # S value
            (?P<unknown_field_tail>[\d,]+)                         # Unknown field tail 
            
            ",
        )
        .unwrap();

        let caps = re.captures(input).ok_or("Failed to capture log data")?;

        if caps["event_type"].to_string() == "-" {
            return Ok(ZDR055PositionData {
                device: caps["device"].to_string(),
                timestamp: caps["timestamp"].to_string(),
                ..Default::default()
            });
        }

        let supply_voltage: f64 = if caps["supply_voltage"].parse::<f64>().is_err() {
            return Err("Failed to parse supply_voltage".to_string());
        } else {
            caps["supply_voltage"].parse().unwrap()
        };
        if supply_voltage == 0.0 {
            return Ok(ZDR055PositionData {
                device: caps["device"].to_string(),
                timestamp: caps["timestamp"].to_string(),
                ..Default::default()
            });
        }

        let x_accel: f64 = if caps["x_accel"].parse::<f64>().is_err() {
            return Err("Failed to parse x_accel".to_string());
        } else {
            caps["x_accel"].parse().unwrap()
        };
        let y_accel: f64 = if caps["y_accel"].parse::<f64>().is_err() {
            return Err("Failed to parse y_accel".to_string());
        } else {
            caps["y_accel"].parse().unwrap()
        };
        let z_accel: f64 = if caps["z_accel"].parse::<f64>().is_err() {
            return Err("Failed to parse z_accel".to_string());
        } else {
            caps["z_accel"].parse().unwrap()
        };
        let latitude: f64 = if caps["latitude"].parse::<f64>().is_err() {
            return Err(format!(
                "Failed to parse latitude: {}",
                caps["latitude"].to_string()
            ));
        } else {
            if &caps["latitude_ns"] == "S" {
                -1.0 * caps["latitude"].parse::<f64>().unwrap()
            } else {
                caps["latitude"].parse().unwrap()
            }
        };
        let longitude: f64 = if caps["longitude"].parse::<f64>().is_err() {
            return Err("Failed to parse longitude".to_string());
        } else {
            if &caps["longitude_we"] == "W" {
                -1.0 * caps["longitude"].parse::<f64>().unwrap()
            } else {
                caps["longitude"].parse().unwrap()
            }
        };
        let speed: f64 = if caps["speed"].parse::<f64>().is_err() {
            return Err("Failed to parse speed".to_string());
        } else {
            caps["speed"].parse().unwrap()
        };

        Ok(ZDR055PositionData {
            device: caps["device"].to_string(),
            timestamp: caps["timestamp"].to_string(),
            x_accel,
            y_accel,
            z_accel,
            unknown_field_t: caps["unknown_field_t"].to_string(),
            supply_voltage,
            event_type: caps["event_type"].to_string(),
            latitude,
            longitude,
            speed,
            e_value: caps["e_value"].parse().unwrap(),
            m_value: caps["m_value"].parse().unwrap(),
            em_value: caps["em_value"].parse().unwrap(),
            sa_value: caps["sa_value"].parse().unwrap(),
            firmware_version: caps["firmware_version"].parse().unwrap(),
            s_value: caps["s_value"].trim_end_matches('k').parse().unwrap(),
            unknown_field_tail: caps["unknown_field_tail"].to_string(),
            is_valid: true,
        })
    }
}

impl ZDR055PositionData {
    pub(crate) fn to_gpx_point(&self) -> GPXTrackPoint {
        GPXTrackPoint::new(
            self.latitude,
            self.longitude,
            0.0,
            self.speed,
            self.timestamp.clone(),
        )
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.is_valid
    }


    // fn to_gpx_string(&self) -> String {
    //     let point = self.to_gpx_point();
    //     format!(
    //         "<trkpt lat=\"{}\" lon=\"{}\"><time>{}</time><speed>{}</speed></trkpt>\n",
    //         point.lat, point.lon, point.time, point.speed
    //     )
    // }
}
