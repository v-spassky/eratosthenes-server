use crate::models::LatLng;

pub fn get_random_position() -> LatLng {
    LatLng {
        lat: 43.7479964,
        lng: 27.406036,
    }
}

pub fn get_guess_score(guess: LatLng, target: LatLng) -> u64 {
    let lat_diff = (guess.lat - target.lat).abs();
    let lng_diff = (guess.lng - target.lng).abs();
    let distance = (lat_diff.powi(2) + lng_diff.powi(2)).sqrt();
    let score = (1.0 / distance) * 1000.0;
    (score as u64).min(2000)
}
