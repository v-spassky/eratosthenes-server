use crate::auth::extractors::User;
use crate::auth::responses::DecodePasscodeResponse;
use axum::response::Json;

pub async fn decode_passcode(user: User) -> Json<DecodePasscodeResponse> {
    Json(DecodePasscodeResponse {
        error: false,
        public_id: user.public_id,
    })
}
