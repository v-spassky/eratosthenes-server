use crate::health::handlers::HealthHttpHandler;
use crate::http::config::{CORS_POLICY, RESPONSE_HEADERS};
use crate::logging::consts::DEFAULT_CLIENT_IP;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::Instant;
use warp::hyper::StatusCode;
use warp::reply::Reply;
use warp::{Filter, Rejection};

// TODO: consider rate-limiting of some kind since this endpoint doesn't require a passcode.
pub fn healthcheck() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("health")
        .and(warp::path("check"))
        .and(warp::addr::remote())
        .and_then(move |client_ip: Option<SocketAddr>| async move {
            let start_time = Instant::now();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let response = HealthHttpHandler::new().healthcheck().await;
            let processing_time_ns = start_time.elapsed().as_nanos();
            tracing::info!(
                task = "http_request",
                http_method = "GET",
                endpoint = "/health/check",
                private_id = "no_auth",
                client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                processing_time_ms = processing_time_ns / 1000,
                timestamp,
            );
            Ok::<_, Infallible>(warp::reply::with_status(
                serde_json::to_string(&response).unwrap(),
                StatusCode::OK,
            ))
        })
        .with(warp::reply::with::headers(RESPONSE_HEADERS.clone()))
        .with(CORS_POLICY.clone())
}
