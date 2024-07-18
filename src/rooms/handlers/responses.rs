use serde::Serialize;

use crate::{
    rooms::models::{ChatMessage, RoomStatus},
    users::models::User,
};

// TODO: consider refactoring single strict with optional fields into multiple structs

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanConnectToRoomResponse {
    /// Whether the client can connect to the room of interest.
    pub can_connect: bool,
    /// Reason of connection refusal, if `can_connect` is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<ConnectionRefusalError>,
}

/// All possible reasons why a user may be denied connection to a room.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionRefusalError {
    RoomNotFound,
    UserAlreadyInRoom,
    UsernameTooLong,
    UserBanned,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
// TODO: refactor & rename this response (status is about room, not users)
pub struct RoomUsersResponse {
    pub error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<RoomUsersResponseError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<User>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RoomStatus>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RoomUsersResponseError {
    RoomNotFound,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomMessagesResponse {
    pub error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<RoomMessagesResponseError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<ChatMessage>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RoomMessagesResponseError {
    RoomNotFound,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomResponse {
    pub room_id: String,
}
