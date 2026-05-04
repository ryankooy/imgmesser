use anyhow::Result;
use image::{
    load_from_memory, DynamicImage, ImageFormat,
    ImageReader, ImageResult, RgbaImage,
};
use imageproc::geometric_transformations::rotate90;
use std::io::Cursor;

use models::{ContentType, Transformations, UploadImage};

pub fn transform_image(
    data: &[u8],
    content_type: &ContentType,
    specs: &Transformations,
) -> Result<Option<Vec<u8>>> {
    let image: RgbaImage = parse_image_data(data)?;

    if let Some(rotate) = specs.rotate {
        match rotate {
            90 => {
                let rotated_image: Vec<u8> = rotate_image_90(&image, content_type)?;
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

/// Parse image data.
pub fn parse_image_data(bytes: &[u8]) -> anyhow::Result<RgbaImage> {
    //let dimensions = ImageReader::new(Cursor::new(bytes))
    //    .with_guessed_format()?
    //    .decode()?;
    let image = load_from_memory(bytes)?;
    Ok(image.to_rgba8())
}

pub fn rotate_image_90(image: &RgbaImage, content_type: &ContentType) -> Result<Vec<u8>> {
    let buffer = rotate90(image);
    let dyn_image = DynamicImage::ImageRgba8(buffer);

    let mut bytes = Vec::new();
    let format: ImageFormat = format_from_image_type(content_type);

    dyn_image.write_to(&mut Cursor::new(&mut bytes), format)?;

    Ok(bytes)
}
