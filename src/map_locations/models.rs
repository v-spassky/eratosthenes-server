#[derive(Copy, Clone, Debug)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

impl LatLng {
    pub fn as_json(&self) -> String {
        format!("{{\"lat\": {}, \"lng\": {}}}", self.lat, self.lng)
    }
}
