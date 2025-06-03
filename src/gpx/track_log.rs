use super::track_point::GPXTrackPoint;

#[derive(Debug, Clone)]
pub(crate) struct GPXTrackLog {
    points: Vec<GPXTrackPoint>,
}

impl GPXTrackLog {
    pub(crate) fn new() -> Self {
        GPXTrackLog { points: Vec::new() }
    }

    pub(crate) fn extend(&mut self, other: GPXTrackLog) {
        self.points.extend(other.points);
    }

    pub(crate) fn push(&mut self, point: GPXTrackPoint) {
        self.points.push(point);
    }

    pub(crate) fn first(&self) -> Option<&GPXTrackPoint> {
        self.points.first()
    }

    pub(crate) fn last(&self) -> Option<&GPXTrackPoint> {
        self.points.last()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.points.is_empty()
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
