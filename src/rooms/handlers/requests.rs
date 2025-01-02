use serde::Deserialize;

#[derive(Deserialize)]
pub struct ScoreChangeRequestBody {
    pub amount: i64,
}
