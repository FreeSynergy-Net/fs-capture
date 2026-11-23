//! Own image type: 8-bit RGBA, row by row, top to bottom.

use crate::error::{CaptureError, ErrorCode, Result};

/// A captured picture. No type of the backend crate appears here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl Image {
    /// Creates an image; the pixel buffer must be exactly `width * height * 4` bytes
    /// and the image must not be empty.
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Result<Self> {
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4));
        if width == 0 || height == 0 || expected != Some(rgba.len()) {
            return Err(CaptureError::new(
                ErrorCode::BadImage,
                format!("{width}x{height} does not match {} bytes", rgba.len()),
            ));
        }
        Ok(Image {
            width,
            height,
            rgba,
        })
    }

    /// A single-colour image (used by the fake source and tests).
    pub fn solid(width: u32, height: u32, rgba: [u8; 4]) -> Result<Self> {
        let n = (width as usize).saturating_mul(height as usize);
        let mut buf = Vec::with_capacity(n.saturating_mul(4));
        for _ in 0..n {
            buf.extend_from_slice(&rgba);
        }
        Image::new(width, height, buf)
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// RGBA bytes.
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }
}
