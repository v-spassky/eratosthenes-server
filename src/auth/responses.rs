use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeIdResponse {
    pub error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_id: Option<String>,
}
