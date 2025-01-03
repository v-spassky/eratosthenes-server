use crate::app_context::{AppContext, RequestContext};
use crate::auth::extractors::User;
use crate::http::query_params::UsernameQueryParam;
use crate::rooms::services::http::RoomHttpHandler;
use crate::rooms::services::responses::CanConnectToRoomResponse;
use crate::storage::interface::IRoomStorage;
use crate::users::handlers::UsersHttpHandler;
use crate::users::responses::IsUserTheHostResponse;
use axum::extract::{Path, Query, State};
use axum::response::Json;

pub async fn can_connect_to_room<RS>(
    user: User,
    Path(room_id): Path<String>,
    Query(query_params): Query<UsernameQueryParam>,
    State(app_context): State<AppContext<RS>>,
) -> Json<CanConnectToRoomResponse>
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
        .can_connect(query_params.username)
        .await;
    Json(response)
}

pub async fn is_host<RS>(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<RS>>,
) -> Json<IsUserTheHostResponse>
where
    RS: IRoomStorage,
{
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
