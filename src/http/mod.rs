pub mod middleware;
pub mod query_params;
pub mod router;

use crate::cli::Args;
use http::Method;
use tower_http::cors::CorsLayer;

pub fn init(_args: &Args) -> CorsLayer {
    CorsLayer::new()
        .allow_origin([
            // TODO: this should be configured from outside the program (config file, CLI args)
            "http://127.0.0.1:3000".parse().unwrap(),
            "http://localhost:3000".parse().unwrap(),
            "https://eratosthenes.vercel.app".parse().unwrap(),
        ])
        .allow_headers([
            "User-Agent".parse().unwrap(),
            "Sec-Fetch-Mode".parse().unwrap(),
            "Referer".parse().unwrap(),
            "Origin".parse().unwrap(),
            "Access-Control-Request-Method".parse().unwrap(),
            "Access-Control-Request-Headers".parse().unwrap(),
            "content-type".parse().unwrap(),
            "Passcode".parse().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
}
