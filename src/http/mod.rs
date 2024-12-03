pub mod config;
pub mod query_params;

use crate::cli::Args;
use config::{CORS_POLICY, RESPONSE_HEADERS};
use warp::http::header::{HeaderMap, HeaderValue};
use warp::hyper::Method;

pub fn init(_args: &Args) {
    CORS_POLICY.get_or_init(|| {
        warp::cors()
            .allow_origin("http://127.0.0.1:3000")
            .allow_origin("http://localhost:3000")
            .allow_origin("https://eratosthenes.vercel.app/")
            .allow_headers(vec![
                "User-Agent",
                "Sec-Fetch-Mode",
                "Referer",
                "Origin",
                "Access-Control-Request-Method",
                "Access-Control-Request-Headers",
                "content-type",
                "Passcode",
            ])
            .allow_methods(&[Method::POST, Method::GET, Method::OPTIONS])
            .build()
    });
    RESPONSE_HEADERS.get_or_init(|| {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers
    });
}
