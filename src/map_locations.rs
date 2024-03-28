use crate::models::LatLng;
use rand::Rng;

static LOCATIONS: [(f64, f64); 840] = include!("street_view_locations.txt");
const EARTH_RADIUS: f64 = 6371e3;
const MAX_SCORE: u64 = 2000;

pub fn get_random_position() -> LatLng {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..LOCATIONS.len());
    LatLng {
        lat: LOCATIONS[index].0,
        lng: LOCATIONS[index].1,
    }
}

pub fn get_guess_score(guess: LatLng, target: LatLng) -> u64 {
    let phi_1 = guess.lat * std::f64::consts::PI / 180.0;
    let phi_2 = target.lat * std::f64::consts::PI / 180.0;
    let delta_phi = (target.lat - guess.lat) * std::f64::consts::PI / 180.0;
    let delta_lambda = (target.lng - guess.lng) * std::f64::consts::PI / 180.0;
    let a = (delta_phi / 2.0).sin().powi(2)
        + phi_1.cos() * phi_2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * (a.sqrt().atan2((1.0 - a).sqrt()));
    let distance = EARTH_RADIUS * c;
    let score = (1.0 / distance) * 1e8;
    (score as u64).min(MAX_SCORE)
}
