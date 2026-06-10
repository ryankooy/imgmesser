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

    // Image resizing
    if let Some(dimensions) = specs.resize {
        transformed = true;
        image = imageops::resize(
            &image,
            dimensions.width as u32,
            dimensions.height as u32,
            FilterType::Lanczos3,
        );
    }

    // Image rotation
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

    // Image cropping
    if let Some(pixels) = specs.crop {
        transformed = true;
        image = crop(&image, Rect {
            x: pixels.x as u32,
            y: pixels.y as u32,
            width: pixels.width as u32,
            height: pixels.height as u32,
        });
    }

    // Image filters
    if let Some(filters) = &specs.filters {
        // Grayscale filter
        if let Some(grayscale) = filters.grayscale && grayscale {
            transformed = true;

            let luma_image = DynamicImage::ImageRgba8(image).into_luma8();
            let (width, height) = luma_image.dimensions();

            let mut morph = "close";
            let mut mask = Mask::square(1);

            if let Some(options) = &filters.options {
                // Set mask shape
                if let Some(mask_type) = &options.mask {
                    let radius = options.radius.unwrap_or(1) as u8;

                    mask = match mask_type.as_str() {
                        "diamond" => Mask::diamond(radius),
                        "disk" => Mask::disk(radius),
                        _ => Mask::square(radius),
                    };
                }

                // Set morphology
                if let Some(m) = &options.morphology {
                    morph = m;
                }
            }

            // Create grayscale image
            let gray_image = match morph {
                "dilate" => morphology::grayscale_dilate(&luma_image, &mask),
                "erode" => morphology::grayscale_erode(&luma_image, &mask),
                "open" => morphology::grayscale_open(&luma_image, &mask),
                _ => morphology::grayscale_close(&luma_image, &mask),
            };

            // Use RGBA version of grayscale image
            image = RgbaImage::from_fn(width, height, |x, y| {
                let Luma([gray_val]) = gray_image.get_pixel(x, y);
                Rgba([*gray_val, *gray_val, *gray_val, 255])
            });
        }

        // Sepia filter
        if let Some(sepia) = filters.sepia && sepia {
            transformed = true;

            for pixel in &mut image.pixels_mut() {
                let r = pixel[0] as f32;
                let g = pixel[1] as f32;
                let b = pixel[2] as f32;

                // Apply sepia matrix
                let new_r = (r * 0.393 + g * 0.769 + b * 0.189).min(255.0) as u8;
                let new_g = (r * 0.349 + g * 0.686 + b * 0.168).min(255.0) as u8;
                let new_b = (r * 0.272 + g * 0.534 + b * 0.131).min(255.0) as u8;

                // Update pixel, preserving original alpha channel
                pixel[0] = new_r;
                pixel[1] = new_g;
                pixel[2] = new_b;
            }
        }
    }

    //TODO: REMOVE:
    println!();
    println!("specs: {:?}", &specs);
    println!("transformed: {:?}", &transformed);
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
