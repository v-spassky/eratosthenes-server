use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentLinkRequest {
    pub attachment_ids: Vec<String>,
}
