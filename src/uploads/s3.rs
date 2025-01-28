use crate::uploads::env::{AWS_ENDPOINT_URL, AWS_REGION, S3_FORCE_PATH_STYLE};
use aws_config::environment::EnvironmentVariableCredentialsProvider;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::http::HttpResponse;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::put_object::{PutObjectError, PutObjectOutput};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Config};
use std::env;
use std::sync::LazyLock;
use std::time::Duration;
use uuid::Uuid;

static S3_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    // TODO: check the environment variables during initialization
    let region = env::var(AWS_REGION).expect("The `AWS_REGION` environment variable is not set.");
    let endpoint_url = env::var(AWS_ENDPOINT_URL);
    let force_path_style = env::var(S3_FORCE_PATH_STYLE).map(|v| v == "true");

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

#[derive(Debug)]
pub struct S3Client {}

impl S3Client {
    const BUCKET_NAME: &str = "ert-chat-message-images";
    const PRESIGNED_URL_DURATION: Duration = Duration::from_secs(3600);

    pub fn new() -> Self {
        Self {}
    }

    pub async fn put_object(
        &self,
        object: S3Object<'_>,
    ) -> Result<PutObjectOutput, SdkError<PutObjectError, HttpResponse>> {
        S3_CLIENT
            .put_object()
            .bucket(Self::BUCKET_NAME)
            .key(object.key.into_inner())
            .body(object.bytes)
            .content_type(object.content_type)
            .content_length(object.size)
            .send()
            .await
    }

    pub async fn generate_presigned_url(
        &self,
        key: &S3ObjectKey,
    ) -> Result<(String, String), SdkError<GetObjectError, HttpResponse>> {
        let presigning_config = PresigningConfig::expires_in(Self::PRESIGNED_URL_DURATION)
            .expect("Failed to create a presigning config.");
        // TODO: Perform those two requests im parallel, or maybe there is a way to get presigned
        // URLs in batches.
        // TODO: Cache the responses for the duration of the presigned URL.
        let original_attachment_url = S3_CLIENT
            .get_object()
            .bucket(Self::BUCKET_NAME)
            .key(key.as_ref())
            .presigned(presigning_config.clone())
            .await?
            .uri()
            .to_string();
        let preview_attachment_url = S3_CLIENT
            .get_object()
            .bucket(Self::BUCKET_NAME)
            .key(format!("{}-preview", key.as_ref()))
            .presigned(presigning_config)
            .await?
            .uri()
            .to_string();
        Ok((original_attachment_url, preview_attachment_url))
    }
}

#[derive(Debug)]
pub struct S3Object<'ct> {
    pub bytes: ByteStream,
    pub key: S3ObjectKey,
    pub size: i64,
    pub content_type: &'ct str,
}

#[derive(Debug, Clone)]
pub struct S3ObjectKey(String);

impl S3ObjectKey {
    pub fn random() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn preview_for(key: &Self) -> Self {
        let inner = format!("{}-preview", key.clone().into_inner());
        Self(inner)
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid.to_string())
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for S3ObjectKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
