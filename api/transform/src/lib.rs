use anyhow::Result;
use bytes::Bytes;
use image::{
    load_from_memory, DynamicImage, ImageFormat,
    ImageReader, RgbaImage,
};
use imageproc::geometric_transformations::{rotate90, rotate270};
use std::io::Cursor;

use models::{ContentType, Transformations};

pub fn transform_image(
    data: &[u8],
    content_type: &ContentType,
    specs: &Transformations,
) -> Result<Option<Vec<u8>>> {
    let image: RgbaImage = parse_image_data(data)?;

    if let Some(rotate) = specs.rotate {
        match rotate {
            90 => {
                let buffer: RgbaImage = rotate90(&image);
                let rotated_image: Vec<u8> = convert_to_bytes(buffer, content_type)?;
                return Ok(Some(rotated_image))
            }
            270 => {
                let buffer: RgbaImage = rotate270(&image);
                let rotated_image: Vec<u8> = convert_to_bytes(buffer, content_type)?;
                return Ok(Some(rotated_image))
            }
            _ => {}
        }
    }

    Ok(None)
}

fn format_from_image_type(content_type: &ContentType) -> ImageFormat {
    match content_type {
        ContentType::JPEG => ImageFormat::Jpeg,
        ContentType::PNG => ImageFormat::Png,
        ContentType::GIF => ImageFormat::Gif,
        ContentType::WEBP => ImageFormat::WebP,
        ContentType::BMP => ImageFormat::Bmp,
        ContentType::UNKNOWN => ImageFormat::Jpeg,
    }
}

/// Parse image bytes as RgbaImage.
pub fn parse_image_data(bytes: &[u8]) -> anyhow::Result<RgbaImage> {
    let image = load_from_memory(bytes)?;
    Ok(image.to_rgba8())
}

/// Convert RgbaImage to bytes.
fn convert_to_bytes(buffer: RgbaImage, content_type: &ContentType) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let format: ImageFormat = format_from_image_type(content_type);

    let dyn_image = DynamicImage::ImageRgba8(buffer);
    dyn_image.write_to(&mut Cursor::new(&mut bytes), format)?;

    Ok(bytes)
}

/// Parse image dimensions from bytes.
pub fn get_dimensions(
    data: &Bytes,
) -> Result<(u32, u32)> {
    let dimensions = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .into_dimensions()?;

    Ok(dimensions)
}
