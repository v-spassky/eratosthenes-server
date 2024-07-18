use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsUserTheHostResponse {
    pub is_host: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitGuessResponse {
    pub error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<GuessSubmissionError>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GuessSubmissionError {
    RoomNotFound,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokeGuessResponse {
    pub error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<GuessRevocationError>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GuessRevocationError {
    RoomNotFound,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MuteUserResponse {
    pub error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<UserMutingError>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UserMutingError {
    RoomNotFound,
    YouAreNotTheHost,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnmuteUserResponse {
    pub error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<UserUnmutingError>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UserUnmutingError {
    RoomNotFound,
    YouAreNotTheHost,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BanUserResponse {
    pub error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<UserBanningError>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UserBanningError {
    RoomNotFound,
    YouAreNotTheHost,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeScoreResponse {
    pub error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<ScoreChangeError>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScoreChangeError {
    RoomNotFound,
    YouAreNotTheHost,
}
