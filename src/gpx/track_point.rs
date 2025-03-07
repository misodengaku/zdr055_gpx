#[derive(Debug)]
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
        format!(
            "<trkpt lat=\"{:.6}\" lon=\"{:.6}\"><ele>{:.2}</ele><time>{}</time><speed>{:.2}</speed></trkpt>",
            self.lat, self.lon, self.ele, self.time, self.speed
        )
    }
}

fn to_gpx_string(child: &str) -> String {
    format!(
        "<gpx xmlns=\"http://www.topografix.com/GPX/1/1\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd\" version=\"1.1\" creator=\"zdr055_gpx\">\n{}\n</gpx>", child
    )
}
