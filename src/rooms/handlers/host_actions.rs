use crate::app_context::{AppContext, RequestContext};
use crate::auth::extractors::User;
use crate::storage::interface::IRoomStorage;
use crate::users::handlers::UsersHttpHandler;
use crate::users::responses::{
    BanUserResponse, ChangeScoreResponse, MuteUserResponse, UnmuteUserResponse,
};
use axum::extract::{Path, State};
use axum::response::Json;

use super::requests::ScoreChangeRequestBody;

pub async fn mute_user<RS>(
    user: User,
    Path((room_id, user_id)): Path<(String, String)>,
    State(app_context): State<AppContext<RS>>,
) -> Json<MuteUserResponse>
where
    RS: IRoomStorage,
{
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .mute(user_id)
        .await;
    Json(response)
}

pub async fn unmute_user<RS>(
    user: User,
    Path((room_id, user_id)): Path<(String, String)>,
    State(app_context): State<AppContext<RS>>,
) -> Json<UnmuteUserResponse>
where
    RS: IRoomStorage,
{
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .unmute(user_id)
        .await;
    Json(response)
}

pub async fn ban_user<RS>(
    user: User,
    Path((room_id, user_id)): Path<(String, String)>,
    State(app_context): State<AppContext<RS>>,
) -> Json<BanUserResponse>
where
    RS: IRoomStorage,
{
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .ban(user_id)
        .await;
    Json(response)
}

pub async fn change_user_score<RS>(
    user: User,
    Path((room_id, user_id)): Path<(String, String)>,
    State(app_context): State<AppContext<RS>>,
    Json(score): Json<ScoreChangeRequestBody>,
) -> Json<ChangeScoreResponse>
where
    RS: IRoomStorage,
{
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .change_score(user_id, score.amount)
        .await;
    Json(response)
}
