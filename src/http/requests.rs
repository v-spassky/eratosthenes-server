use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PasscodeQueryParam {
    pub passcode: String,
}

#[derive(Serialize, Deserialize)]
pub struct UsernameQueryParam {
    pub username: String,
}
