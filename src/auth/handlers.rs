use crate::auth::extractors::User;
use crate::auth::responses::DecodeIdResponse;
use axum::response::Json;

#[axum::debug_handler]
pub async fn decode_passcode(user: User) -> Json<DecodeIdResponse> {
    Json(DecodeIdResponse {
        error: false,
        public_id: user.public_id,
    })
}
