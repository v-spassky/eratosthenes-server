use crate::health::responses::HealthCheckResponse;
use axum::response::Json;

pub async fn healthcheck() -> Json<HealthCheckResponse> {
    Json(HealthCheckResponse { error: false })
}
