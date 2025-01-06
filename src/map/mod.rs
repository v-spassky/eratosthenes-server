use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use crate::cli::Args;
use consts::{EARTH_RADIUS, MAX_SCORE};
use locations::LOCATIONS;
use models::LatLng;

pub mod consts;
pub mod locations;
pub mod models;

pub fn init(args: &Args) {
    let locations_file = File::open(&args.locations).expect("Failed to open the locations file.");
    let file_reader = BufReader::new(locations_file);
    let mut locations = Vec::new();
    for line in file_reader.lines() {
        let maybe_line = line.expect("Failed to read a line in the locations file.");
        let location: LatLng = serde_json::from_str(&maybe_line)
            .expect("Failed to deserialize a line in the locations file into a `LatLng`.");
        locations.push(location);
    }
    LOCATIONS
        .set(locations)
        .expect("Somehow `LOCATIONS` was set before `init`.");
}

pub fn estimate_guess(guess: LatLng, target: LatLng) -> u64 {
    let phi_1 = guess.lat * std::f64::consts::PI / 180.0;
    let phi_2 = target.lat * std::f64::consts::PI / 180.0;
    let delta_phi = (target.lat - guess.lat) * std::f64::consts::PI / 180.0;
    let delta_lambda = (target.lng - guess.lng) * std::f64::consts::PI / 180.0;
    let a = (delta_phi / 2.0).sin().powi(2)
        + phi_1.cos() * phi_2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * (a.sqrt().atan2((1.0 - a).sqrt()));
    let distance = EARTH_RADIUS * c;
    (MAX_SCORE * (1.65_f64).powf(-distance * 1e-6)) as u64
}
