use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketMessage {
    pub r#type: SocketMessageType,
    pub payload: Option<SocketMessagePayload>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SocketMessageType {
    ChatMessage,
    UserConnected,
    UserReConnected,
    UserDisconnected,
    RoundStarted,
    RoundFinished,
    GameFinished,
    GuessSubmitted,
    GuessRevoked,
    UserMuted,
    UserUnmuted,
    UserBanned,
    UserScoreChanged,
    Ping,
    Pong,
    Tick,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SocketMessagePayload {
    ChatMessage(ChatMessagePayload),
    BriefUserInfo(BriefUserInfoPayload),
    Username(UserPubIdInfoPayload),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessagePayload {
    pub from: Option<String>,
    pub content: String,
    pub is_from_bot: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BriefUserInfoPayload {
    pub username: String,
    pub avatar_emoji: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPubIdInfoPayload {
    pub public_id: String,
}
