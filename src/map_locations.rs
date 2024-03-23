use crate::models::LatLng;
use rand::Rng;

static LOCATIONS: [(f64, f64); 840] = include!("street_view_locations.txt");

pub fn get_random_position() -> LatLng {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..LOCATIONS.len());
    LatLng {
        lat: LOCATIONS[index].0,
        lng: LOCATIONS[index].1,
    }
}

pub fn get_guess_score(guess: LatLng, target: LatLng) -> u64 {
    let lat_diff = (guess.lat - target.lat).abs();
    let lng_diff = (guess.lng - target.lng).abs();
    let distance = (lat_diff.powi(2) + lng_diff.powi(2)).sqrt();
    let score = (1.0 / distance) * 1000.0;
    (score as u64).min(2000)
}
