use crate::auth::JWT_SIGNING_KEY;
use jwt::VerifyWithKey;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JwtPayload {
    pub public_id: String,
    pub private_id: String,
}

pub fn decode(passcode: &str) -> Result<JwtPayload, ()> {
    passcode
        .verify_with_key(
            JWT_SIGNING_KEY
                .get()
                .expect("`JWT_SIGNING_KEY` was not initialized."),
        )
        .map_err(|_err| ())
}
