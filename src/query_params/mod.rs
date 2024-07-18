use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserIdsQueryParams {
    pub public_id: String,
    pub private_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct UsernameQueryParam {
    pub username: String,
}
