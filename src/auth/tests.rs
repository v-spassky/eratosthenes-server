use crate::auth::responses::{
    DecodePasscodeResponse, PasscodeExtractionError, PasscodeExtractionReason,
};
use crate::http::tests::test_server;

pub static PASSCODE: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJwdWJsaWNfaWQiOiJ0ZXN0UHVibGlj\
    SWQiLCJwcml2YXRlX2lkIjoidGVzdFByaXZhdGVJZCJ9.ILcGiySld5q0tcxaipaRl506ZIMp6qrJ5ZlfIM_0fwU";

#[tokio::test]
async fn test_decode_good_passcode() {
    let server = test_server();

    let response = server
        .get("/auth/passcode/decode")
        .add_header("Passcode", PASSCODE)
        .await;

    response.assert_status_ok();
    response.assert_json(&DecodePasscodeResponse {
        error: false,
        public_id: String::from("testPublicId"),
    });
}

#[tokio::test]
async fn test_decode_bad_passcode() {
    let server = test_server();

    let response = server
        .get("/auth/passcode/decode")
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

    let response = server.get("/auth/passcode/decode").await;

    response.assert_status_unauthorized();
    response.assert_json(&PasscodeExtractionError {
        error: true,
        reason: PasscodeExtractionReason::NoPasscodeHeaderProvided,
    });
}
