use axum::body::Bytes;
use image::imageops::FilterType;
use image::{ImageFormat, ImageReader};
use std::io::Cursor;

pub fn resize(image_bytes: Bytes, width: u32, height: u32) -> Vec<u8> {
    let cursor = Cursor::new(image_bytes);
    let image = ImageReader::new(cursor)
        .with_guessed_format()
        .expect("Failed to read image format.")
        .decode()
        .expect("Failed to decode image.");

    let resized_image = image.resize(width, height, FilterType::Lanczos3);

    // For a rough estimate, assume 4 bytes per pixel (RGBA) for a PNG image.
    let estimated_size = (width as usize) * (height as usize) * 4;
    let mut resized_image_bytes_buffer = Cursor::new(Vec::with_capacity(estimated_size));
    resized_image
        .write_to(&mut resized_image_bytes_buffer, ImageFormat::Png)
        .expect("Failed to write the resized image.");

    resized_image_bytes_buffer.into_inner()
}
