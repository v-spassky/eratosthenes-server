use crate::health::responses::HealthCheckResponse;
use axum::response::Json;

#[axum::debug_handler]
pub async fn healthcheck() -> Json<HealthCheckResponse> {
    Json(HealthCheckResponse { error: false })
}
