pub(crate) mod track_log;
pub(crate) mod track_point;

use track_log::GPXTrackLog;

#[derive(Debug)]
pub(crate) struct GPX {
    child: GPXTrackLog,
}

impl GPX {
    pub(crate) fn new(child: GPXTrackLog) -> Self {
        GPX { child }
    }

    pub(crate) fn to_str(&self) -> String {
        format!(
        "<gpx xmlns=\"http://www.topografix.com/GPX/1/1\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd\" version=\"1.1\" creator=\"zdr055_gpx\">\n{}\n</gpx>", self.child.to_str()
        )
    }

    pub(crate) fn save(&self, path: &std::path::Path) -> Result<(), String> {
        let gpx_string = self.to_str();
        std::fs::write(path, gpx_string).map_err(|e| format!("Failed to write GPX file: {}", e))
    }
}
