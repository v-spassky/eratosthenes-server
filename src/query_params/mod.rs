use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserIdQueryParam {
    pub user_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct UsernameQueryParam {
    pub username: String,
}
