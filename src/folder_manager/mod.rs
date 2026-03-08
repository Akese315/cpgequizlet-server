use crate::constants::IMAGES_DIRECTORY;
use actix_multipart::form::tempfile::TempFile;
use std::fs::create_dir_all;

use std::io;
pub fn add_image(image: TempFile, uuid: &str) -> io::Result<String> {
    create_dir_all(IMAGES_DIRECTORY)?;

    let extension = match image.content_type.as_ref() {
        Some(content_type) => content_type.subtype(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid image context type",
            ));
        }
    };

    let filename = format!("{}{}.{}", IMAGES_DIRECTORY, uuid, extension);

    // We use persist instead of read+write because actix_multipart TempFile abstracts NamedTempFile
    // which allows persisting directly to a new path.
    match image.file.persist(&filename) {
        Ok(_) => Ok(filename),
        Err(e) => {
            println!("Error persisting tempfile: {}", e);
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Failed to persist file",
            ))
        }
    }
}

pub fn read_image(file_name: &str) -> io::Result<Vec<u8>> {
    let image = std::fs::read(file_name)?;
    Ok(image)
}

pub async fn download_and_save_image(url: &str, uuid: &str) -> io::Result<String> {
    create_dir_all(IMAGES_DIRECTORY)?;

    let response = reqwest::get(url).await.map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to download image: {}", e),
        )
    })?;

    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to download image: HTTP Status Not Success",
        ));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let extension = match content_type {
        "image/jpeg" => "jpeg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "png",
    };

    let filename = format!("{}{}.{}", IMAGES_DIRECTORY, uuid, extension);

    let bytes = response.bytes().await.map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Failed to read bytes: {}", e))
    })?;

    std::fs::write(&filename, bytes)?;

    Ok(filename)
}
