use std::convert::Infallible;

pub struct HealthHttpHandler {}

impl HealthHttpHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn healthcheck(&self) -> Result<String, Infallible> {
        Ok::<_, Infallible>(String::new())
    }
}
