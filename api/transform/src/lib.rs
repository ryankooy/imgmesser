use anyhow::Result;
use bytes::Bytes;
use image::{
    imageops::{self, colorops, FilterType},
    math::Rect,
    DynamicImage, ImageBuffer, ImageFormat, ImageReader,
    Luma, Rgba, RgbaImage,
};
use imageproc::{
    compose::crop,
    geometric_transformations::{rotate90, rotate180, rotate270},
    morphology::{self, Mask},
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::Cursor,
};

use models::{
    ContentType, Crop, Filters, Resize, Transformations,
};

/// Return hash of an image's raw pixels.
fn get_hash(image: &RgbaImage) -> u64 {
    let mut hasher = DefaultHasher::new();
    image.as_raw().hash(&mut hasher);
    hasher.finish()
}

/// Apply transformations to image data.
pub fn transform_image(
    data: &[u8],
    content_type: &ContentType,
    specs: &Transformations,
) -> Result<Option<Vec<u8>>> {
    let mut image: RgbaImage = parse_image_data(data)?;
    let original_hash: u64 = get_hash(&image);

    // Image resizing
    if let Some(dimensions) = specs.resize {
        image = resize_image(&image, dimensions);
    }

    // Image rotation
    if let Some(degrees) = specs.rotate && let Some(img) = rotate_image(
        &image,
        degrees,
    ) {
        image = img;
    }

    // Image cropping
    if let Some(pixels) = specs.crop {
        image = crop_image(&image, pixels);
    }

    // Image filters
    if let Some(filters) = &specs.filters {
        // Grayscale filter
        if let Some(grayscale) = filters.grayscale && grayscale {
            // Create Luma (grayscale) version of image data
            let luma_image = DynamicImage::ImageRgba8(image).into_luma8();
            image = apply_grayscale_filter(luma_image, &filters);
        }

        // Sepia filter
        if let Some(sepia) = filters.sepia && sepia {
            apply_sepia_filter(&mut image);
        }
    }

    // Image lighting
    if let Some(lighting) = &specs.lighting {
        if let Some(brightness) = lighting.brightness {
            adjust_brightness(&mut image, brightness);
        }
    }

    println!(); //TODO: REMOVE
    println!("specs: {:?}", &specs); //TODO: REMOVE

    if get_hash(&image) != original_hash {
        println!("transformed!"); //TODO: REMOVE
        println!(); //TODO: REMOVE

        let new_image: Vec<u8> = convert_to_bytes(image, content_type)?;
        return Ok(Some(new_image));
    }

    println!("NOT transformed!"); //TODO: REMOVE
    println!(); //TODO: REMOVE

    Ok(None)
}

/// Apply resize transformation to image data.
fn resize_image(image: &RgbaImage, dimensions: Resize) -> RgbaImage {
    imageops::resize(
        image,
        dimensions.width as u32,
        dimensions.height as u32,
        FilterType::Lanczos3,
    )
}

/// Apply rotate transformation to image data.
fn rotate_image(image: &RgbaImage, degrees: i32) -> Option<RgbaImage> {
    match degrees {
        90 => Some(rotate90(image)),
        180 => Some(rotate180(image)),
        270 => Some(rotate270(image)),
        _ => None,
    }
}

/// Apply crop transformation to image data.
fn crop_image(image: &RgbaImage, pixels: Crop) -> RgbaImage {
    crop(image, Rect {
        x: pixels.x as u32,
        y: pixels.y as u32,
        width: pixels.width as u32,
        height: pixels.height as u32,
    })
}

/// Apply grayscale filter to image data.
fn apply_grayscale_filter(
    luma_image: ImageBuffer<Luma<u8>, Vec<u8>>,
    filters: &Filters,
) -> RgbaImage {
    let (width, height) = luma_image.dimensions();
    let mut morph = "close";
    let mut mask_radius = Mask::disk(1);

    if let Some(options) = &filters.options {
        let radius = options.mask_radius.unwrap_or(1) as u8;
        if radius != 1 {
            mask_radius = Mask::disk(radius);
        }

        // Set morphology
        if let Some(m) = &options.morphology {
            morph = m;
        }
    }

    // Create grayscale image
    let gray_image = match morph {
        "dilate" => morphology::grayscale_dilate(&luma_image, &mask_radius),
        "erode" => morphology::grayscale_erode(&luma_image, &mask_radius),
        "open" => morphology::grayscale_open(&luma_image, &mask_radius),
        _ => morphology::grayscale_close(&luma_image, &mask_radius),
    };

    // Return RGBA version of grayscale image
    RgbaImage::from_fn(width, height, |x, y| {
        let Luma([gray_val]) = gray_image.get_pixel(x, y);
        Rgba([*gray_val, *gray_val, *gray_val, 255])
    })
}

/// Apply sepia filter to image data, in place.
fn apply_sepia_filter(image: &mut RgbaImage) {
    for pixel in image.pixels_mut() {
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

/// Adjust brightness of image data, in place.
fn adjust_brightness(image: &mut RgbaImage, brightness: i32) {
    colorops::brighten_in_place(image, brightness);
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
