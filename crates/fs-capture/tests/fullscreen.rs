//! The full-screen guard: no whole-screen capture without explicit consent.

use std::cell::Cell;

use fs_capture::{
    CaptureSource, ErrorCode, FakeSource, FullScreenConsent, Image, Result, ScreenId, ScreenInfo,
    ScreenPermit, Session, WindowId, WindowInfo,
};

/// Spy: forwards to a `FakeSource` and counts the calls that take a picture.
struct Spy {
    inner: FakeSource,
    screen_shots: Cell<u32>,
    window_shots: Cell<u32>,
}

impl Spy {
    fn new() -> Self {
        Spy {
            inner: FakeSource::standard(),
            screen_shots: Cell::new(0),
            window_shots: Cell::new(0),
        }
    }
}

impl CaptureSource for Spy {
    fn screens(&self) -> Result<Vec<ScreenInfo>> {
        self.inner.screens()
    }
    fn windows(&self) -> Result<Vec<WindowInfo>> {
        self.inner.windows()
    }
    fn capture_screen(&self, id: ScreenId, permit: &ScreenPermit) -> Result<Image> {
        self.screen_shots.set(self.screen_shots.get() + 1);
        self.inner.capture_screen(id, permit)
    }
    fn capture_window(&self, id: WindowId) -> Result<Image> {
        self.window_shots.set(self.window_shots.get() + 1);
        self.inner.capture_window(id)
    }
}

#[test]
fn default_consent_is_denied() {
    assert_eq!(FullScreenConsent::default(), FullScreenConsent::Denied);
}

#[test]
fn screen_without_consent_is_refused_and_never_captured() {
    let d = tempfile::tempdir().unwrap();
    let spy = Spy::new();
    let mut s = Session::open(d.path()).unwrap();
    let err = s
        .add_screen(&spy, ScreenId(1), FullScreenConsent::Denied)
        .unwrap_err();
    assert_eq!(err.code(), ErrorCode::FullScreenNotConfirmed);
    assert_eq!(err.code().as_str(), "E_FULL_SCREEN_NOT_CONFIRMED");
    assert!(err.message().contains("--window") && err.message().contains("--full-screen"));
    assert_eq!(
        spy.screen_shots.get(),
        0,
        "the source must not be asked for a picture"
    );
    assert!(s.entries().is_empty());
}

#[test]
fn screen_with_consent_is_captured() {
    let d = tempfile::tempdir().unwrap();
    let spy = Spy::new();
    let mut s = Session::open(d.path()).unwrap();
    s.add_screen(&spy, ScreenId(1), FullScreenConsent::Confirmed)
        .unwrap();
    assert_eq!(spy.screen_shots.get(), 1);
    assert_eq!(s.entries().len(), 1);
}

#[test]
fn window_needs_no_consent() {
    let d = tempfile::tempdir().unwrap();
    let spy = Spy::new();
    let mut s = Session::open(d.path()).unwrap();
    s.add_window(&spy, WindowId(1)).unwrap();
    assert_eq!(spy.window_shots.get(), 1);
    assert_eq!(spy.screen_shots.get(), 0);
}
