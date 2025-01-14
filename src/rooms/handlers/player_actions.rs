use crate::app_context::{AppContext, RequestContext};
use crate::auth::extractors::User;
use crate::map::models::LatLng;
use crate::storage::interface::IRoomStorage;
use crate::users::handlers::UsersHttpHandler;
use crate::users::responses::{RevokeGuessResponse, SaveGuessResponse, SubmitGuessResponse};
use axum::extract::{Path, State};
use axum::response::Json;

pub async fn save_guess<RS>(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<RS>>,
    Json(guess): Json<LatLng>,
) -> Json<SaveGuessResponse>
where
    RS: IRoomStorage,
{
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .save_guess(guess)
        .await;
    Json(response)
}

pub async fn submit_guess<RS>(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<RS>>,
    Json(guess): Json<LatLng>,
) -> Json<SubmitGuessResponse>
where
    RS: IRoomStorage,
{
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .submit_guess(guess)
        .await;
    Json(response)
}

pub async fn revoke_guess<RS>(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<RS>>,
) -> Json<RevokeGuessResponse>
where
    RS: IRoomStorage,
{
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .revoke_guess()
        .await;
    Json(response)
}
