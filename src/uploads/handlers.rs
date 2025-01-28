use crate::auth::extractors::User;
use crate::uploads::consts::{PREVIEW_IMAGE_HEIGHT, PREVIEW_IMAGE_WIDTH};
use crate::uploads::img;
use crate::uploads::requests::AttachmentLinkRequest;
use crate::uploads::responses::{AttachmentLink, AttachmentLinksResponse, UploadImageResponse};
use crate::uploads::s3::{S3Client, S3Object, S3ObjectKey};
use aws_sdk_s3::primitives::ByteStream;
use axum::extract::Multipart;
use axum::response::Json;

pub async fn upload_images(_user: User, mut multipart: Multipart) -> Json<UploadImageResponse> {
    let mut image_ids = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let s3_client = S3Client::new();
        let original_image_key = S3ObjectKey::random();
        let resized_image_key = S3ObjectKey::preview_for(&original_image_key);
        image_ids.push(original_image_key.clone().into_inner().to_string());
        let original_image_bytes = field
            .bytes()
            .await
            .expect("Failed to read multipart form body field contents.");

        tokio::spawn(async move {
            // Resize and upload the smaller image version first because it is needed by the
            // frontend immediately after uploading.
            let resized_image_bytes = img::resize(
                original_image_bytes.clone(),
                PREVIEW_IMAGE_WIDTH,
                PREVIEW_IMAGE_HEIGHT,
            );

            let resized_image_object = S3Object {
                size: resized_image_bytes.len() as i64,
                bytes: ByteStream::from(resized_image_bytes),
                key: resized_image_key,
                content_type: "image/png",
            };
            let _ = s3_client.put_object(resized_image_object).await;

            let original_image_object = S3Object {
                size: original_image_bytes.len() as i64,
                bytes: ByteStream::from(original_image_bytes),
                key: original_image_key,
                content_type: "image/png",
            };
            let _ = s3_client.put_object(original_image_object).await;
        });
    }

    Json(UploadImageResponse {
        error: false,
        image_ids,
    })
}

pub async fn attachment_links(
    _user: User,
    Json(payload): Json<AttachmentLinkRequest>,
) -> Json<AttachmentLinksResponse> {
    let s3_client = S3Client::new();
    let mut links = Vec::new();

    for attachment_id in payload.attachment_ids {
        let key = match uuid::Uuid::parse_str(&attachment_id) {
            Ok(uuid) => S3ObjectKey::from_uuid(uuid),
            Err(_) => continue,
        };
        match s3_client.generate_presigned_url(&key).await {
            Ok((full, preview)) => links.push(AttachmentLink { full, preview }),
            Err(err) => {
                eprintln!(
                    "Failed to generate presigned URL for key {}: {:?}",
                    attachment_id, err
                );
            }
        }
    }

    Json(AttachmentLinksResponse {
        error: false,
        links,
    })
}
