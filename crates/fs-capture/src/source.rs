//! The picture source: what a backend has to offer.

use crate::consent::ScreenPermit;
use crate::error::{CaptureError, ErrorCode, Result};
use crate::image::Image;

/// Identifier of a screen, valid for one source and one moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScreenId(pub u64);

/// Identifier of a window, valid for one source and one moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub u64);

/// A screen (monitor).
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenInfo {
    /// Id for `capture_screen`.
    pub id: ScreenId,
    /// Name given by the system.
    pub name: String,
    /// Width in pixels as reported by the backend.
    pub width: u32,
    /// Height in pixels as reported by the backend.
    pub height: u32,
    /// Scale factor (1.0 = no scaling).
    pub scale: f32,
    /// Is this the primary screen?
    pub primary: bool,
}

/// A window.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    /// Id for `capture_window`.
    pub id: WindowId,
    /// Window title (may be empty).
    pub title: String,
    /// Name of the program (may be empty).
    pub app_name: String,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Is the window minimized?
    pub minimized: bool,
}

/// Source of pictures. Backends implement this; callers only use this trait,
/// so the backend can be replaced (for example by a Wayland portal source).
pub trait CaptureSource {
    /// Lists the screens.
    fn screens(&self) -> Result<Vec<ScreenInfo>>;
    /// Lists the windows.
    fn windows(&self) -> Result<Vec<WindowInfo>>;
    /// Captures one whole screen. Needs a [`ScreenPermit`] (only `FullScreenConsent::require` makes one).
    fn capture_screen(&self, id: ScreenId, permit: &ScreenPermit) -> Result<Image>;
    /// Captures one window.
    fn capture_window(&self, id: WindowId) -> Result<Image>;

    /// The primary screen.
    fn primary_screen(&self) -> Result<ScreenInfo> {
        self.screens()?
            .into_iter()
            .find(|s| s.primary)
            .ok_or_else(|| CaptureError::new(ErrorCode::NoPrimaryScreen, "no primary screen"))
    }

    /// The screen with this id.
    fn screen_by_id(&self, id: ScreenId) -> Result<ScreenInfo> {
        self.screens()?
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| CaptureError::new(ErrorCode::UnknownScreen, format!("screen {}", id.0)))
    }

    /// The window with this id.
    fn window_by_id(&self, id: WindowId) -> Result<WindowInfo> {
        self.windows()?
            .into_iter()
            .find(|w| w.id == id)
            .ok_or_else(|| CaptureError::new(ErrorCode::UnknownWindow, format!("window {}", id.0)))
    }

    /// The single window whose title contains `text` (case-insensitive).
    /// Zero matches and several matches are errors.
    fn window_by_title(&self, text: &str) -> Result<WindowInfo> {
        let needle = text.to_lowercase();
        let mut hits: Vec<WindowInfo> = self
            .windows()?
            .into_iter()
            .filter(|w| w.title.to_lowercase().contains(&needle))
            .collect();
        match hits.len() {
            0 => Err(CaptureError::new(
                ErrorCode::UnknownWindow,
                format!("no window with title containing {text:?}"),
            )),
            1 => Ok(hits.remove(0)),
            n => {
                let ids: Vec<String> = hits.iter().map(|w| w.id.0.to_string()).collect();
                Err(CaptureError::new(
                    ErrorCode::AmbiguousWindow,
                    format!(
                        "{n} windows match {text:?} (ids: {}); use --window <id>",
                        ids.join(", ")
                    ),
                ))
            }
        }
    }
}
