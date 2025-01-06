use crate::map::models::LatLng;
use rand::Rng;
use std::sync::OnceLock;

pub static LOCATIONS: OnceLock<Vec<LatLng>> = OnceLock::new();

pub fn random() -> LatLng {
    let mut rng = rand::thread_rng();
    let locations = LOCATIONS.get().expect("`LOCATIONS` was not initialized.");
    let index = rng.gen_range(0..locations.len());
    locations[index]
}
