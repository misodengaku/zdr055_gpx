use chrono::{DateTime, NaiveDateTime, TimeZone};
use chrono_tz::{Asia::Tokyo, Tz};

#[derive(Debug, Clone)]
pub(crate) struct GPXTrackPoint {
    lat: f64,
    lon: f64,
    ele: f64,
    speed: f64,
    time: String,
}

impl GPXTrackPoint {
    pub(crate) fn new(lat: f64, lon: f64, ele: f64, speed: f64, time: String) -> Self {
        GPXTrackPoint {
            lat,
            lon,
            ele,
            speed,
            time,
        }
    }

    pub(crate) fn to_gpx_string(&self) -> String {
        let timestamp = self.timestamp().unwrap_or_else(|_| {
            // If timestamp parsing fails, use a default value
            Tokyo
                .from_local_datetime(&NaiveDateTime::from_timestamp(0, 0))
                .unwrap()
        });

        format!(
            "<trkpt lat=\"{:.7}\" lon=\"{:.7}\"><ele>{:.2}</ele><time>{}</time><desc>{:.2} km/h</desc></trkpt>",
            self.lat, self.lon, self.ele, timestamp.format("%Y-%m-%dT%H:%M:%S%:z"), self.speed
        )
    }

    pub(crate) fn timestamp(&self) -> Result<DateTime<Tz>, String> {
        if self.time.is_empty() {
            return Err("Time is empty".to_string());
        }
        let timestamp = NaiveDateTime::parse_from_str(&self.time, "%Y-%m-%d %H:%M:%S")
            .map(|naive_date| Tokyo.from_local_datetime(&naive_date).unwrap());

        if timestamp.is_err() {
            Err("Failed to parse timestamp".to_string())
        } else {
            Ok(timestamp.unwrap())
        }
    }
}

fn to_gpx_string(child: &str) -> String {
    format!(
        "<gpx xmlns=\"http://www.topografix.com/GPX/1/1\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd\" version=\"1.1\" creator=\"zdr055_gpx\">\n{}\n</gpx>", child
    )
}
