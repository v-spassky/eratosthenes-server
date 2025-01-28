pub const AWS_ACCESS_KEY_ID: &str = "AWS_ACCESS_KEY_ID";
pub const AWS_SECRET_ACCESS_KEY: &str = "AWS_SECRET_ACCESS_KEY";
pub const AWS_REGION: &str = "AWS_REGION";
pub const AWS_ENDPOINT_URL: &str = "AWS_ENDPOINT_URL";
pub const S3_FORCE_PATH_STYLE: &str = "S3_FORCE_PATH_STYLE";

#[macro_export]
macro_rules! warn_if_env_var_is_missing {
    ($var:expr, $message:expr) => {
        if let Err(_) = std::env::var($var) {
            tracing::warn!(
                "The `{}` environment variable isn't set. {}",
                $var,
                $message
            );
        }
    };
}
