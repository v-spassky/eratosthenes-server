use crate::app_context::{AppContext, RequestContext};
use crate::auth::extractors::User;
use crate::storage::rooms::HashMapRoomsStorage;
use crate::users::handlers::UsersHttpHandler;
use crate::users::responses::{
    BanUserResponse, ChangeScoreResponse, MuteUserResponse, UnmuteUserResponse,
};
use axum::extract::{Path, State};
use axum::response::Json;

use super::requests::ScoreChangeRequestBody;

#[axum::debug_handler]
pub async fn mute_user(
    user: User,
    Path((room_id, user_id)): Path<(String, String)>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
) -> Json<MuteUserResponse> {
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
        // client_ip,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .mute(user_id)
        .await;
    Json(response)
}

#[axum::debug_handler]
pub async fn unmute_user(
    user: User,
    Path((room_id, user_id)): Path<(String, String)>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
) -> Json<UnmuteUserResponse> {
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
        // client_ip,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .unmute(user_id)
        .await;
    Json(response)
}

#[axum::debug_handler]
pub async fn ban_user(
    user: User,
    Path((room_id, user_id)): Path<(String, String)>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
) -> Json<BanUserResponse> {
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
        // client_ip,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .ban(user_id)
        .await;
    Json(response)
}

#[axum::debug_handler]
pub async fn change_user_score(
    user: User,
    Path((room_id, user_id)): Path<(String, String)>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
    Json(score): Json<ScoreChangeRequestBody>,
) -> Json<ChangeScoreResponse> {
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
        // client_ip,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .change_score(user_id, score.amount)
        .await;
    Json(response)
}
