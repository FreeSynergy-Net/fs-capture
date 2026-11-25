//! A source with fixed pictures: for tests and demos without a real screen.

use crate::consent::ScreenPermit;
use crate::error::{CaptureError, ErrorCode, Result};
use crate::image::Image;
use crate::source::{CaptureSource, ScreenId, ScreenInfo, WindowId, WindowInfo};

/// Fixed screens and windows. Each capture is a solid picture whose colour
/// depends on the id, so tests can tell the pictures apart.
#[derive(Debug, Clone, Default)]
pub struct FakeSource {
    screens: Vec<ScreenInfo>,
    windows: Vec<WindowInfo>,
}

impl FakeSource {
    /// A source with two screens (1 = primary) and three windows (3 is minimized).
    pub fn standard() -> Self {
        let screen = |id: u64, primary: bool| ScreenInfo {
            id: ScreenId(id),
            name: format!("FAKE-{id}"),
            width: 8,
            height: 6,
            scale: 1.0,
            primary,
        };
        let window = |id: u64, title: &str, app: &str, minimized: bool| WindowInfo {
            id: WindowId(id),
            title: title.to_string(),
            app_name: app.to_string(),
            width: 5,
            height: 4,
            minimized,
        };
        FakeSource {
            screens: vec![screen(1, true), screen(2, false)],
            windows: vec![
                window(1, "Notepad - notes.txt", "notepad.exe", false),
                window(2, "Notepad - todo.txt", "notepad.exe", false),
                window(3, "Calculator", "calc.exe", true),
            ],
        }
    }

    /// A source with your own lists.
    pub fn new(screens: Vec<ScreenInfo>, windows: Vec<WindowInfo>) -> Self {
        FakeSource { screens, windows }
    }
}

fn colour(id: u64, base: u8) -> [u8; 4] {
    [base, (id % 256) as u8, 128, 255]
}

impl CaptureSource for FakeSource {
    fn screens(&self) -> Result<Vec<ScreenInfo>> {
        Ok(self.screens.clone())
    }

    fn windows(&self) -> Result<Vec<WindowInfo>> {
        Ok(self.windows.clone())
    }

    fn capture_screen(&self, id: ScreenId, _permit: &ScreenPermit) -> Result<Image> {
        let s = self.screen_by_id(id)?;
        Image::solid(s.width, s.height, colour(id.0, 10))
    }

    fn capture_window(&self, id: WindowId) -> Result<Image> {
        let w = self.window_by_id(id)?;
        if w.minimized {
            return Err(CaptureError::new(ErrorCode::Backend, "minimized"));
        }
        Image::solid(w.width, w.height, colour(id.0, 200))
    }
}
