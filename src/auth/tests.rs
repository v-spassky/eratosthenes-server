use crate::auth::extractors::{PasscodeExtractionError, PasscodeExtractionReason};
use crate::auth::responses::DecodeIdResponse;
use crate::http::tests::test_server;

pub static PASSCODE: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJwdWJsaWNfaWQiOiJ0ZXN0UHVibGlj\
    SWQiLCJwcml2YXRlX2lkIjoidGVzdFByaXZhdGVJZCJ9.ILcGiySld5q0tcxaipaRl506ZIMp6qrJ5ZlfIM_0fwU";

#[tokio::test]
async fn test_decode_good_passcode() {
    let server = test_server();

    let response = server
        .get("/auth/decode-passcode")
        .add_header("Passcode", PASSCODE)
        .await;

    response.assert_status_ok();
    response.assert_json(&DecodeIdResponse {
        error: false,
        public_id: String::from("testPublicId"),
    });
}

#[tokio::test]
async fn test_decode_bad_passcode() {
    let server = test_server();

    let response = server
        .get("/auth/decode-passcode")
        .add_header("Passcode", "notReallyAPasscode")
        .await;

    response.assert_status_unauthorized();
    response.assert_json(&PasscodeExtractionError {
        error: true,
        reason: PasscodeExtractionReason::InvalidPasscode,
    });
}

#[tokio::test]
async fn test_decode_missing_passcode() {
    let server = test_server();

    let response = server.get("/auth/decode-passcode").await;

    response.assert_status_unauthorized();
    response.assert_json(&PasscodeExtractionError {
        error: true,
        reason: PasscodeExtractionReason::NoPasscodeHeaderProvided,
    });
}
