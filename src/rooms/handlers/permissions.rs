use crate::app_context::{AppContext, RequestContext};
use crate::auth::extractors::User;
use crate::http::query_params::UsernameQueryParam;
use crate::rooms::services::http::RoomHttpHandler;
use crate::rooms::services::responses::CanConnectToRoomResponse;
use crate::storage::rooms::HashMapRoomsStorage;
use crate::users::handlers::UsersHttpHandler;
use crate::users::responses::IsUserTheHostResponse;
use axum::extract::{Path, Query, State};
use axum::response::Json;

#[axum::debug_handler]
pub async fn can_connect_to_room(
    user: User,
    Path(room_id): Path<String>,
    Query(query_params): Query<UsernameQueryParam>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
) -> Json<CanConnectToRoomResponse> {
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
        // client_ip,
    };
    let response = RoomHttpHandler::new(app_context, &request_context)
        .can_connect(query_params.username)
        .await;
    Json(response)
}

#[axum::debug_handler]
pub async fn is_host(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
) -> Json<IsUserTheHostResponse> {
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
        // client_ip,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .is_host()
        .await;
    Json(response)
}
