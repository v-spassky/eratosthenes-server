use crate::cli::Args;
use hmac::{Hmac, Mac};
use once_cell::sync::OnceCell;
use sha2::Sha256;

pub mod endpoints;
pub mod handlers;
pub mod passcode;
pub mod responses;

// TODO: use standard library here once migrated to newer Rust version
static JWT_SIGNING_KEY: OnceCell<Hmac<Sha256>> = OnceCell::new();

pub fn init(args: &Args) {
    JWT_SIGNING_KEY.get_or_init(|| {
        Hmac::new_from_slice(args.jwt_signing_key.as_bytes())
            .expect("Failed to create JWT signing key!")
    });
}
