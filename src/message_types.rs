use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct SocketMessage {
    pub r#type: SocketMessageType,
    pub payload: Option<SocketMessagePayload>,
}

#[derive(Deserialize, Debug)]
pub enum SocketMessageType {
    #[serde(rename = "chatMessage")]
    ChatMessage,
    #[serde(rename = "userConnected")]
    UserConnected,
    #[serde(rename = "userDisconnected")]
    UserDisconnected,
    #[serde(rename = "ping")]
    Ping,
}

impl ToString for SocketMessageType {
    fn to_string(&self) -> String {
        match &self {
            SocketMessageType::ChatMessage => "chatMessage",
            SocketMessageType::UserConnected => "userConnected",
            SocketMessageType::UserDisconnected => "userDisconnected",
            SocketMessageType::Ping => "ping",
        }
        .to_string()
    }
}

impl Serialize for SocketMessageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum SocketMessagePayload {
    ChatMessage(ChatMessagePayload),
    BriefUserInfo(BriefUserInfoPayload),
}

#[derive(Deserialize, Debug)]
pub struct ChatMessagePayload {
    pub from: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct BriefUserInfoPayload {
    pub username: String,
    #[serde(rename = "avatarEmoji")]
    pub avatar_emoji: String,
}
