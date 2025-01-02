use crate::app_context::{AppContext, RequestContext};
use crate::auth::extractors::User;
use crate::rooms::services::http::{CreateRoomHttpHandler, RoomHttpHandler};
use crate::rooms::services::responses::{
    CreateRoomResponse, RoomMessagesResponse, RoomUsersResponse,
};
use crate::storage::rooms::HashMapRoomsStorage;
use axum::extract::{Path, State};
use axum::response::Json;

#[axum::debug_handler]
pub async fn create(
    _user: User,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
) -> Json<CreateRoomResponse> {
    let response = CreateRoomHttpHandler::new(app_context).create().await;
    Json(response)
}

#[axum::debug_handler]
pub async fn users(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
) -> Json<RoomUsersResponse> {
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
        // client_ip,
    };
    let response = RoomHttpHandler::new(app_context, &request_context)
        .users()
        .await;
    Json(response)
}

#[axum::debug_handler]
pub async fn messages(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
) -> Json<RoomMessagesResponse> {
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
        // client_ip,
    };
    let response = RoomHttpHandler::new(app_context, &request_context)
        .messages()
        .await;
    Json(response)
}
