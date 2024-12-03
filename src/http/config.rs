use std::sync::OnceLock;
use warp::filters::cors::Cors;
use warp::http::header::HeaderMap;

pub static CORS_POLICY: OnceLock<Cors> = OnceLock::new();
pub static RESPONSE_HEADERS: OnceLock<HeaderMap> = OnceLock::new();
