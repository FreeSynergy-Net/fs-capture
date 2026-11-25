//! Consent for capturing a whole screen.
//!
//! Pattern: Capability object - only [`FullScreenConsent::require`] can hand out a [`ScreenPermit`],
//! and `CaptureSource::capture_screen` needs one, so a whole-screen picture cannot be taken without
//! passing the check. Deviation: the consent itself ([`FullScreenConsent`]) is a plain value object
//! (an enum anyone can write as `Confirmed`); it records the explicit act of the person (CLI:
//! `--full-screen`), it is not a secret. The guard stops accidents (a default, a forgotten flag),
//! not a caller that deliberately writes `Confirmed`.
//! Reason: a whole screen shows the contents of all programs, not only the one asked for.

use crate::error::{CaptureError, ErrorCode, Result};

/// The warning to show whenever a whole screen is captured with consent.
pub const FULL_SCREEN_WARNING: &str =
    "The whole screen is captured; contents of other programs may be in the picture. / Der ganze Bildschirm wird aufgenommen; Inhalte anderer Programme koennen im Bild sein.";

/// Explicit consent to capture a whole screen. Window captures need none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FullScreenConsent {
    /// No consent given (the default): whole-screen captures are refused.
    #[default]
    Denied,
    /// The person asked for a whole screen explicitly (CLI: `--full-screen`).
    Confirmed,
}

/// Proof that the consent was checked. It has a private field, so it cannot be built outside this
/// module; `CaptureSource::capture_screen` takes it.
///
/// ```compile_fail
/// // Cannot be built by hand: the field is private.
/// let _ = fs_capture::ScreenPermit { _private: () };
/// ```
#[derive(Debug)]
pub struct ScreenPermit {
    _private: (),
}

impl FullScreenConsent {
    /// The permit with consent, otherwise `E_FULL_SCREEN_NOT_CONFIRMED` saying why and how to avoid it.
    pub fn require(self) -> Result<ScreenPermit> {
        match self {
            FullScreenConsent::Confirmed => Ok(ScreenPermit { _private: () }),
            FullScreenConsent::Denied => Err(CaptureError::new(
                ErrorCode::FullScreenNotConfirmed,
                "a whole screen shows the contents of all programs (your private data may be in \
                 the picture); capture one window instead (`shot --window <id>` or `shot --title \
                 <text>`) or confirm explicitly with `--full-screen`",
            )),
        }
    }
}
