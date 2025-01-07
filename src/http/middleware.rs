use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

pub async fn tracing(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    // TODO: use a dedicated extractor
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("127.0.0.1")
        .to_string();

    let start_time = Instant::now();
    let response = next.run(request).await;
    let elapsed_time_ms = start_time.elapsed().as_millis();

    tracing::info!(
        task = "http_request",
        http_method = %method,
        endpoint = %path,
        client_ip = %client_ip,
        processing_time_ms = elapsed_time_ms,
    );

    response
}
