use crate::app_context::{AppContext, RequestContext};
use crate::auth::extractors::User;
use crate::map_locations::models::LatLng;
use crate::storage::rooms::HashMapRoomsStorage;
use crate::users::handlers::UsersHttpHandler;
use crate::users::responses::{RevokeGuessResponse, SaveGuessResponse, SubmitGuessResponse};
use axum::extract::{Path, State};
use axum::response::Json;

#[axum::debug_handler]
pub async fn save_guess(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
    Json(guess): Json<LatLng>,
) -> Json<SaveGuessResponse> {
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
        // client_ip,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .save_guess(guess)
        .await;
    Json(response)
}

#[axum::debug_handler]
pub async fn submit_guess(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
    Json(guess): Json<LatLng>,
) -> Json<SubmitGuessResponse> {
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
        // client_ip,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .submit_guess(guess)
        .await;
    Json(response)
}

#[axum::debug_handler]
pub async fn revoke_guess(
    user: User,
    Path(room_id): Path<String>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
) -> Json<RevokeGuessResponse> {
    let request_context = RequestContext {
        public_id: user.public_id,
        private_id: user.private_id,
        room_id,
        // client_ip,
    };
    let response = UsersHttpHandler::new(app_context, &request_context)
        .revoke_guess()
        .await;
    Json(response)
}
