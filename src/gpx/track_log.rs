use super::track_point::GPXTrackPoint;

#[derive(Debug)]
pub(crate) struct GPXTrackLog {
    points: Vec<GPXTrackPoint>,
}

impl GPXTrackLog {
    pub(crate) fn new() -> Self {
        GPXTrackLog { points: Vec::new() }
    }

    fn join(&mut self, other: GPXTrackLog) {
        self.points.extend(other.points);
    }

    pub(crate) fn push(&mut self, point: GPXTrackPoint) {
        self.points.push(point);
    }

    pub(crate) fn to_str(&self) -> String {
        if self.points.is_empty() {
            return String::new();
        }

        let track_points = self
            .points
            .iter()
            .map(|log| log.to_gpx_string())
            .collect::<Vec<String>>();

        format!("<trk><trkseg>{}</trkseg></trk>\n", track_points.join(""))
    }
}
