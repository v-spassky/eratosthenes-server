use crate::models::ChatMessage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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
    #[serde(rename = "userReConnected")]
    UserReConnected,
    #[serde(rename = "userDisconnected")]
    UserDisconnected,
    #[serde(rename = "gameStarted")]
    GameStarted,
    #[serde(rename = "gameFinished")]
    GameFinished,
    #[serde(rename = "guessSubmitted")]
    GuessSubmitted,
    #[serde(rename = "guessRevoked")]
    GuessRevoked,
    #[serde(rename = "ping")]
    Ping,
}

impl ToString for SocketMessageType {
    fn to_string(&self) -> String {
        match &self {
            SocketMessageType::ChatMessage => "chatMessage",
            SocketMessageType::UserConnected => "userConnected",
            SocketMessageType::UserReConnected => "userReConnected",
            SocketMessageType::UserDisconnected => "userDisconnected",
            SocketMessageType::GameStarted => "gameStarted",
            SocketMessageType::GameFinished => "gameFinished",
            SocketMessageType::GuessSubmitted => "guessSubmitted",
            SocketMessageType::GuessRevoked => "guessRevoked",
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SocketMessagePayload {
    ChatMessage(ChatMessagePayload),
    BriefUserInfo(BriefUserInfoPayload),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessagePayload {
    pub from: String,
    pub content: String,
}

impl ChatMessagePayload {
    pub fn to_model(&self) -> ChatMessage {
        ChatMessage {
            author_name: self.from.clone(),
            content: self.content.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BriefUserInfoPayload {
    pub username: String,
    #[serde(rename = "avatarEmoji")]
    pub avatar_emoji: String,
}
