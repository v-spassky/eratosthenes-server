use crate::auth::extractors::User;
use crate::uploads::responses::UploadImageResponse;
use crate::uploads::S3_CLIENT;
use aws_sdk_s3::primitives::ByteStream;
use axum::extract::Multipart;
use axum::response::Json;
use image::imageops::FilterType;
use image::{ImageFormat, ImageReader};
use std::io::Cursor;

pub async fn upload_images(_user: User, mut multipart: Multipart) -> Json<UploadImageResponse> {
    // TODO: The server should generate & return image ids, not the client
    while let Some(field) = multipart.next_field().await.unwrap() {
        let image_name = field
            .name()
            .expect("Failed to read a field name from a multipart form body.")
            .to_string();
        let image_bytes = field
            .bytes()
            .await
            .expect("Failed to read multipart form body field contents.");
        tokio::spawn(async move {
            // Resize and upload the smaller image version first.
            let resized_bytes = image_bytes.clone();
            let original_image_content_length = image_bytes.len();
            let resized_image_name = format!("{}-preview", &image_name);
            let image = ImageReader::new(Cursor::new(resized_bytes))
                .with_guessed_format()
                .expect("Failed to read image format.")
                .decode()
                .expect("Failed to decode image.");
            let resized_image = image.resize(50, 30, FilterType::Lanczos3);
            let mut resized_image_bytes_buffer = Cursor::new(Vec::new());
            resized_image
                .write_to(&mut resized_image_bytes_buffer, ImageFormat::Png)
                .expect("Failed to write resized image.");
            let resized_image_bytes = resized_image_bytes_buffer.into_inner();
            let resized_content_length = resized_image_bytes.len();
            let resized_image_bytes_stream = ByteStream::from(resized_image_bytes);
            S3_CLIENT
                .put_object()
                .bucket("ert-chat-message-images")
                .key(&resized_image_name)
                .body(resized_image_bytes_stream)
                .content_type("image/png")
                .content_length(resized_content_length as i64)
                .send()
                .await
                .expect("Uploading the resized image to S3 failed.");

            // TODO: move operations like this to a dedicated method
            let original_image_bytes_stream = ByteStream::from(image_bytes);
            S3_CLIENT
                .put_object()
                .bucket("ert-chat-message-images")
                .key(&image_name)
                .body(original_image_bytes_stream)
                .content_type("image/png")
                .content_length(original_image_content_length as i64)
                .send()
                .await
                .expect("Uploading an image to S3 failed.");
        });
    }
    Json(UploadImageResponse { error: false })
}
