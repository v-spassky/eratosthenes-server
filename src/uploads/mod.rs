pub mod handlers;
pub mod responses;

use aws_config::environment::EnvironmentVariableCredentialsProvider;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client, Config};
use std::env;
use std::sync::LazyLock;

pub static S3_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    // TODO: check the environment variables during initialization
    let region = env::var("AWS_REGION").expect("The `AWS_REGION` environment variable is not set.");
    let endpoint_url = env::var("AWS_ENDPOINT_URL");
    let force_path_style = env::var("S3_FORCE_PATH_STYLE").map(|v| v == "true");

    let mut config_builder = Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(region))
        .credentials_provider(EnvironmentVariableCredentialsProvider::new());
    if let Ok(endpoint_url) = endpoint_url {
        // Configure custom endpoint URL for local development (used when connecting to LocalStack
        // instead of AWS S3).
        config_builder = config_builder.endpoint_url(endpoint_url);
    }
    if let Ok(force_path_style) = force_path_style {
        // Enable path-style addressing for LocalStack compatibility (LocalStack uses a different
        // URL structure than AWS S3).
        config_builder = config_builder.force_path_style(force_path_style);
    }
    let config = config_builder.build();

    Client::from_conf(config)
});
