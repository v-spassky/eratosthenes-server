use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodePasscodeResponse {
    pub error: bool,
    pub public_id: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasscodeExtractionError {
    pub error: bool,
    pub reason: PasscodeExtractionReason,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PasscodeExtractionReason {
    NoPasscodeHeaderProvided,
    InvalidPasscode,
}
