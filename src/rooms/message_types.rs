use serde::{Deserialize, Serialize};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ClientSentSocketMessage {
    ChatMessage {
        #[allow(dead_code)]
        // This field is actually being pattern-matched on. Same for other variants.
        r#type: ChatMessage,
        payload: ClientSentChatMessagePayload,
    },
    UserConnected {
        #[allow(dead_code)]
        r#type: UserConnected,
        payload: BriefUserInfoPayload,
    },
    UserReConnected {
        #[allow(dead_code)]
        r#type: UserReConnected,
        payload: BriefUserInfoPayload,
    },
    UserDisconnected {
        #[allow(dead_code)]
        r#type: UserDisconnected,
        payload: BriefUserInfoPayload,
    },
    RoundStarted {
        #[allow(dead_code)]
        r#type: RoundStarted,
    },
    Ping {
        #[allow(dead_code)]
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
    BotMessage {
        r#type: BotMessage,
        /// TODO: ideally `id` should be inside `payload`
        id: usize,
        payload: BotMessagePayload,
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
pub struct BotMessage;

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
#[serde(rename_all = "camelCase")]
pub struct ClientSentChatMessagePayload {
    pub from: String,
    pub content: String,
    pub attachment_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSentChatMessagePayload {
    pub id: usize,
    pub from: String,
    pub content: String,
    pub attachment_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum BotMessagePayload {
    RoundStarted {
        r#type: RoundStartedBotMsg,
        payload: RoundStartedBotMessagePayload,
    },
    RoundEnded {
        r#type: RoundEndedBotMsg,
        payload: RoundEndedBotMessagePayload,
    },
    UserConnected {
        r#type: UserConnectedBotMsg,
        payload: UserConnectedBotMessagePayload,
    },
    UserDisconnected {
        r#type: UserDisconnectedBotMsg,
        payload: UserDisconnectedBotMessagePayload,
    },
}

#[derive(Clone, Debug, Serialize_unit_struct)]
pub struct RoundStartedBotMsg;

#[derive(Clone, Debug, Serialize_unit_struct)]
pub struct RoundEndedBotMsg;

#[derive(Clone, Debug, Serialize_unit_struct)]
pub struct UserConnectedBotMsg;

#[derive(Clone, Debug, Serialize_unit_struct)]
pub struct UserDisconnectedBotMsg;

#[derive(Clone, Debug, Serialize)]
pub struct RoundStartedBotMessagePayload {
    pub round_number: u64,
    pub rounds_per_game: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct RoundEndedBotMessagePayload {
    pub round_number: u64,
    pub rounds_per_game: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserConnectedBotMessagePayload {
    pub username: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserDisconnectedBotMessagePayload {
    pub username: String,
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
