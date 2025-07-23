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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // --- 1. デバイス名 ---
        let (device, mut s) = s.split_once(':').ok_or("Invalid format for device name")?;

        // --- 2. タイムスタンプ ---
        s = s.trim_start();
        let (timestamp, mut s) = s.split_at(19); // "YYYY-MM-DD HH:MM:SS" は19バイト
        if timestamp.len() != 19 {
            return Err("Invalid timestamp format".to_string());
        }

        // --- 3. 加速度 (X, Y, Z) ---
        s = s
            .trim_start()
            .strip_prefix("X:")
            .ok_or("Missing X acceleration")?
            .trim_start();
        let (x_accel_str, mut s) = s
            .split_once(' ')
            .ok_or("Invalid format for X acceleration")?;
        let x_accel = x_accel_str
            .parse::<f64>()
            .map_err(|_| format!("Failed to parse X acceleration: {}", x_accel_str))?;

        s = s
            .trim_start()
            .strip_prefix("Y:")
            .ok_or("Missing Y acceleration")?
            .trim_start();
        let (y_accel_str, mut s) = s
            .split_once(' ')
            .ok_or("Invalid format for Y acceleration")?;
        let y_accel = y_accel_str
            .parse::<f64>()
            .map_err(|_| format!("Failed to parse Y acceleration: {}", y_accel_str))?;

        s = s
            .trim_start()
            .strip_prefix("Z:")
            .ok_or("Missing Z acceleration")?
            .trim_start();
        let (z_accel_str, mut s) = s
            .split_once(' ')
            .ok_or("Invalid format for Z acceleration")?;
        let z_accel = z_accel_str
            .parse::<f64>()
            .map_err(|_| format!("Failed to parse Z acceleration: {}", z_accel_str))?;

        // --- 4. 不明なTフィールド & 電圧 ---
        s = s.trim_start();
        let (unknown_field_t, mut s) = s
            .split_once(' ')
            .ok_or("Invalid format for unknown field T")?;
        s = s.trim_start();
        let (supply_voltage_str, mut s) = s
            .split_once(' ')
            .ok_or("Invalid format for supply voltage")?;
        let supply_voltage = supply_voltage_str
            .strip_suffix('V')
            .ok_or("Invalid format for supply voltage")?
            .parse::<f64>()
            .map_err(|_| format!("Failed to parse supply voltage: {}", supply_voltage_str))?;

        // --- 5. イベントタイプ & 緯度経度 ---
        s = s.trim_start();
        let (event_type, mut s) = s.split_once(' ').ok_or("Invalid format for event type")?;
        s = s.trim_start();
        let (latitude_str, mut s) = s.split_once(' ').ok_or("Invalid format for latitude")?;
        let latitude = latitude_str
            .parse::<f64>()
            .map_err(|_| format!("Failed to parse latitude: {}", latitude_str))?;
        s = s.trim_start();
        let (_ns, mut s) = s.split_once(' ').ok_or("Invalid format for N/S")?; // N/S をスキップ
        s = s.trim_start();
        let (longitude_str, mut s) = s.split_once(' ').ok_or("Invalid format for longitude")?;
        let longitude = longitude_str
            .parse::<f64>()
            .map_err(|_| format!("Failed to parse longitude: {}", longitude_str))?;
        s = s.trim_start();
        let (_we, mut s) = s.split_once(' ').ok_or("Invalid format for W/E")?; // W/E をスキップ

        // --- 6. 速度 ---
        s = s.trim_start();
        let (speed_str, mut s) = s.split_once("km/h").ok_or("Invalid format for speed")?;
        let speed = speed_str
            .parse::<f64>()
            .map_err(|_| format!("Failed to parse speed: {}", speed_str))?;

        // --- 7. 残りのフィールド (空白で分割) ---
        let mut parts = s.split_whitespace();
        let e_value = parts
            .next()
            .ok_or("Missing E field")?
            .strip_prefix("E:")
            .ok_or("Invalid format for E field")?
            .parse::<u8>()
            .map_err(|_| "Failed to parse E value")?;
        let m_value = parts
            .next()
            .ok_or("Missing M field")?
            .strip_prefix("M:")
            .ok_or("Invalid format for M field")?
            .parse::<u8>()
            .map_err(|_| "Failed to parse M value")?;
        let em_value = parts
            .next()
            .ok_or("Missing EM field")?
            .strip_prefix("EM:")
            .ok_or("Invalid format for EM field")?
            .parse::<u8>()
            .map_err(|_| "Failed to parse EM value")?;
        let sa_value = parts
            .next()
            .ok_or("Missing SA field")?
            .strip_prefix("SA:")
            .ok_or("Invalid format for SA field")?
            .parse::<u8>()
            .map_err(|_| "Failed to parse SA value")?;
        let firmware_version = parts
            .next()
            .ok_or("Missing firmware version field")?
            .strip_prefix("V:")
            .ok_or("Invalid format for firmware version field")?
            .parse::<u8>()
            .map_err(|_| "Failed to parse firmware version")?;
        let s_value_str = parts
            .next()
            .ok_or("Missing S field")?
            .strip_prefix("S:")
            .ok_or("Invalid format for S field")?
            .strip_suffix('k')
            .ok_or("Invalid format for S field")?;
        let s_value = s_value_str
            .parse::<u32>()
            .map_err(|_| format!("Failed to parse S value: {}", s_value_str))?;
        let unknown_field_tail = parts.next().ok_or("Missing unknown field tail")?;

        // --- 8. 構造体を構築して返す ---
        Ok(ZDR055PositionData {
            device: device.to_string(),
            timestamp: timestamp.to_string(),
            x_accel,
            y_accel,
            z_accel,
            unknown_field_t: unknown_field_t.to_string(),
            supply_voltage,
            event_type: event_type.to_string(),
            latitude,
            longitude,
            speed,
            e_value,
            m_value,
            em_value,
            sa_value,
            firmware_version,
            s_value,
            unknown_field_tail: unknown_field_tail.to_string(),
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

    pub(crate) fn has_same_timestamp(&self, other: &ZDR055PositionData) -> bool {
        self.timestamp == other.timestamp
    }

    pub(crate) fn has_same_position(&self, other: &ZDR055PositionData) -> bool {
        self.latitude == other.latitude && self.longitude == other.longitude
    }

    // fn to_gpx_string(&self) -> String {
    //     let point = self.to_gpx_point();
    //     format!(
    //         "<trkpt lat=\"{}\" lon=\"{}\"><time>{}</time><speed>{}</speed></trkpt>\n",
    //         point.lat, point.lon, point.time, point.speed
    //     )
    // }
}
