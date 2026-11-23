//! Screenshots of screens and windows, collected in a session folder and
//! bundled as ZIP. Built with help from Claude (AI).
//!
//! The picture source is the trait [`CaptureSource`]. The `xcap` adapter
//! (feature `xcap`, on by default) is one implementation; [`FakeSource`]
//! delivers fixed pictures for tests. No type of the backend appears in this API.
//!
//! ```
//! use fs_capture::{FakeSource, CaptureSource, Session};
//! # let dir = std::env::temp_dir().join(format!("fs-capture-doc-{}", std::process::id()));
//! let source = FakeSource::standard();
//! let mut session = Session::open(&dir).unwrap();
//! let primary = source.primary_screen().unwrap();
//! let entry = session.add_screen(&source, primary.id).unwrap();
//! assert_eq!(entry.nr, 1);
//! # std::fs::remove_dir_all(&dir).unwrap();
//! ```

#![forbid(unsafe_code)]

mod error;
mod fake;
mod image;
mod index;
mod naming;
mod png_io;
mod session;
mod source;
mod time;
#[cfg(feature = "xcap")]
mod xcap_source;
mod zipper;

pub use error::{CaptureError, ErrorCode, Result};
pub use fake::FakeSource;
pub use image::Image;
pub use index::{format_index, parse_index, Entry, Kind, INDEX_FILE};
pub use naming::{capture_file_name, slug, MAX_SLUG_LEN};
pub use png_io::encode_png;
pub use session::Session;
pub use source::{CaptureSource, ScreenId, ScreenInfo, WindowId, WindowInfo};
pub use time::{Clock, FixedClock, SystemClock, UtcTime};
#[cfg(feature = "xcap")]
pub use xcap_source::XcapSource;
