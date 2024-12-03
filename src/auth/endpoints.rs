use crate::auth::handlers::AuthHttpHandler;
use crate::auth::passcode;
use crate::auth::responses::DecodeIdResponse;
use crate::http::config::{CORS_POLICY, RESPONSE_HEADERS};
use crate::logging::consts::DEFAULT_CLIENT_IP;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::Instant;
use warp::hyper::StatusCode;
use warp::reply::Reply;
use warp::{Filter, Rejection};

pub fn decode_passcode() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("auth")
        .and(warp::path("decode-passcode"))
        .and(warp::header::<String>("Passcode"))
        .and(warp::addr::remote())
        .and_then(
            move |passcode: String, client_ip: Option<SocketAddr>| async move {
                let start_time = Instant::now();
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let jwt_payload = match passcode::decode(&passcode) {
                    Ok(payload) => payload,
                    Err(_) => {
                        let response = DecodeIdResponse {
                            error: true,
                            public_id: None,
                        };
                        return Ok(warp::reply::with_status(
                            serde_json::to_string(&response).unwrap(),
                            StatusCode::UNAUTHORIZED,
                        ));
                    }
                };
                let response = AuthHttpHandler::new(jwt_payload).acquire_passcode().await;
                let processing_time_ns = start_time.elapsed().as_nanos();
                tracing::info!(
                    task = "http_request",
                    http_method = "GET",
                    endpoint = "/auth/acquire-ids",
                    private_id = "no_auth",
                    client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                    processing_time_ms = processing_time_ns / 1000,
                    timestamp,
                );
                Ok::<_, Infallible>(warp::reply::with_status(
                    serde_json::to_string(&response).unwrap(),
                    StatusCode::OK,
                ))
            },
        )
        .with(warp::reply::with::headers(
            RESPONSE_HEADERS
                .get()
                .expect("`RESPONSE_HEADERS` was not initialized.")
                .clone(),
        ))
        .with(
            CORS_POLICY
                .get()
                .expect("`CORS_POLICY` was not initialized.")
                .clone(),
        )
}
