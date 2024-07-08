use crate::auth::responses::AcquireIdResponse;
use crate::auth::user_id;
use std::convert::Infallible;

pub struct AuthHttpHandler {}

impl AuthHttpHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn acquire_id(&self) -> Result<String, Infallible> {
        let response = AcquireIdResponse {
            error: false,
            user_id: user_id::generate(),
        };
        Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
    }
}
