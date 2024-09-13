use once_cell::sync::Lazy;
use warp::filters::cors::Cors;
use warp::http::header::{HeaderMap, HeaderValue};
use warp::hyper::Method;

pub static CORS_POLICY: Lazy<Cors> = Lazy::new(|| {
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

pub static RESPONSE_HEADERS: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers
});
