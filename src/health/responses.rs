use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResponse {
    pub error: bool,
}
