use serde::{Deserialize, Serialize};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ClientSentSocketMessage {
    ChatMessage {
        r#type: ChatMessage,
        payload: ClientSentChatMessagePayload,
    },
    UserConnected {
        r#type: UserConnected,
        payload: BriefUserInfoPayload,
    },
    UserReConnected {
        r#type: UserReConnected,
        payload: BriefUserInfoPayload,
    },
    UserDisconnected {
        r#type: UserDisconnected,
        payload: BriefUserInfoPayload,
    },
    RoundStarted {
        r#type: RoundStarted,
    },
    Ping {
        r#type: Ping,
    },
}

#[macro_export]
macro_rules! name_of {
    ($name:ident) => {{
        let _ = &$name;
        stringify!($name)
    }};
}

impl ClientSentSocketMessage {
    pub fn message_type_as_string(&self) -> String {
        match self {
            ClientSentSocketMessage::ChatMessage { .. } => name_of!(ChatMessage),
            ClientSentSocketMessage::UserConnected { .. } => name_of!(UserConnected),
            ClientSentSocketMessage::UserReConnected { .. } => name_of!(UserReConnected),
            ClientSentSocketMessage::UserDisconnected { .. } => name_of!(UserDisconnected),
            ClientSentSocketMessage::RoundStarted { .. } => name_of!(RoundStarted),
            ClientSentSocketMessage::Ping { .. } => name_of!(Ping),
        }
        .to_string()
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ServerSentSocketMessage {
    ChatMessage {
        r#type: ChatMessage,
        payload: ServerSentChatMessagePayload,
    },
    UserConnected {
        r#type: UserConnected,
        payload: BriefUserInfoPayload,
    },
    UserDisconnected {
        r#type: UserDisconnected,
        payload: BriefUserInfoPayload,
    },
    RoundStarted {
        r#type: RoundStarted,
    },
    GameFinished {
        r#type: GameFinished,
    },
    RoundFinished {
        r#type: RoundFinished,
    },
    GuessSubmitted {
        r#type: GuessSubmitted,
    },
    GuessRevoked {
        r#type: GuessRevoked,
    },
    UserMuted {
        r#type: UserMuted,
    },
    UserUnmuted {
        r#type: UserUnmuted,
    },
    UserBanned {
        r#type: UserBanned,
        payload: UserPubIdInfoPayload,
    },
    UserScoreChanged {
        r#type: UserScoreChanged,
    },
    Pong {
        r#type: Pong,
    },
    Tick {
        r#type: Tick,
        payload: i32,
    },
}

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct ChatMessage;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct UserConnected;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct UserReConnected;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct UserDisconnected;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct RoundStarted;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct GameFinished;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct RoundFinished;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct GuessSubmitted;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct GuessRevoked;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct UserMuted;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct UserUnmuted;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct UserBanned;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct UserScoreChanged;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct Ping;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct Pong;

#[derive(Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct Tick;

#[derive(Debug, Deserialize)]
pub struct ClientSentChatMessagePayload {
    pub from: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSentChatMessagePayload {
    pub id: usize,
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
