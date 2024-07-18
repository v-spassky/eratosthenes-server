use crate::auth::responses::AcquireIdResponse;
use crate::auth::user_id;

pub struct AuthHttpHandler {}

impl AuthHttpHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn acquire_id(&self) -> AcquireIdResponse {
        AcquireIdResponse {
            error: false,
            public_id: Some(user_id::generate()),
            private_id: Some(user_id::generate()),
        }
    }
}
