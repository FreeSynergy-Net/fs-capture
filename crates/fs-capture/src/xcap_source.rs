//! Adapter for the `xcap` crate (feature `xcap`). No xcap type leaves this file.

use crate::consent::ScreenPermit;
use crate::error::{window_list_error, CaptureError, ErrorCode, Result};
use crate::image::Image;
use crate::source::{CaptureSource, ScreenId, ScreenInfo, WindowId, WindowInfo};

/// Real screens and windows of the current desktop, through `xcap`.
#[derive(Debug, Clone, Copy, Default)]
pub struct XcapSource;

fn backend<E: std::fmt::Display>(e: E) -> CaptureError {
    CaptureError::new(ErrorCode::Backend, e.to_string())
}

fn to_image(img: xcap::image::RgbaImage) -> Result<Image> {
    let (w, h) = (img.width(), img.height());
    Image::new(w, h, img.into_raw())
}

fn find_monitor(id: ScreenId) -> Result<xcap::Monitor> {
    for m in xcap::Monitor::all().map_err(backend)? {
        if m.id().ok().map(u64::from) == Some(id.0) {
            return Ok(m);
        }
    }
    Err(CaptureError::new(
        ErrorCode::UnknownScreen,
        format!("screen {}", id.0),
    ))
}

fn find_window(id: WindowId) -> Result<xcap::Window> {
    for w in xcap::Window::all().map_err(backend)? {
        if w.id().ok().map(u64::from) == Some(id.0) {
            return Ok(w);
        }
    }
    Err(CaptureError::new(
        ErrorCode::UnknownWindow,
        format!("window {}", id.0),
    ))
}

impl CaptureSource for XcapSource {
    fn screens(&self) -> Result<Vec<ScreenInfo>> {
        let mut out = Vec::new();
        for m in xcap::Monitor::all().map_err(backend)? {
            let Ok(id) = m.id() else { continue };
            out.push(ScreenInfo {
                id: ScreenId(u64::from(id)),
                name: m.name().unwrap_or_default(),
                width: m.width().unwrap_or(0),
                height: m.height().unwrap_or(0),
                scale: m.scale_factor().unwrap_or(1.0),
                primary: m.is_primary().unwrap_or(false),
            });
        }
        Ok(out)
    }

    fn windows(&self) -> Result<Vec<WindowInfo>> {
        let mut out = Vec::new();
        for w in xcap::Window::all().map_err(|e| window_list_error(e.to_string()))? {
            let Ok(id) = w.id() else { continue };
            out.push(WindowInfo {
                id: WindowId(u64::from(id)),
                title: w.title().unwrap_or_default(),
                app_name: w.app_name().unwrap_or_default(),
                width: w.width().unwrap_or(0),
                height: w.height().unwrap_or(0),
                minimized: w.is_minimized().unwrap_or(false),
            });
        }
        Ok(out)
    }

    fn capture_screen(&self, id: ScreenId, _permit: &ScreenPermit) -> Result<Image> {
        to_image(find_monitor(id)?.capture_image().map_err(backend)?)
    }

    fn capture_window(&self, id: WindowId) -> Result<Image> {
        to_image(find_window(id)?.capture_image().map_err(backend)?)
    }
}
