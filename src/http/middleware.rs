use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use axum_client_ip::InsecureClientIp;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::auth::extractors::MaybeUser;

pub async fn tracing(user: MaybeUser, request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|matched_path| matched_path.as_str())
        .unwrap_or(request.uri().path())
        .to_string();
    let client_ip = InsecureClientIp::from(request.headers(), request.extensions())
        .map(|InsecureClientIp(ip)| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let start_time = Instant::now();

    let response = next.run(request).await;

    let elapsed_time_ms = start_time.elapsed().as_millis();
    tracing::info!(
        task = "http_request",
        http_method = %method,
        endpoint = %path,
        private_id = user.private_id.unwrap_or_else(|| "missing".to_string()),
        client_ip = %client_ip,
        processing_time_ms = elapsed_time_ms,
        timestamp,
    );

    response
}
