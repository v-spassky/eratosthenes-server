use crate::health::responses::HealthCheckResponse;
use crate::http::tests::test_server;

#[tokio::test]
async fn test_health_check() {
    let server = test_server();

    let response = server.get("/health/check").await;

    response.assert_status_ok();
    response.assert_json(&HealthCheckResponse { error: false });
}
