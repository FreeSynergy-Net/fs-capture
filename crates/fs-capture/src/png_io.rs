//! PNG encoding (one responsibility: image to bytes).

use crate::error::{CaptureError, ErrorCode, Result};
use crate::image::Image;

/// Encodes an image as 8-bit RGBA PNG.
pub fn encode_png(image: &Image) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let png_err = |e: png::EncodingError| CaptureError::new(ErrorCode::Png, e.to_string());
    {
        let mut enc = png::Encoder::new(&mut out, image.width(), image.height());
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().map_err(png_err)?;
        writer.write_image_data(image.rgba()).map_err(png_err)?;
    }
    Ok(out)
}
