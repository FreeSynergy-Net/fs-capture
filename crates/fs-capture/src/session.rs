//! A session: one folder with PNG files and the list `index.tsv`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{CaptureError, ErrorCode, Result};
use crate::image::Image;
use crate::index::{format_index, parse_index, Entry, Kind, INDEX_FILE};
use crate::naming::{capture_file_name, is_session_file_name};
use crate::png_io::encode_png;
use crate::source::{CaptureSource, ScreenId, WindowId};
use crate::time::{Clock, SystemClock};
use crate::zipper::bundle_zip;

/// A capture session bound to one folder. All output goes only into this folder.
pub struct Session {
    dir: PathBuf,
    entries: Vec<Entry>,
    clock: Box<dyn Clock>,
}

impl Session {
    /// Opens the folder (created if missing) and reads its list.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self> {
        Session::open_with_clock(dir, Box::new(SystemClock))
    }

    /// Like `open`, with your own clock (tests).
    pub fn open_with_clock(dir: impl Into<PathBuf>, clock: Box<dyn Clock>) -> Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)
            .map_err(|e| CaptureError::new(ErrorCode::Io, format!("{}: {e}", dir.display())))?;
        let index_path = dir.join(INDEX_FILE);
        let entries = match fs::read_to_string(&index_path) {
            Ok(text) => parse_index(&text)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        Ok(Session {
            dir,
            entries,
            clock,
        })
    }

    /// The session folder.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The captures, in order of their numbers.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Captures a screen and adds it.
    pub fn add_screen(&mut self, source: &dyn CaptureSource, id: ScreenId) -> Result<Entry> {
        let info = source.screen_by_id(id)?;
        let image = source.capture_screen(id)?;
        self.add_image(Kind::Screen, &info.name, "", &image)
    }

    /// Captures a window and adds it. Minimized windows are refused.
    pub fn add_window(&mut self, source: &dyn CaptureSource, id: WindowId) -> Result<Entry> {
        let info = source.window_by_id(id)?;
        if info.minimized {
            return Err(CaptureError::new(
                ErrorCode::WindowMinimized,
                format!("window {} is minimized; restore it first", id.0),
            ));
        }
        let image = source.capture_window(id)?;
        self.add_image(Kind::Window, &info.title, &info.app_name, &image)
    }

    /// Adds a ready image (writes the PNG, then the list).
    pub fn add_image(
        &mut self,
        kind: Kind,
        title: &str,
        app_name: &str,
        image: &Image,
    ) -> Result<Entry> {
        let nr = self.entries.iter().map(|e| e.nr).max().unwrap_or(0) + 1;
        let time = self.clock.now();
        let file = capture_file_name(nr, &time.to_compact(), kind.as_str(), title);
        let png = encode_png(image)?;
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.dir.join(&file))?;
        f.write_all(&png)?;
        f.sync_all()?;
        let entry = Entry {
            nr,
            time,
            kind,
            title: title.to_string(),
            app_name: app_name.to_string(),
            width: image.width(),
            height: image.height(),
            file,
        };
        self.entries.push(entry.clone());
        if let Err(e) = self.write_index() {
            // Keep list and folder consistent: take the new capture back.
            self.entries.pop();
            let _ = fs::remove_file(self.dir.join(&entry.file));
            return Err(e);
        }
        Ok(entry)
    }

    /// Deletes capture number `nr` (file and list line).
    pub fn remove(&mut self, nr: u32) -> Result<Entry> {
        let pos = self
            .entries
            .iter()
            .position(|e| e.nr == nr)
            .ok_or_else(|| CaptureError::new(ErrorCode::UnknownCapture, format!("capture {nr}")))?;
        let entry = self.entries.remove(pos);
        if let Err(e) = self.write_index() {
            self.entries.insert(pos, entry);
            return Err(e);
        }
        if is_session_file_name(&entry.file) {
            match fs::remove_file(self.dir.join(&entry.file)) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(entry)
    }

    /// Bundles all captures and the list into a ZIP at `target`.
    /// An existing `target` is never overwritten.
    pub fn bundle(&self, target: &Path) -> Result<()> {
        let bytes = bundle_zip(&self.dir, &self.entries)?;
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
        Ok(())
    }

    fn write_index(&self) -> Result<()> {
        let tmp = self.dir.join(format!("{INDEX_FILE}.tmp"));
        fs::write(&tmp, format_index(&self.entries))?;
        fs::rename(&tmp, self.dir.join(INDEX_FILE))?;
        Ok(())
    }
}
