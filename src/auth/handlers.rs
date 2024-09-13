use crate::auth::responses::DecodeIdResponse;

use crate::auth::passcode::JwtPayload;

pub struct AuthHttpHandler {
    jwt_payload: JwtPayload,
}

impl AuthHttpHandler {
    pub fn new(jwt_payload: JwtPayload) -> Self {
        Self { jwt_payload }
    }

    pub async fn acquire_passcode(self) -> DecodeIdResponse {
        DecodeIdResponse {
            error: false,
            public_id: Some(self.jwt_payload.public_id),
        }
    }
}
