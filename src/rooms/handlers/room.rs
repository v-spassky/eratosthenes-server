use crate::app_context::{AppContext, RequestContext};
use crate::auth::extractors::User;
use crate::rooms::services::http::{CreateRoomHttpHandler, RoomHttpHandler};
use crate::rooms::services::responses::{
    CreateRoomResponse, RoomMessagesResponse, RoomUsersResponse,
};
use crate::storage::interface::IRoomStorage;
use axum::extract::{Path, State};
use axum::response::Json;

pub async fn create<RS>(
    _user: User,
    State(app_context): State<AppContext<RS>>,
) -> Json<CreateRoomResponse>
where
    RS: IRoomStorage,
{
    let response = CreateRoomHttpHandler::new(app_context).create().await;
    Json(response)
}

pub async fn users<RS>(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<RS>>,
) -> Json<RoomUsersResponse>
where
    RS: IRoomStorage,
{
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

pub async fn messages<RS>(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<RS>>,
) -> Json<RoomMessagesResponse>
where
    RS: IRoomStorage,
{
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
