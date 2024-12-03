use crate::cli::Args;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::OnceLock;

pub mod endpoints;
pub mod handlers;
pub mod passcode;
pub mod responses;

static JWT_SIGNING_KEY: OnceLock<Hmac<Sha256>> = OnceLock::new();

pub fn init(args: &Args) {
    JWT_SIGNING_KEY.get_or_init(|| {
        Hmac::new_from_slice(args.jwt_signing_key.as_bytes()).expect("Failed to create HMAC code.")
    });
}
