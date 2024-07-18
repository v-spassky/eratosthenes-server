use crate::health::responses::HealthCheckResponse;

pub struct HealthHttpHandler {}

impl HealthHttpHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn healthcheck(&self) -> HealthCheckResponse {
        HealthCheckResponse { error: false }
    }
}
