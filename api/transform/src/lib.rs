use anyhow::Result;
use bytes::Bytes;
use image::{
    imageops::{self, FilterType},
    math::Rect,
    DynamicImage, ImageFormat, ImageReader,
    Luma, Rgba, RgbaImage,
};
use imageproc::{
    compose::crop,
    geometric_transformations::{rotate90, rotate180, rotate270},
    morphology::{self, Mask},
};
use std::io::Cursor;

use models::{ContentType, Transformations};

pub fn transform_image(
    data: &[u8],
    content_type: &ContentType,
    specs: &Transformations,
) -> Result<Option<Vec<u8>>> {
    let mut image: RgbaImage = parse_image_data(data)?;
    let mut transformed: bool = false;

    if let Some(dimensions) = specs.resize {
        transformed = true;
        image = imageops::resize(
            &image,
            dimensions.width as u32,
            dimensions.height as u32,
            FilterType::Lanczos3,
        );
    }

    if let Some(degrees) = specs.rotate {
        transformed = true;
        image = match degrees {
            90 => rotate90(&image),
            180 => rotate180(&image),
            270 => rotate270(&image),
            _ => {
                transformed = false;
                image
            },
        };
    }

    if let Some(pixels) = specs.crop {
        transformed = true;
        image = crop(&image, Rect {
            x: pixels.x as u32,
            y: pixels.y as u32,
            width: pixels.width as u32,
            height: pixels.height as u32,
        });
    }

    if let Some(filters) = specs.filters {
        if let Some(grayscale) = filters.grayscale && grayscale {
            transformed = true;

            // Create grayscale image
            let gray_image = morphology::grayscale_close(
                &DynamicImage::ImageRgba8(image).into_luma8(),
                &Mask::square(1),
            );
            let (width, height) = gray_image.dimensions();

            // Use RGBA version of grayscale image
            image = RgbaImage::from_fn(width, height, |x, y| {
                let Luma([gray_val]) = gray_image.get_pixel(x, y);
                Rgba([*gray_val, *gray_val, *gray_val, 255])
            });
        }
    }

    println!();
    println!();
    println!("specs: {:?}", &specs); //TODO:REMOVE
    println!("transformed: {:?}", &transformed); //TODO:REMOVE
    println!();
    println!();

    if transformed {
        let new_image: Vec<u8> = convert_to_bytes(image, content_type)?;
        return Ok(Some(new_image));
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
    let image = image::load_from_memory(bytes)?;
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
