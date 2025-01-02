use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeIdResponse {
    pub error: bool,
    pub public_id: String,
}
