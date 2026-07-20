use anyhow::Result;
use bytes::Bytes;
use image::{
    imageops::colorops,
    DynamicImage, ImageBuffer, ImageReader,
    Luma, Rgba, RgbaImage,
};
use imageproc::{
    morphology::{self, Mask},
};
use serde::{Deserialize, Serialize};
use std::{
    io::Cursor,
};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LightingOptions {
    pub brightness: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GrayscaleOptions {
    pub morphology: Option<String>,
    pub mask_radius: Option<i32>,
}

/// Apply grayscale filter to image data.
#[wasm_bindgen]
pub fn apply_grayscale_filter(
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    options: JsValue,
) -> Vec<u8> {
    if let Some(image) = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, pixels) {
        let luma_image = DynamicImage::ImageRgba8(image).into_luma8(); //FIXME:
        let (width, height) = luma_image.dimensions();
        let mut morph = String::from("close");
        let mut mask_radius = Mask::disk(1);

        if let Ok(opts) = serde_wasm_bindgen::from_value::<GrayscaleOptions>(options) {
            let radius = opts.mask_radius.unwrap_or(1) as u8;
            if radius != 1 {
                mask_radius = Mask::disk(radius);
            }

            // Set morphology
            if let Some(m) = opts.morphology {
                morph = m;
            }
        }

        // Create grayscale image
        let gray_image = match morph.as_str() {
            "dilate" => morphology::grayscale_dilate(&luma_image, &mask_radius),
            "erode" => morphology::grayscale_erode(&luma_image, &mask_radius),
            "open" => morphology::grayscale_open(&luma_image, &mask_radius),
            _ => morphology::grayscale_close(&luma_image, &mask_radius),
        };

        // Return RGBA version of grayscale image
        return RgbaImage::from_fn(width, height, |x, y| {
            let Luma([gray_val]) = gray_image.get_pixel(x, y);
            Rgba([*gray_val, *gray_val, *gray_val, 255])
        })
        .into_raw();
    }

    vec![]
}

/// Apply sepia filter to image data.
#[wasm_bindgen]
pub fn apply_sepia_filter(
    pixels: Vec<u8>,
    width: u32,
    height: u32,
) -> Vec<u8> {
    if let Some(mut image) = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, pixels) {
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

        return image.into_raw();
    }

    vec![]
}

/// Apply lighting transformation to image data.
#[wasm_bindgen]
pub fn adjust_lighting(
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    options: JsValue,
) -> Vec<u8> {
    if let Some(image) = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, pixels) {
        if let Ok(opts) = serde_wasm_bindgen::from_value::<LightingOptions>(options) {
            if let Some(brightness) = opts.brightness {
                let updated_image = colorops::brighten(&image, brightness);
                return updated_image.into_raw();
            }
        }
    }

    vec![]
}

/// Parse image bytes as RgbaImage.
pub fn parse_image_data(bytes: &[u8]) -> Result<RgbaImage> {
    let image = image::load_from_memory(bytes)?;
    Ok(image.to_rgba8())
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
