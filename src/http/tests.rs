use crate::cli::tests::fake_args;
use crate::http::router;
use crate::{app_context, auth};
use axum_test::TestServer;

pub fn test_server() -> TestServer {
    let args = fake_args();
    auth::init(&args);
    let app_context = app_context::init();
    let router = router::new(&args, app_context);
    TestServer::new(router).expect("Failed to run test server.")
}
